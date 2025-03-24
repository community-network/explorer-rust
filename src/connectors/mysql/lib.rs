use std::env;

use crate::connectors::mysql::schema::current_experiences::dsl::*;
use crate::connectors::mysql::schema::experiences::dsl::*;
use anyhow::anyhow;
use diesel::{associations::HasTable, prelude::*};
use dotenvy::dotenv;

use super::models::{CurrentExperience, Experience};

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

    pub fn current_experience(&mut self) -> anyhow::Result<u32> {
        let current_id: Option<u32> = current_experiences::table()
            .select(code)
            .first(&mut self.client)
            .optional()?;

        if let Some(e) = current_id {
            return Ok(e);
        }
        diesel::insert_into(current_experiences::table())
            .values(CurrentExperience { id: 1, code: 0 })
            .execute(&mut self.client);
        return Ok(0);
    }

    pub fn set_current_experience(&mut self, current_id: u32) {
        let _ = diesel::replace_into(current_experiences::table())
            .values(&CurrentExperience {
                id: 1,
                code: current_id,
            })
            .execute(&mut self.client)
            .expect("Error saving current experience id");
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
