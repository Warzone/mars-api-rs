use rocket::{serde::json::Json, State, Build, Rocket};
use crate::{database::models::broadcast::Broadcast, MarsAPIState};

#[get("/")]
pub fn broadcasts(state: &State<MarsAPIState>) -> Json<&Vec<Broadcast>> {
    Json(&state.config.data.broadcasts) 
}

pub fn mount(rocket_build: Rocket<Build>, state: &MarsAPIState) -> Rocket<Build> {
    rocket_build.mount("/mc/broadcasts", routes![broadcasts])
}
