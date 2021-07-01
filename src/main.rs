use clap::{app_from_crate, Arg};
use tokio_tun::TunBuilder;
use tracing::{info, span, Level};
use tracing_bunyan_formatter::BunyanFormattingLayer;
use tracing_bunyan_formatter::JsonStorageLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

const OPTION_DEVICE_NAME: &str = "dev";

#[tokio::main]
async fn main() -> Result<(), ()> {
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
    let tap_device;
    let span = span!(Level::INFO, "TAP device creation");
    let enter = span.enter();

    info!(
        "Creating TAP device with name `{}`",
        matches.value_of(OPTION_DEVICE_NAME).unwrap()
    );

    tap_device = TunBuilder::new()
        .name(matches.value_of(OPTION_DEVICE_NAME).unwrap())
        .tap(true)
        .packet_info(false)
        .up()
        .try_build();

    drop(enter);

    loop {}

    // let span = span!(Level::INFO, "TAP device read loop");
    // let enter = span.enter();

    // let reader = tap_device.reader();

    // drop(enter);
}
