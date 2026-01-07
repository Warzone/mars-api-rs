use std::{path::Path, sync::Arc};

use futures::{FutureExt, StreamExt, future::join_all};
use image::{create_image_decoder, parse_image_data};
use mongodb::bson::doc;
use rocket::{Build, Data, Rocket, State, data::ToByteUnit, fs::FileServer, serde::json::Json};
use tokio::sync::RwLock;
use tokio_util::codec::FramedRead;

use crate::{MarsAPIState, database::{Database, models::level::{Level, LevelRecords}}, http::map::payload::MapLoadOneRequest, util::{auth::AuthorizationToken, error::ApiErrorResponder, r#macro::unwrap_helper, stream::LengthPrefixedDataDecoder, time::get_u64_time_millis}};

mod payload;
pub mod image;

#[post("/", format = "json", data = "<maps>")]
async fn add_maps(
    state: &State<MarsAPIState>,
    maps: Json<Vec<MapLoadOneRequest>>,
    _auth_guard: AuthorizationToken
) -> Json<Vec<Level>> {
    let map_list = maps.0;
    let map_list_length = map_list.len();
    let time_millis = get_u64_time_millis();
    let mut maps_to_save : Vec<Level> = Vec::new();
    
    let query_tasks : Vec<_> = map_list.iter().map(|map| {
        state.database.find_by_fields::<Level>(
            map.slug.as_ref().unwrap_or(&map.name), 
            vec![String::from("slug"), String::from("nameLower")]
        )
    }).collect();
    let level_docs = join_all(query_tasks).await;
    for (map, level_opt) in map_list.into_iter().zip(level_docs) {
        maps_to_save.push(if let Some(mut existing_map) = level_opt {
            existing_map.name = map.name;
            existing_map.name_lower = existing_map.name.to_lowercase();
            existing_map.version = map.version;
            existing_map.gamemodes = map.gamemodes;
            existing_map.authors = map.authors;
            existing_map.updated_at = time_millis;
            existing_map.contributors = map.contributors;
            existing_map
        } else {
            // do this before the move
            let lowercase_map_name = map.name.to_lowercase();
            Level {
                id: map.id,
                name: map.name,
                name_lower: lowercase_map_name,
                slug: map.slug,
                version: map.version,
                gamemodes: map.gamemodes,
                loaded_at: time_millis,
                updated_at: time_millis,
                authors: map.authors,
                contributors: map.contributors,
                records: LevelRecords::default(),
                goals: None,
                last_match_id: None
            }
        });
    }


    let tasks = {
        let mut all_tasks = Vec::new();
        let save_tasks : Vec<_> = 
            maps_to_save.iter().map(|map| { state.database.save(map).boxed() }).collect();
        let update_map_state = {
            let rwlock = state.map_state.last_update.clone();
            async move {
                let mut timestamp = rwlock.write().await;
                *timestamp = time_millis;
            }.boxed()
        };
        all_tasks.extend(save_tasks);
        all_tasks.push(update_map_state);
        all_tasks
    };
    join_all(tasks).await;

    info!("Received {} maps. Updating {} maps.", map_list_length, maps_to_save.len());
    Json(state.database.get_all_documents().await)
}

#[post("/images", format = "application/octet-stream", data = "<image_data>")]
async fn add_map_images(
    state: &State<MarsAPIState>,
    image_data: Data<'_>,
    _auth_guard: AuthorizationToken
) {
    let t1 = get_u64_time_millis();
    let Some(image_state) = state.image_state.as_ref() else {return};
    let decoder = create_image_decoder();
    let ds = image_data.open(128.mebibytes());
    let mut framed = FramedRead::new(ds, decoder);
    let mut frame_count = 0;
    while let Some(frame_result) = framed.next().await {
        frame_count += 1;
        match frame_result {
            Ok(frame) => {
                if frame.len() == 0 {
                    break;
                }
                let data = parse_image_data(frame);
                image_state.transmit.send(data).await;
            },
            Err(e) => {
                warn!("Bailed reading more images: {:?}", e);
            },
        }
    }
    let t2 = get_u64_time_millis();
    info!("Took {}ms to store {} map images", t2 - t1, frame_count);
}

#[get("/")]
async fn get_all_maps(state: &State<MarsAPIState>) -> Json<Vec<Level>> {
    Json(state.database.get_all_active_maps(*state.map_state.last_update.read().await).await)
}

#[get("/<map_id>")]
async fn get_map_by_id(state: &State<MarsAPIState>, map_id: &str) -> Result<Json<Level>, ApiErrorResponder> {
    let map = unwrap_helper::return_default!(Database::find_by_id(&state.database.levels, map_id).await, Err(ApiErrorResponder::missing_map()));
    Ok(Json(map))
}

#[derive(Clone)]
pub struct MapState {
    pub last_update: Arc<RwLock<u64>>
}

pub fn mount(build: Rocket<Build>, state: &MarsAPIState) -> Rocket<Build> {
    let build = build.mount(
        "/mc/maps", 
        routes![add_maps, get_all_maps, get_map_by_id, add_map_images]
    );
    if let Some(images_path) = &state.config.options.images_path {
        if !std::fs::exists(&images_path).unwrap_or(false) {
            std::fs::create_dir(&images_path).expect("Could not create images directory");
        }
        build.mount("/mc/maps/images", FileServer::from(Path::new(images_path.as_str())))
    } else {
        build
    }
}