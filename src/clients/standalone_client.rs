use crate::{connectors::mongo::lib::MongoClient, experience_code::ExperienceCode};
use bf_sparta::{cookie_request, sparta_api};
use dotenvy::dotenv;
use grpc_rust::{
    grpc::KingstonClient,
    modules::{communitygames::PlaygroundInfoResponse, CommunityGames},
};
use std::env;

pub struct StandaloneClient {
    pub kingston_client: Option<KingstonClient>,
}

impl StandaloneClient {
    pub async fn connect(&mut self, mut mongo_client: MongoClient) -> anyhow::Result<()> {
        dotenv().ok();
        let main_account = env::var("API_MAIN_ACCOUNT").expect("API_MAIN_ACCOUNT must be set");
        let password =
            env::var("API_MAIN_ACCOUNT_PASSWORD").expect("API_MAIN_ACCOUNT_PASSWORD must be set");

        let mut cookie = match mongo_client.get_cookies(&main_account).await {
            Ok(result) => result,
            Err(_) => bf_sparta::cookie::Cookie {
                sid: "".to_string(),
                remid: "".to_string(),
            },
        };

        cookie = match sparta_api::get_token(cookie.clone(), "pc", "tunguska", "en-us").await {
            Ok(_) => cookie.clone(),
            Err(e) => {
                log::warn!("Cookie failed, {} - requesting new cookie", e);
                let cookie_auth = cookie_request::request_cookie(cookie_request::Login {
                    email: main_account.clone(),
                    pass: password,
                })
                .await?;
                let cookie = bf_sparta::cookie::Cookie {
                    sid: cookie_auth.sid,
                    remid: cookie_auth.remid,
                };
                mongo_client
                    .push_new_cookies(&main_account, &cookie)
                    .await?;
                cookie
            }
        };

        let session_id = match self.kingston_client.clone() {
            Some(client) => client.session_id,
            None => "".to_string(),
        };

        let mut kingston_client = KingstonClient::new(session_id.clone()).await?;
        match kingston_client.auth(cookie.clone()).await {
            Ok(_) => {}
            Err(e) => anyhow::bail!("kingston session failed: {:#?}", e),
        };

        self.kingston_client = Some(kingston_client);
        Ok(())
    }

    pub async fn get_playground(
        &self,
        e_code: &ExperienceCode,
    ) -> Result<PlaygroundInfoResponse, anyhow::Error> {
        match &self.kingston_client {
            Some(kingston_client) => {
                CommunityGames::get_shared_playground_v2(kingston_client, e_code.clone().into())
                    .await
            }
            None => anyhow::bail!("no kingston client available"),
        }
    }
}
