use rocket::{Rocket, State, Build, serde::json::Json};

use crate::{MarsAPIState, database::models::{join_sound::JoinSound, player::Player}, util::{auth::AuthorizationToken, responder::JsonResponder, error::ApiErrorResponder}};

use self::payload::JoinSoundSetRequest;

mod payload;

#[get("/join_sounds")]
fn get_join_sounds(
    state: &State<MarsAPIState>
) -> Json<&Vec<JoinSound>> {
    Json(&state.config.data.join_sounds)
}

#[post("/join_sounds/<player_id>/sound", format = "json", data = "<set_join_req>")]
async fn update_join_sound(
    state: &State<MarsAPIState>,
    player_id: &str,
    set_join_req: Json<JoinSoundSetRequest>,
    _auth_guard: AuthorizationToken
) -> Result<JsonResponder<Player>, ApiErrorResponder> {
    match state.player_cache.get(&state.database, player_id).await {
        Some(mut p) => {
            let current_sound = set_join_req.0.active_join_sound_id;
            if p.active_join_sound_id == current_sound {
                return Ok(JsonResponder::ok(p));
            };
            p.active_join_sound_id = current_sound;
            state.player_cache.set(&state.database, &p.name, &p, true).await;
            Ok(JsonResponder::ok(p))
        },
        None => {
            Err(ApiErrorResponder::missing_player())
        }
    }
}

pub fn mount(rocket_build: Rocket<Build>, state: &MarsAPIState) -> Rocket<Build> {
    rocket_build.mount("/mc/perks", routes![
        get_join_sounds,
        update_join_sound
    ])
}
