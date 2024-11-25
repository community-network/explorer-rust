use self::structs::mysql_schema::experiences::dsl::*;
use anyhow::anyhow;
use diesel::{associations::HasTable, prelude::*};
use dotenvy::dotenv;
use grpc_rust::modules::communitygames::PlaygroundInfo;
use std::env;

use crate::{
    experience_code::ExperienceCode,
    structs::{self, mysql_schema::experiences::share_code},
};

#[derive(AsChangeset, Queryable, Selectable, Insertable)]
#[diesel(table_name = structs::mysql_schema::experiences)]
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

pub struct MysqlClient {
    pub client: MysqlConnection,
}

impl MysqlClient {
    pub fn connect() -> anyhow::Result<Self> {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let connection = MysqlConnection::establish(&database_url).unwrap();

        Ok(MysqlClient { client: connection })
    }

    pub fn has_experience(&mut self, _share_code: String) -> bool {
        let experience = experiences::table()
            .filter(share_code.eq(_share_code))
            .select(Experience::as_select())
            .first(&mut self.client)
            .optional();

        match experience {
            Ok(Some(_)) => true,
            Ok(None) => false,
            Err(_) => false,
        }
    }

    pub fn add_or_update_experience(&mut self, experience: Experience) {
        let _ = diesel::replace_into(experiences::table())
            .values(&experience)
            .execute(&mut self.client)
            .expect("Error saving new post");
    }

    pub fn update_experience(&mut self, experience: Experience) {
        let _ = diesel::update(experiences::table())
            .set(experience)
            .execute(&mut self.client)
            .expect("Error updating post");
    }

    pub fn get_last_experience(&mut self) -> anyhow::Result<u32> {
        let experience: Option<u32> = experiences::table()
            .select(experience_id)
            .order_by(experience_id.desc())
            .first(&mut self.client)
            .optional()?;

        if let Some(e) = experience {
            return Ok(e);
        }
        anyhow::bail!("Database is empty!")
    }
}
