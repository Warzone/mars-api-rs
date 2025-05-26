use std::collections::HashMap;
use std::iter::Map;

use rocket::{http::{ContentType, Status}, Request, response::{self, Responder, Response}, serde::json::Json};
use rocket::serde::json::serde_json;
use rocket_okapi::{gen::OpenApiGenerator, okapi::{Map as OkapiMap, schemars, map}, response::OpenApiResponderInner, Result as OkapiResult};
use rocket_okapi::okapi::openapi3::{MediaType, RefOr, Response as OkapiResponse, Responses, SchemaObject};
use rocket_okapi::okapi::schemars::schema::Schema;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{database::models::{player::{Player, SimplePlayer}, punishment::Punishment, session::Session}, socket::leaderboard::ScoreType};

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct PlayerPreLoginRequest {
    pub player: SimplePlayer,
    pub ip: String
}

pub type PlayerLoginRequest = PlayerPreLoginRequest;

pub struct PlayerPreLoginResponder {
    pub response: PlayerPreLoginResponse
}

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlayerPreLoginResponse {
    pub new: bool,
    pub allowed: bool,
    pub player: Player,
    pub active_punishments: Vec<Punishment>
}

impl<'r> Responder<'r, 'static> for PlayerPreLoginResponder {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let is_new = self.response.new;
        let data = Json(self.response);
        Response::build_from(data.respond_to(req)?)
            .header(ContentType::JSON)
            .status(if is_new { Status::Created } else { Status::Ok })
            .ok()
    }
}

pub struct PlayerLoginResponder {
    pub response: PlayerLoginResponse
}

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlayerLoginResponse {
    pub active_session: Session
}

impl<'r> Responder<'r, 'static> for PlayerLoginResponder {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let data = Json(self.response);
        Response::build_from(data.respond_to(req)?)
            .header(ContentType::JSON)
            .status(Status::Created)
            .ok()
    }
}

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlayerLogoutRequest {
    pub player: SimplePlayer,
    pub session_id: String,
    pub playtime: u64
}

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlayerLookupResponse {
    pub player: Player,
    pub alts: Vec<PlayerAltResponse>
}

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlayerAltResponse {
    pub player: Player,
    pub punishments: Vec<Punishment>
}


#[derive(Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlayerProfileResponse {
    pub player: Player,
    pub leaderboard_positions: HashMap<ScoreType, u64>
}

pub enum PlayerProfileResponder {
    RawProfile(Player),
    ProfileWithLeaderboardPositions(PlayerProfileResponse)
}

impl <'r> Responder<'r, 'static> for PlayerProfileResponder {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        match &self {
            PlayerProfileResponder::RawProfile(profile) => {
                let data = Json(profile);
                Response::build_from(data.respond_to(req)?)
                    .header(ContentType::JSON)
                    .status(Status::Ok)
                    .ok()
            },
            PlayerProfileResponder::ProfileWithLeaderboardPositions(wrapped) => {
                let data = Json(wrapped);
                Response::build_from(data.respond_to(req)?)
                    .header(ContentType::JSON)
                    .status(Status::Ok)
                    .ok()
            },
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct PlayerAddNoteRequest {
    pub author: SimplePlayer,
    pub content: String
}

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlayerSetActiveTagRequest {
    pub active_tag_id: Option<String>
}
