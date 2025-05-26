use log::warn;
use mars_api_rs_derive::IdentifiableDocument;
use mars_api_rs_macro::IdentifiableDocument;
use rocket_okapi::okapi::schemars;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::database::CollectionOwner;

use super::player::SimplePlayer;

#[derive(Deserialize, Serialize, JsonSchema, IdentifiableDocument, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    #[id]
    #[serde(rename = "_id")]
    pub id: String,
    pub ip: String,
    pub player: SimplePlayer,
    pub server_id: String,
    pub created_at: u64,
    pub ended_at: Option<u64>
} 

impl Session {
    pub fn is_active(&self) -> bool {
        self.ended_at.is_none()
    }

    pub fn length(&self) -> Option<u64> {
        if self.ended_at.is_none() { None } else { 
            let ended_at = self.ended_at.unwrap();
            if ended_at < self.created_at {
                warn!("Session length is negative");
                None 
            } else { 
                Some(ended_at - self.created_at) 
            }
        } 
    }
}

impl CollectionOwner<Session> for Session {
    fn get_collection(database: &crate::database::Database) -> &mongodb::Collection<Session> {
        &database.sessions
    }

    fn get_collection_name() -> &'static str {
        "session"
    }
}
