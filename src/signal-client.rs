use clap::app_from_crate;
use clap::Arg;
use futures_util::{sink::SinkExt, stream::StreamExt};
use scopeguard::guard;
use std::error::Error as StdError;
use tracing::span;
use tracing::Level;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

const OPTION_DEVICE_REMOTE_ADDRESS: &str = "raddr";

#[actix_web::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    // Setup logger
    let formatting_layer = BunyanFormattingLayer::new("werson-signal".into(), std::io::stdout);
    let subscriber = Registry::default()
        .with(JsonStorageLayer)
        .with(formatting_layer);
    tracing::subscriber::set_global_default(subscriber).unwrap();

    // Setup CLI
    let matches = app_from_crate!()
        .arg(
            Arg::new(OPTION_DEVICE_REMOTE_ADDRESS)
                .short('r')
                .default_value("ws://127.0.0.1:15325")
                .about("Address to connect to"),
        )
        .get_matches();

    // Connect to the signaler
    let remote_address = matches.value_of(OPTION_DEVICE_REMOTE_ADDRESS).unwrap();
    let span = span!(
        Level::INFO,
        "Connecting to",
        remote_address = remote_address
    );
    let enter = span.enter();
    guard((), |_| drop(enter));

    let (_resp, mut connection) = awc::Client::new()
        .ws(matches.value_of(OPTION_DEVICE_REMOTE_ADDRESS).unwrap())
        .connect()
        .await?;

    connection
        .send(awc::ws::Message::Text("Hello, world!".to_string()))
        .await?;
    let response = connection.next().await.unwrap()?;

    println!("{:?}", response);

    Ok(())
}
