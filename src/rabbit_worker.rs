use std::{
    sync::{atomic, Arc},
    time::Duration,
};

use anyhow::Result;

mod clients;
mod connectors;
mod experience_code;

use clients::standalone_client::StandaloneClient;
use connectors::{
    mongo::lib::MongoClient,
    postgres::{lib::PostgresClient, models::Experience},
};
use experience_code::ExperienceCode;
use tokio::{runtime::Runtime, time::sleep};

use chrono::Utc;
use warp::Filter;

use std::sync::atomic::AtomicI64;

use lapin::Channel;
use uuid::Uuid;

use crate::connectors::ampq;
use futures::stream::StreamExt;

pub(crate) struct FunctionWorker {
    pub client: StandaloneClient,
    pub db_client: PostgresClient,
    /// Rabbit MQ channel connection
    pub rabbit: Channel,
    /// Mongo Client connection
    pub mongo: MongoClient,
    /// Uniq Worker ID
    pub uuid: String,
    // last group checkup
    last_update: Arc<AtomicI64>,
}

impl FunctionWorker {
    /**
        Create a new Fortress Node
        Returns Self
    */
    pub async fn new(last_update: Arc<AtomicI64>) -> Result<Self> {
        Ok(Self {
            client: StandaloneClient {
                kingston_client: None,
            },
            db_client: PostgresClient::connect()?,
            mongo: MongoClient::connect().await?,
            rabbit: ampq::create_channel().await?,
            uuid: Uuid::new_v4().to_string(),
            last_update,
        })
    }

    /// Init all needed ques
    async fn init_ques(&self) -> Result<()> {
        let ques = ["experience_code-v1", "workerstatuscollector-v4"];

        for que in ques.iter() {
            ampq::declare_que(&self.rabbit, que).await?;
        }

        ampq::declare_que_worker(&self.rabbit, &self.uuid).await?;

        Ok(())
    }

    /// Run Fortress node
    pub async fn run(&mut self) -> Result<()> {
        self.client.connect(self.mongo.clone()).await?;

        log::info!("Emitting Ques");
        self.init_ques().await?;

        log::info!("Running {} fortress node", &self.uuid);
        self.run_loop().await?;

        Ok(())
    }

    async fn run_loop(&mut self) -> Result<()> {
        ampq::publish(
            &self.rabbit,
            "experience_workerstatuscollector-v1",
            "startup;ok".to_string(),
        )
        .await?;

        ampq::set_qos(&self.rabbit).await?;
        let consumer = ampq::new_consumer(&self.rabbit, "experience_code-v1").await?;
        let mut iter = consumer;

        // For each new group
        while let Some(next_task) = iter.next().await {
            // Get delivery
            let delivery = next_task?;

            // Get the group id, where we will use cookies
            let current_experience: String = String::from_utf8_lossy(&delivery.data).into();
            let e_code = ExperienceCode::from_i32(current_experience.parse::<i32>()?)?;
            log::info!("Assigned {}", Into::<String>::into(e_code.clone()));

            let result = self.check_experience(&e_code).await;

            // Ack delivery
            delivery
                .ack(lapin::options::BasicAckOptions::default())
                .await?;

            let response = match result {
                Ok(_) => "okay",
                Err(e) => {
                    log::error!("{} failed: {:#?}", current_experience, e);
                    "error"
                }
            };

            // Send a response
            ampq::publish(
                &self.rabbit,
                "experience_workerstatuscollector-v1",
                format!("{};{}", current_experience, response),
            )
            .await?;

            // healthcheck
            let current_timestamp_minutes =
                Utc::now().timestamp().checked_div(60).unwrap_or_default();
            self.last_update
                .store(current_timestamp_minutes, atomic::Ordering::Relaxed);

            // don't go to fast, otherwise you will get temporarily blocked.
            sleep(Duration::from_secs(5)).await;
        }

        Ok(())
    }

    async fn check_experience(&mut self, e_code: &ExperienceCode) -> Result<()> {
        let res = self.client.get_playground(&e_code).await?;

        if let Some(playground) = res.playground {
            log::info!(
                "gathered experience: {}",
                playground
                    .clone()
                    .original_playground
                    .unwrap()
                    .playground_name
            );
            self.db_client
                .add_or_update_experience(Experience::init_standalone(e_code.clone(), playground)?)
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    match dotenvy::dotenv() {
        Ok(_) => {}
        Err(_) => log::info!(".env not found, using env variables..."),
    };

    let last_update = Arc::new(atomic::AtomicI64::new(0));
    let last_update_clone = Arc::clone(&last_update);

    flexi_logger::Logger::try_with_str("info")
        .unwrap()
        .start()
        .unwrap();

    // Use Info, Debug, Trace to see more
    // log::set_max_level(log::LevelFilter::Warn);
    log::info!("Startup...");

    // Create the runtime
    let rt = Runtime::new().unwrap();

    // healthcheck
    rt.spawn(async move {
        let hello = warp::any().map(move || {
            let last_update_i64 = last_update_clone.load(atomic::Ordering::Relaxed);
            let now_minutes = Utc::now().timestamp().checked_div(60).unwrap_or_default();

            // error if 10 minutes without traffic
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

    // For multigame we can potentially pass game param in here
    let mut fortress = rt.block_on(FunctionWorker::new(last_update))?;

    // Run infinity loop
    rt.block_on(fortress.run())?;

    Ok(())
}
