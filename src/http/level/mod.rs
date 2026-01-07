use rocket::{Rocket, Build, State, serde::json::Json};
use crate::{MarsAPIState, database::models::level_color::LevelColor};

#[get("/colors")]
fn get_level_colors(state: &State<MarsAPIState>) -> Json<&Vec<LevelColor>> {
    Json(&state.config.data.level_colors)
}

pub fn mount(rocket_build: Rocket<Build>, state: &MarsAPIState) -> Rocket<Build> {
    rocket_build.mount("/mc/levels", routes![get_level_colors])
}
