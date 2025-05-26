use rocket::serde::json::serde_json;
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::okapi::openapi3::{MediaType, RefOr, Response as OkapiResponse, Responses, SchemaObject};
use rocket_okapi::okapi::{map, schemars};
use rocket_okapi::okapi::schemars::schema::Schema;
use rocket_okapi::response::OpenApiResponderInner;
use crate::database::models::player::Player;
use crate::http::player::payloads::{PlayerLoginResponder, PlayerLoginResponse, PlayerPreLoginResponder, PlayerPreLoginResponse, PlayerProfileResponder, PlayerProfileResponse};

fn create_200_response<T: schemars::JsonSchema>(gen: &mut OpenApiGenerator, responses: &mut Responses) {
    responses.responses.insert(
        "200".to_owned(),
        RefOr::Object(OkapiResponse {
            description: "Successful response".to_owned(),
            content: map! {
                "application/json".to_owned() => MediaType {
                    schema: Some(gen.json_schema::<T>()),
                    ..Default::default()
                }
            },
            ..Default::default()
        }),
    );
}

impl OpenApiResponderInner for PlayerProfileResponder {
    fn responses(gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        let player_schema = gen.json_schema::<Player>();
        let profile_resp_schema = gen.json_schema::<PlayerProfileResponse>();

        let mut responses = Responses::default();

        let mut schema = SchemaObject {
            subschemas: Some(Box::new(schemars::schema::SubschemaValidation {
                one_of: Some(vec![
                    Schema::from(player_schema),
                    Schema::from(profile_resp_schema),
                ]),
                ..Default::default()
            })),
            ..Default::default()
        };

        // Add discriminator to the schema to differentiate between Player and PlayerProfileResponse
        schema.extensions.insert(
            "discriminator".to_string(),
            serde_json::json!({
                "propertyName": "include_leaderboard_positions",
                "mapping": {
                    "false": "#/components/schemas/Player",
                    "true": "#/components/schemas/PlayerProfileResponse"
                }
            }),
        );

        let mut content = map! {
            "application/json".to_owned() => MediaType {
                schema: Some(schema),
                ..Default::default()
            }
        };

        responses.responses.insert(
            "200".to_owned(),
            RefOr::Object(OkapiResponse {
                description: "Successful response".to_owned(),
                content,
                ..Default::default()
            }),
        );

        Ok(responses)
    }
}

impl OpenApiResponderInner for PlayerPreLoginResponder {
    fn responses(gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        let mut responses = Responses::default();

        create_200_response::<PlayerPreLoginResponse>(gen, &mut responses);

        Ok(responses)
    }

}

impl OpenApiResponderInner for PlayerLoginResponder {
    fn responses(gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        let mut responses = Responses::default();

        create_200_response::<PlayerLoginResponse>(gen, &mut responses);

        Ok(responses)
    }
}