mod payload;

use rocket::{serde::json::Json, State, Build, Rocket};

use crate::{util::{auth::AuthorizationToken, error::ApiErrorResponder}, MarsAPIState};

use self::payload::ReportCreateRequest;


#[post("/", format = "json", data = "<report>")]
pub async fn new_report(
    state: &State<MarsAPIState>,
    report: Json<ReportCreateRequest>,
    auth_guard: AuthorizationToken,
) -> Result<(), ApiErrorResponder> {
    let data = report.0;
    state.config.webhooks.send_report_webhook(
        &auth_guard.server_id,
        &data.reporter, 
        &data.target, 
        &data.reason, 
        &data.online_staff
    ).await;
    Ok(())
}

pub fn mount(rocket_build: Rocket<Build>, state: &MarsAPIState) -> Rocket<Build> {
    rocket_build.mount("/mc/reports", routes![new_report])
}
