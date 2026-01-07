use rocket::{Rocket, Build};
use rocket::serde::{Serialize, json::Json};
use rocket::http::Status;

use crate::MarsAPIState;

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct StatusResponse {
    status: &'static str
}

#[get("/")]
pub fn status() -> Json<StatusResponse> {
   Json(StatusResponse { status: Status::Ok.reason().unwrap_or("OK") }) 
}

pub fn mount(rocket_build: Rocket<Build>, state: &MarsAPIState) -> Rocket<Build> {
    rocket_build.mount("/status", routes![status])
}
