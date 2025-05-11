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
        let bf2042_account =
            env::var("API_BF2042_ACCOUNT").expect("API_BF2042_ACCOUNT must be set");

        let (bf2042_cookie, ea_access_token) = match mongo_client.get_cookies(&bf2042_account).await
        {
            Ok(result) => result,
            Err(e) => {
                log::warn!("Cookie failed, {}", e);
                (
                    bf_sparta::cookie::Cookie {
                        sid: "".to_string(),
                        remid: "".to_string(),
                    },
                    "".to_string(),
                )
            }
        };

        let session_id = match self.kingston_client.clone() {
            Some(client) => client.session_id,
            None => "".to_string(),
        };

        let mut kingston_client = KingstonClient::new(session_id.clone()).await?;
        match kingston_client
            .ea_desktop_auth(bf2042_cookie, ea_access_token)
            .await
        {
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
