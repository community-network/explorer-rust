mod clients;
mod connectors;
mod experience_code;

use std::time::Duration;

use clients::standalone_client::StandaloneClient;
use connectors::{
    mongo::lib::MongoClient,
    postgres::{lib::PostgresClient, models::Experience},
};
use experience_code::ExperienceCode;
use std::sync::{atomic, Arc};
use tokio::time::sleep;
use warp::Filter;

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

    let mut client = PostgresClient::connect()?;

    let mongo_client = MongoClient::connect().await?;

    let mut standalone_client = StandaloneClient {
        kingston_client: None,
    };
    standalone_client.connect(mongo_client).await?;
    let mut current_experience = client.current_experience()?;

    loop {
        let e_code = ExperienceCode::from_i32(current_experience)?;

        let res = standalone_client.get_playground(&e_code).await?;

        if let Some(playground) = res.playground {
            println!(
                "{}",
                playground
                    .clone()
                    .original_playground
                    .unwrap()
                    .playground_name
            );
            client.add_or_update_experience(Experience::init_standalone(e_code, playground)?)
        }

        // don't go to fast, otherwise you will get temporarily blocked.
        sleep(Duration::from_secs(3)).await;

        last_update.store(
            chrono::Utc::now().timestamp() / 60,
            atomic::Ordering::Relaxed,
        );
        current_experience += 1;
        client.set_current_experience(current_experience);
    }
}
