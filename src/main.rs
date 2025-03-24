mod connectors;
mod experience_code;

use std::{env, time::Duration};

use bf_sparta::{cookie_request, sparta_api};
use connectors::{
    mongo::lib::MongoClient,
    mysql::{lib::MysqlClient, models::Experience},
};
use dotenvy::dotenv;
use experience_code::ExperienceCode;
use grpc_rust::{grpc::KingstonClient, modules::CommunityGames};
use std::sync::{atomic, Arc};
use tokio::time::sleep;
use warp::Filter;

async fn connect(
    mut mongo_client: MongoClient,
    old_session: String,
) -> anyhow::Result<KingstonClient> {
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

    let mut kingston_client = KingstonClient::new(old_session).await?;
    match kingston_client.auth(cookie.clone()).await {
        Ok(_) => {}
        Err(e) => anyhow::bail!("kingston session failed: {:#?}", e),
    };

    Ok(kingston_client)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    flexi_logger::Logger::try_with_str("info")?.start()?;
    log::info!("Starting...");

    let last_update = Arc::new(atomic::AtomicI64::new(chrono::Utc::now().timestamp() / 60));
    let last_update_clone = Arc::clone(&last_update);

    tokio::spawn(async move {
        let hello = warp::any().map(move || {
            let last_update_i64 = last_update_clone.load(atomic::Ordering::Relaxed);
            let now_minutes = chrono::Utc::now().timestamp() / 60;

            // error if 10 minutes without updates
            if (now_minutes - last_update_i64) > 10 {
                warp::reply::with_status(
                    format!("{}", now_minutes - last_update_i64),
                    warp::http::StatusCode::SERVICE_UNAVAILABLE,
                )
            } else {
                warp::reply::with_status(
                    format!("{}", now_minutes - last_update_i64),
                    warp::http::StatusCode::OK,
                )
            }
        });
        warp::serve(hello).run(([0, 0, 0, 0], 3030)).await;
    });

    let mut client = MysqlClient::connect()?;

    let mongo_client = MongoClient::connect().await?;
    let mut session_id = "".to_string();
    let kingston_client = connect(mongo_client, session_id).await?;
    session_id = kingston_client.session_id.clone();
    let mut current_experience = client.current_experience()?;

    loop {
        let e_code = ExperienceCode::from_u32(current_experience)?;

        let res = CommunityGames::get_shared_playground_v2(&kingston_client, e_code.clone().into())
            .await?;

        if let Some(playground) = res.playground {
            println!(
                "{}",
                playground
                    .clone()
                    .original_playground
                    .unwrap()
                    .playground_name
            );
            client.add_or_update_experience(Experience::init(e_code, playground)?)
        }

        // don't go to fast, otherwise you will get temporarily blocked.
        sleep(Duration::from_secs(5)).await;

        last_update.store(
            chrono::Utc::now().timestamp() / 60,
            atomic::Ordering::Relaxed,
        );
        current_experience += 1;
        client.set_current_experience(current_experience);
    }
}
