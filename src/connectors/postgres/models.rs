use crate::{
    connectors::{self, postgres::schema::current_experiences},
    experience_code::ExperienceCode,
};
use chrono::{NaiveDateTime, Utc};
use connectors::postgres::schema::experiences;
use diesel::prelude::*;
use grpc_rust::modules::communitygames::PlaygroundInfo;

#[derive(AsChangeset, Queryable, Selectable, Insertable)]
#[diesel(table_name = experiences)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Experience {
    pub experience_id: i32,
    pub share_code: String,
    pub playground_name: String,
    pub playground_description: String,
    pub playground_created_at: NaiveDateTime,
    pub playground_updated_at: NaiveDateTime,
    pub playground_data: serde_json::Value,
    pub tags: serde_json::Value,
    pub maps: Vec<Option<String>>,
    pub game_sizes: Vec<Option<i32>>,
    pub modes: Vec<Option<String>>,
    pub progression_mode: serde_json::Value,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Experience {
    pub fn init_standalone(
        experience_code: ExperienceCode,
        playground: PlaygroundInfo,
    ) -> anyhow::Result<Self> {
        let p_data = playground.clone().original_playground.unwrap_or_default();
        let map_rotations = p_data.map_rotation.unwrap_or_default().maps;
        let mut maps: Vec<Option<String>> = vec![];
        let mut game_sizes: Vec<Option<i32>> = vec![];
        let mut modes: Vec<Option<String>> = vec![];
        for map_rotation in map_rotations {
            maps.push(Some(map_rotation.mapname));
            game_sizes.push(Some(map_rotation.game_size as i32));
            modes.push(Some(map_rotation.mode));
        }
        Ok(Experience {
            experience_id: experience_code.to_usize()? as i32,
            share_code: experience_code.into(),
            playground_name: p_data.playground_name,
            playground_description: p_data.playground_description,
            playground_data: serde_json::to_value(&playground)?,
            tags: serde_json::to_value(playground.clone().tag)?,
            progression_mode: serde_json::to_value(playground.clone().progression_mode)?,
            playground_created_at: chrono::DateTime::from_timestamp(
                p_data.created_at.unwrap_or_default().seconds.into(),
                p_data
                    .created_at
                    .unwrap_or_default()
                    .nanos
                    .try_into()
                    .unwrap(),
            )
            .unwrap_or_default()
            .naive_utc(),
            playground_updated_at: chrono::DateTime::from_timestamp(
                p_data.updated_at.unwrap_or_default().seconds.into(),
                p_data
                    .updated_at
                    .unwrap_or_default()
                    .nanos
                    .try_into()
                    .unwrap(),
            )
            .unwrap_or_default()
            .naive_utc(),
            maps: maps,
            game_sizes: game_sizes,
            modes: modes,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        })
    }

    // pub fn init_gametools(
    //     experience_code: ExperienceCode,
    //     playground: serde_json::Value,
    // ) -> anyhow::Result<Self> {
    //     let default_value = &serde_json::Value::String("".to_string());
    //     let p_data = playground.clone().get("originalPlayground").unwrap();
    //     Ok(Experience {
    //         experience_id: experience_code.to_usize()? as i32,
    //         share_code: experience_code.into(),
    //         playground_name: p_data
    //             .get("playgroundName")
    //             .unwrap_or()
    //             .as_str()
    //             .unwrap_or_default()
    //             .to_string(),
    //         playground_description: p_data.playground_description,
    //         playground_data: serde_json::to_value(&playground)?,
    //         tags: serde_json::to_value(playground.clone().tag)?,
    //         progression_mode: serde_json::to_value(playground.clone().progression_mode)?,
    //         playground_created_at: chrono::DateTime::from_timestamp(
    //             p_data.created_at.unwrap_or_default().seconds.into(),
    //             p_data
    //                 .created_at
    //                 .unwrap_or_default()
    //                 .nanos
    //                 .try_into()
    //                 .unwrap(),
    //         )
    //         .unwrap_or_default()
    //         .naive_utc(),
    //         playground_updated_at: chrono::DateTime::from_timestamp(
    //             p_data.updated_at.unwrap_or_default().seconds.into(),
    //             p_data
    //                 .updated_at
    //                 .unwrap_or_default()
    //                 .nanos
    //                 .try_into()
    //                 .unwrap(),
    //         )
    //         .unwrap_or_default()
    //         .naive_utc(),
    //         created_at: Utc::now().naive_utc(),
    //         updated_at: Utc::now().naive_utc(),
    //     })
    // }
}

#[derive(AsChangeset, Queryable, Selectable, Insertable)]
#[diesel(table_name = current_experiences)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CurrentExperience {
    pub id: i32,
    pub code: i32,
}
