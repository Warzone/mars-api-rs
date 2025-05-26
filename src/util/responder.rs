use rocket::{
    http::{ContentType, Status},
    Request,
    response::{self, Responder, Response},
    serde::json::Json,
};
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::okapi;
use rocket_okapi::okapi::openapi3::{RefOr, Responses};
use rocket_okapi::okapi::openapi3;
use rocket_okapi::okapi::schemars;
use rocket_okapi::response::OpenApiResponderInner;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub struct JsonResponder<T> {
    pub response: T,
    pub status: Status
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct EmptyResponse {}

impl<T: Serialize> JsonResponder<T> {
    pub fn created(data: T) -> Self {
        JsonResponder::from(data, Status::Created)
    }

    pub fn ok(data: T) -> Self {
        JsonResponder::from(data, Status::Ok)
    }

    pub fn from(data: T, status: Status) -> Self {
        Self { response: data, status }
    }
}

impl<'r, T: Serialize> Responder<'r, 'static> for JsonResponder<T> {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let data = Json(self.response);
        Response::build_from(data.respond_to(req)?)
            .header(ContentType::JSON)
            .status(self.status)
            .ok()
    }
}

// TODO: Handle more success cases. This just returns 200 for all responses by default.
impl<T> OpenApiResponderInner for JsonResponder<T>
where
    T: JsonSchema + Send + Sync,
{
    fn responses(gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        let mut responses = Responses::default();
        let schema = gen.json_schema::<T>();
        responses.responses.insert(
            "200".to_owned(),
            RefOr::Object(openapi3::Response {
                description: "Successful response".to_owned(),
                content: {
                    let mut content = okapi::Map::new();
                    content.insert(
                        "application/json".to_owned(),
                        openapi3::MediaType {
                            schema: Some(schema),
                            ..Default::default()
                        },
                    );
                    content
                },
                ..Default::default()
            }),
        );
        Ok(responses)
    }
}