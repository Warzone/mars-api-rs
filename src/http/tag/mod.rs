use log::info;
use mongodb::{results::DeleteResult, bson::doc};
use rocket::{State, Rocket, Build, http::Status, serde::json::Json};
use uuid::Uuid;

use crate::{util::{auth::AuthorizationToken, responder::JsonResponder, error::{ApiErrorResponder}, time::get_u64_time_millis, r#macro::unwrap_helper}, MarsAPIState, database::{models::tag::Tag, Database}};

use self::payload::TagCreateRequest;

mod payload;

#[post("/", format = "json", data = "<tag_create_req>")]
async fn create_tag(
    state: &State<MarsAPIState>,
    tag_create_req: Json<TagCreateRequest>,
    _auth_guard: AuthorizationToken
) -> Result<JsonResponder<Tag>, ApiErrorResponder> {
    match state.database.find_by_id_or_name::<Tag>(&tag_create_req.name).await {
        Some(_tag) => return Err(ApiErrorResponder::tag_conflict()),
        None => {},
    };

    let TagCreateRequest { name, display } = tag_create_req.0;

    let tag = Tag {
        id: Uuid::new_v4().to_string(),
        name: name.clone(),
        name_lower: name.clone().to_lowercase(),
        display: display.clone(),
        created_at: get_u64_time_millis() as f64,
    };

    state.database.save::<Tag>(&tag).await;
    return Ok(JsonResponder::from(tag, Status::Ok));
}

#[get("/")]
async fn get_tags(state: &State<MarsAPIState>) -> Json<Vec<Tag>> {
    Json(state.database.get_all_documents::<Tag>().await)
}

#[get("/<tag_id>")]
async fn get_tag_by_id(
    state: &State<MarsAPIState>,
    tag_id: &str,
) -> Result<JsonResponder<Tag>, ApiErrorResponder> {
    Ok(JsonResponder::ok(
        unwrap_helper::return_default!(
            state.database.find_by_id_or_name(tag_id).await,
            Err(ApiErrorResponder::tag_missing())
        )
    ))
}

// delete("/{tagId}") {
//     protected(this) { _ ->
//         val id = call.parameters["tagId"] ?: throw ValidationException()
//         val result = Database.tags.deleteById(id)
//         if (result.deletedCount == 0L) throw TagMissingException()
//         call.respond(Unit)

//         val playersWithTag = Database.players.find(Player::tagIds contains id).toList()
//         playersWithTag.forEach {
//             it.tagIds = it.tagIds.filterNot { tagId -> tagId == id }
//             if (it.activeTagId == id) it.activeTagId = null
//             PlayerCache.set(it.name, it, persist = true)
//         }
//         application.log.info(
//             "Tag '$id' was deleted. Affected players: ${
//                 playersWithTag.joinToString(", ") { "${it._id} (${it.name})" }
//             }"
//         )
//     }
// }

#[delete("/<tag_id>")]
async fn delete_tag(
    state: &State<MarsAPIState>,
    tag_id: &str,
    _auth_guard: AuthorizationToken
) -> Result<(), ApiErrorResponder> {
    match state.database.delete_by_id::<Tag>(tag_id).await {
        Some(DeleteResult { deleted_count: 0, .. }) | None => {
            return Err(ApiErrorResponder::tag_missing());
        },
        _ => {}
    };
    let mut players_with_tag = Database::consume_cursor_into_owning_vec_option(
        state.database.players.find(doc! {"tagIds": tag_id}, None).await.ok()
    ).await;
    for player in players_with_tag.iter_mut() {
        match player.tag_ids.iter().position(|e| { e == &tag_id.to_string() }) {
            Some(tag_idx) => { player.tag_ids.swap_remove(tag_idx); },
            None => {},
        };
        if player.active_tag_id.is_some() && player.active_tag_id.as_ref().unwrap() == &tag_id.to_string() {
            player.active_tag_id = Option::None;
        }
        state.player_cache.set(&state.database, &player.name, &player, true).await;
    };
    info!(
        "Tag {} was deleted. Affected players: {}", 
        tag_id, 
        players_with_tag.iter()
            .map(|p| { format!("{} {}", &p.id, &p.name) })
            .collect::<Vec<String>>()
            .join(", ")
    );
    Ok(())
}

#[put("/<tag_id>", format = "json", data = "<tag_update_req>")]
async fn update_tag(
    state: &State<MarsAPIState>,
    tag_update_req: Json<TagCreateRequest>,
    tag_id: &str,
    _auth_guard: AuthorizationToken
) -> Result<JsonResponder<Tag>, ApiErrorResponder> {
    match state.database.find_by_id_or_name::<Tag>(tag_id).await {
        Some(tag) => {
            let updated_tag = Tag {
                id: tag.id.clone(),
                name: tag_update_req.name.clone(),
                name_lower: tag_update_req.name.clone().to_lowercase(),
                display: tag_update_req.display.clone(),
                created_at: tag.created_at
            };
            if let Ok(bson_tag) = mongodb::bson::to_bson(&updated_tag) {
                if let Some(tag_document) = bson_tag.as_document() {
                    state.database.tags.update_one(
                        doc! { "_id": tag.id.clone() }, 
                        doc! { "$set": tag_document }, 
                        None
                    ).await;
                }
            };
            Ok(JsonResponder::ok(updated_tag))
        }
        None => {
            Err(ApiErrorResponder::tag_missing())
        }
    }
}

pub fn mount(rocket_build: Rocket<Build>, state: &MarsAPIState) -> Rocket<Build> {
    rocket_build.mount("/mc/tags", routes![
        create_tag,
        get_tags,
        get_tag_by_id,
        delete_tag,
        update_tag
    ])
}
