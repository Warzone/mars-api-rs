use rocket::{Rocket, Build, serde::json::Json, State};

use crate::{database::{models::punishment::{PunishmentType, Punishment, PunishmentReversion}, Database}, MarsAPIState, util::{error::ApiErrorResponder, auth::AuthorizationToken, r#macro::unwrap_helper, time::get_u64_time_millis}};

use self::payloads::PunishmentRevertRequest;

pub mod payloads;

#[get("/types")]
fn get_pun_types(state: &State<MarsAPIState>, _auth_guard: AuthorizationToken) -> Json<&Vec<PunishmentType>> {
    Json(&state.config.data.punishment_types)
}

#[get("/<punishment_id>")]
async fn get_pun(
    state: &State<MarsAPIState>, 
    punishment_id: &str, 
    _auth_guard: AuthorizationToken
) -> Result<Json<Punishment>, ApiErrorResponder> {
    Ok(Json(unwrap_helper::return_default!(Database::find_by_id(&state.database.punishments, punishment_id).await, Err(ApiErrorResponder::missing_punishment()))))
}

#[post("/<punishment_id>/revert", format = "json", data = "<revert_req>")]
async fn revert_pun(
    state: &State<MarsAPIState>, 
    punishment_id: &str, 
    revert_req: Json<PunishmentRevertRequest>, 
    _auth_guard: AuthorizationToken
) -> Result<Json<Punishment>, ApiErrorResponder> {
    let data = revert_req.0;
    let mut punishment = unwrap_helper::return_default!(Database::find_by_id(&state.database.punishments, punishment_id).await, Err(ApiErrorResponder::missing_punishment()));
    punishment.reversion = Some(PunishmentReversion { reverted_at: get_u64_time_millis(), reverter: data.reverter, reason: data.reason });
    state.database.save(&punishment).await;
    {
        // take ownership for the spawned task
        let pun_clone = punishment.clone();
        let state_clone = state.config.clone();
        tokio::spawn(async move {
            state_clone.webhooks.send_punishment_reversion_webhook(&pun_clone).await;
        });
    }
    Ok(Json(punishment))
}

pub fn mount(rocket: Rocket<Build>, state: &MarsAPIState) -> Rocket<Build> {
    rocket.mount("/mc/punishments", routes![get_pun_types, get_pun, revert_pun])
}
