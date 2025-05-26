use mars_api_rs_derive::IdentifiableDocument;
use mars_api_rs_macro::IdentifiableDocument;
use rocket_okapi::okapi::schemars;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum_macros::Display;

use crate::{database::CollectionOwner, util::time::get_u64_time_millis};

use super::player::SimplePlayer;

#[derive(Debug, Serialize, Deserialize, JsonSchema, IdentifiableDocument, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Punishment {
    #[id]
    #[serde(rename = "_id")]
    pub id: String,
    pub reason: PunishmentReason,
    pub issued_at: f64,
    pub silent: bool,
    pub offence: u32,
    pub action: PunishmentAction,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub punisher: Option<SimplePlayer>,
    pub target: SimplePlayer,
    pub target_ips: Vec<String>,
    #[serde(default)]
    pub reversion: Option<PunishmentReversion>,
    #[serde(default)]
    pub server_id: Option<String>
}

impl Punishment {
    pub fn expires_at(&self) -> i64 {
        if self.action.length == -1 {
            -1
        } else {
            (self.issued_at as i64) + self.action.length
        }
    }

    pub fn is_active(&self) -> bool {
        if self.reversion.is_some() {
            return false;
        } else {
            return self.action.length == -1 || (get_u64_time_millis() as i64) < self.expires_at()
        }
    }
}

impl CollectionOwner<Punishment> for Punishment {
    fn get_collection(database: &crate::database::Database) -> &mongodb::Collection<Punishment> {
        &database.punishments
    }

    fn get_collection_name() -> &'static str {
        "punishment"
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PunishmentType {
    pub name: String,
    pub short: String,
    pub message: String,
    pub actions: Vec<PunishmentAction>,
    pub material: String,
    pub position: u32,
    #[serde(default)]
    pub tip: Option<String>,
    #[serde(default = "default_required_permission")]
    pub required_permission: String
}


#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct PunishmentAction {
    pub kind: PunishmentKind,
    #[serde(default = "default_punishment_length")]
    length: i64
}

impl PunishmentAction {
    pub fn is_ban(&self) -> bool {
        self.kind == PunishmentKind::Ban || self.kind == PunishmentKind::IpBan
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum PunishmentKind {
    Warn,
    Kick,
    Mute,
    Ban,
    IpBan
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StaffNote {
    pub id: u32,
    pub author: SimplePlayer,
    pub content: String,
    pub created_at: u64
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct PunishmentReason {
    pub name: String, 
    message: String, 
    short: String
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PunishmentReversion {
    pub reverted_at: u64,
    pub reverter: SimplePlayer,
    pub reason: String
}

// default providers

fn default_required_permission() -> String {
    String::from("mars.punish")
}

fn default_punishment_length() -> i64 {
    0
}
