use clap::{app_from_crate, Arg};
use scopeguard::guard;
use tokio::io::AsyncReadExt;
use tokio_tun::result::Result;
use tokio_tun::TunBuilder;
use tracing::span;
use tracing::trace;
use tracing::Level;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

const OPTION_DEVICE_NAME: &str = "dev";

#[tokio::main]
async fn main() -> Result<()> {
    // Setup logger
    let formatting_layer = BunyanFormattingLayer::new("werson".into(), std::io::stdout);
    let subscriber = Registry::default()
        .with(JsonStorageLayer)
        .with(formatting_layer);
    tracing::subscriber::set_global_default(subscriber).unwrap();

    // Setup CLI
    let matches = app_from_crate!()
        .arg(
            Arg::new(OPTION_DEVICE_NAME)
                .short('d')
                .default_value("werson0")
                .about("Name of the network device to create"),
        )
        .get_matches();

    // Create TAP device
    let device_name = matches.value_of(OPTION_DEVICE_NAME).unwrap();
    let span = span!(
        Level::INFO,
        "TAP device creation",
        device_name = device_name
    );
    let enter = span.enter();

    let tap_device = TunBuilder::new()
        .name(device_name)
        .tap(true)
        .packet_info(false)
        .up()
        .try_build()?;

    drop(enter);

    // Start reading from TAP device
    let span = span!(Level::INFO, "TAP device read loop");
    let enter = span.enter();
    guard((), |_| drop(enter));

    let (mut reader, mut _writer) = tokio::io::split(tap_device);

    let mut buf = [0u8; 1024];
    loop {
        let n = reader.read(&mut buf).await?;

        trace!("Read {} bytes: {:?}", n, &buf[..n]);
    }
}
