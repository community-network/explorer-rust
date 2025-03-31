use std::env;

use crate::connectors::postgres::schema::current_experiences::dsl::*;
use crate::connectors::postgres::schema::experiences::dsl::*;
use diesel::{associations::HasTable, prelude::*, upsert::excluded};
use dotenvy::dotenv;

use super::models::{CurrentExperience, Experience};

pub struct PostgresClient {
    pub client: PgConnection,
}

impl PostgresClient {
    pub fn connect() -> anyhow::Result<Self> {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let connection = PgConnection::establish(&database_url).unwrap();

        Ok(PostgresClient { client: connection })
    }

    pub fn current_experience(&mut self) -> anyhow::Result<i32> {
        let current_id: Option<i32> = current_experiences::table()
            .select(code)
            .first(&mut self.client)
            .optional()?;

        if let Some(e) = current_id {
            return Ok(e);
        }
        let _ = diesel::insert_into(current_experiences::table())
            .values(CurrentExperience { id: 1, code: 1 })
            .execute(&mut self.client);
        return Ok(1);
    }

    pub fn set_current_experience(&mut self, current_id: i32) {
        let value = &CurrentExperience {
            id: 1,
            code: current_id,
        };
        let _ = diesel::insert_into(current_experiences::table())
            .values(value)
            .on_conflict(id)
            .do_update()
            .set(value)
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
        let _ = diesel::insert_into(experiences::table())
            .values(&experience)
            .on_conflict(experience_id)
            .do_update()
            .set((
                experience_id.eq(&experience.experience_id),
                share_code.eq(&experience.share_code),
                playground_name.eq(&experience.playground_name),
                playground_description.eq(&experience.playground_description),
                playground_created_at.eq(&experience.playground_created_at),
                playground_updated_at.eq(&experience.playground_updated_at),
                playground_data.eq(&experience.playground_data),
                tags.eq(&experience.tags),
                progression_mode.eq(&experience.progression_mode),
                updated_at.eq(&experience.updated_at),
                created_at.eq(excluded(created_at)),
            ))
            .execute(&mut self.client)
            .expect("Error saving new post");
    }

    pub fn update_experience(&mut self, experience: Experience) {
        let _ = diesel::update(experiences::table())
            .set(experience)
            .execute(&mut self.client)
            .expect("Error updating post");
    }

    pub fn get_last_experience(&mut self) -> anyhow::Result<i32> {
        let experience: Option<i32> = experiences::table()
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
