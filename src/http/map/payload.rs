use serde::{Serialize, Deserialize};

use crate::database::models::level::{LevelGamemode, LevelContributor};

#[derive(Serialize, Deserialize, Debug)]
pub struct MapLoadOneRequest {
    #[serde(rename = "_id")] 
    pub id: String,
    pub name: String,
    #[serde(default)] 
    pub slug: Option<String>,
    pub version: String,
    pub gamemodes: Vec<LevelGamemode>,
    pub authors: Vec<LevelContributor>,
    pub contributors: Vec<LevelContributor>,
}
