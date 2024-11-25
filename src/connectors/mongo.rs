use std::env;

use bf_sparta::cookie::Cookie;
use dotenvy::dotenv;
use mongodb::error::Result;
use mongodb::{results::UpdateResult, Client, Collection};
use serde::{Deserialize, Serialize};

pub struct MongoClient {
    pub backend_cookies: Collection<BackendCookie>,
    pub client: Client,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BackendCookie {
    pub _id: String,
    pub sid: String,
    pub remid: String,
}

impl From<BackendCookie> for Cookie {
    fn from(cookie: BackendCookie) -> Self {
        Cookie {
            remid: cookie.remid,
            sid: cookie.sid,
        }
    }
}

impl MongoClient {
    pub async fn connect() -> Result<Self> {
        // Possible env
        dotenv().ok();
        let mongo_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        // Try connect to mongo client
        let client = Client::with_uri_str(mongo_url).await?;

        // Server manager DB
        let db = client.database("serverManager");

        Ok(MongoClient {
            backend_cookies: db.collection("backendCookies"),
            client,
        })
    }

    pub async fn push_new_cookies(
        &mut self,
        acc_email: &str,
        cookie: &Cookie,
    ) -> Result<UpdateResult> {
        let id = acc_email.split('@').collect::<Vec<&str>>()[0];
        let cookie = BackendCookie {
            _id: format!("main-{}", id),
            sid: cookie.sid.clone(),
            remid: cookie.remid.clone(),
        };
        self.backend_cookies
            .replace_one(bson::doc! {"_id": format!("main-{}", id)}, cookie)
            .upsert(true)
            .await
    }

    pub async fn get_cookies(&mut self, acc_email: &str) -> anyhow::Result<Cookie> {
        let backend_cookie = match self.backend_cookies.find_one(bson::doc! {"_id": format!("main-{}", acc_email.split('@').collect::<Vec<&str>>()[0])}).await? {
            Some(result) => result,
            None => anyhow::bail!("no cookie"),
        };
        Ok(backend_cookie.into())
    }
}
