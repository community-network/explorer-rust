use connectors::mysql::schema::experiences;
use diesel::prelude::*;
use grpc_rust::modules::communitygames::PlaygroundInfo;

use crate::{
    connectors::{self, mysql::schema::current_experiences},
    experience_code::ExperienceCode,
};

#[derive(AsChangeset, Queryable, Selectable, Insertable)]
#[diesel(table_name = experiences)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Experience {
    experience_id: u32,
    share_code: String,
    playground_name: String,
    playground_description: String,

    playground_data: serde_json::Value,
}

impl Experience {
    pub fn init(
        experience_code: ExperienceCode,
        playground: PlaygroundInfo,
    ) -> anyhow::Result<Self> {
        let p_data = playground.clone().original_playground.unwrap_or_default();
        Ok(Experience {
            experience_id: experience_code.to_usize()? as u32,
            share_code: experience_code.into(),
            playground_name: p_data.playground_name,
            playground_description: p_data.playground_description,
            playground_data: serde_json::to_value(&playground)?,
        })
    }
}

#[derive(AsChangeset, Queryable, Selectable, Insertable)]
#[diesel(table_name = current_experiences)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct CurrentExperience {
    pub id: i32,
    pub code: u32,
}
