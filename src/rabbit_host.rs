use std::sync::{
    atomic::{self, AtomicI64},
    Arc,
};

use anyhow::Result;
use chrono::Utc;
use connectors::postgres::lib::PostgresClient;
use futures::StreamExt;
use lapin::Channel;
use tokio::runtime::Runtime;
use warp::Filter;

mod connectors;
mod experience_code;

use crate::connectors::ampq;

pub(crate) struct FunctionMaster {
    pub client: PostgresClient,
    pub rabbit: Channel,
    // last group checkup
    pub last_update: Arc<AtomicI64>,
}

impl FunctionMaster {
    pub async fn new(last_update: Arc<AtomicI64>) -> Result<Self> {
        Ok(Self {
            client: PostgresClient::connect()?,
            rabbit: ampq::create_channel().await?,
            last_update,
        })
    }

    /// Init all needed ques
    async fn init_ques(&self) -> Result<()> {
        let ques = ["experience_code-v1", "experience_workerstatuscollector-v1"];

        for que in ques.iter() {
            ampq::delete_que(&self.rabbit, que).await?;
            ampq::declare_que(&self.rabbit, que).await?;
        }

        Ok(())
    }

    /// Run Fortress node
    pub async fn run(&mut self) -> Result<()> {
        loop {
            log::info!("Started!");

            self.init_ques().await?;

            self.run_loop().await?;
        }
    }

    async fn run_loop(&mut self) -> Result<()> {
        let mut current_experience = self.client.current_experience()?;
        match ampq::publish(
            &self.rabbit,
            "experience_code-v1",
            current_experience.to_string(),
        )
        .await
        {
            Ok(_) => {}
            Err(_) => log::error!("couldn't make queue for {}", &current_experience),
        };

        log::info!("Sent initial item");

        ampq::set_qos(&self.rabbit).await?;

        let consumer =
            ampq::new_consumer(&self.rabbit, "experience_workerstatuscollector-v1").await?;
        let mut iter = consumer;

        while let Some(next_task) = iter.next().await {
            // Get delivery
            let delivery = next_task?;

            let delivery_str = String::from_utf8_lossy(&delivery.data).to_string();
            let result: &Vec<&str> = &delivery_str.split(';').collect::<Vec<&str>>();
            let experience_id = &result[0].to_string();

            // result[1]

            // Ack delivery
            delivery
                .ack(lapin::options::BasicAckOptions::default())
                .await?;
            log::info!("Finished {} status {}", experience_id, result[1]);

            if result[1] != "error" {
                current_experience += 1;
                self.client
                    .set_current_experience(current_experience.clone());
            }
            match ampq::publish(
                &self.rabbit,
                "experience_code-v1",
                current_experience.to_string(),
            )
            .await
            {
                Ok(_) => {}
                Err(_) => log::error!("couldn't make queue for {}", &current_experience),
            };

            // healthcheck
            let current_timestamp_minutes =
                Utc::now().timestamp().checked_div(60).unwrap_or_default();
            self.last_update
                .store(current_timestamp_minutes, atomic::Ordering::Relaxed);
        }

        self.rabbit.close(200, "Normal shutdown").await?;
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
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
    let mut fortress = rt.block_on(FunctionMaster::new(last_update))?;

    // Run infinity loop
    rt.block_on(fortress.run())?;

    Ok(())
}
