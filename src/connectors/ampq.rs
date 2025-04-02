use std::env;

use base64::{prelude::BASE64_STANDARD, Engine};
use lapin::{
    options::*,
    tcp::{OwnedIdentity, OwnedTLSConfig},
    types::FieldTable,
    BasicProperties, Channel, Connection, ConnectionProperties, Consumer, Result,
};

async fn get_tls_config() -> OwnedTLSConfig {
    let client_cert_and_key = env::var("AMQPS_CERT_CLIENT").expect("AMQPS_CERT_CLIENT wasn't set");

    OwnedTLSConfig {
        identity: Some(OwnedIdentity {
            der: BASE64_STANDARD.decode(client_cert_and_key).unwrap(),
            password: env::var("AMQPS_CERT_PASS").expect("AMQPS_CERT_PASS wasn't set"),
        }),
        cert_chain: Some(env::var("AMQPS_CERT").expect("AMQPS_CERT wasn't set")),
    }
}

/// Function to create ampq channel
pub async fn create_channel() -> Result<Channel> {
    let addr: String = env::var("AMQPS_STRING").expect("AMQPS_STRING wasn't set");
    let conn = Connection::connect_with_config(
        &addr,
        ConnectionProperties::default(),
        get_tls_config().await,
    )
    .await?;
    conn.create_channel().await
}

pub async fn declare_que_worker(channel: &Channel, name: &str) -> Result<()> {
    channel
        .queue_declare(
            name,
            lapin::options::QueueDeclareOptions {
                auto_delete: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;
    Ok(())
}

pub async fn declare_que(channel: &Channel, name: &str) -> Result<()> {
    let mut args = FieldTable::default();
    args.insert("x-max-length".into(), 1000.into());
    channel
        .queue_declare(name, QueueDeclareOptions::default(), args)
        .await?;
    Ok(())
}

pub async fn set_qos(channel: &Channel) -> Result<()> {
    channel
        .basic_qos(1, lapin::options::BasicQosOptions::default())
        .await?;
    Ok(())
}

pub async fn delete_que(channel: &Channel, name: &str) -> Result<()> {
    let options = QueueDeleteOptions::default();
    channel.queue_delete(name, options).await?;
    Ok(())
}

pub async fn new_consumer(channel: &Channel, name: &str) -> Result<Consumer> {
    channel
        .basic_consume(
            name,
            "",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
}

pub async fn publish(channel: &Channel, que: &str, value: String) -> Result<()> {
    let options = BasicProperties::default().with_delivery_mode(2);
    channel
        .basic_publish(
            "",
            que,
            BasicPublishOptions::default(),
            &Vec::from(value.as_bytes()),
            options,
        )
        .await?;
    Ok(())
}
