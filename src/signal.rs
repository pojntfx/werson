use actix::{Actor, StreamHandler};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use clap::app_from_crate;
use clap::Arg;
use scopeguard::guard;
use tracing::span;
use tracing::Level;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

const OPTION_DEVICE_LISTEN_ADDRESS: &str = "laddr";

struct Signaler;

impl Actor for Signaler {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Signaler {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => span!(Level::TRACE, "Handling ping").in_scope(|| {
                ctx.pong(&msg);
            }),
            Ok(ws::Message::Text(text)) => span!(Level::INFO, "Handling text").in_scope(|| {
                ctx.text(text);
            }),
            Ok(ws::Message::Binary(bin)) => span!(Level::INFO, "Handling binary").in_scope(|| {
                ctx.binary(bin);
            }),
            _ => (),
        }
    }
}

async fn index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    span!(Level::TRACE, "Handling connection").in_scope(|| ws::start(Signaler {}, &req, stream))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Setup logger
    let formatting_layer = BunyanFormattingLayer::new("werson-signal".into(), std::io::stdout);
    let subscriber = Registry::default()
        .with(JsonStorageLayer)
        .with(formatting_layer);
    tracing::subscriber::set_global_default(subscriber).unwrap();

    // Setup CLI
    let matches = app_from_crate!()
        .arg(
            Arg::new(OPTION_DEVICE_LISTEN_ADDRESS)
                .short('l')
                .default_value("127.0.0.1:15325")
                .about("Address to listen on"),
        )
        .get_matches();

    // Start listening
    let listen_address = matches.value_of(OPTION_DEVICE_LISTEN_ADDRESS).unwrap();
    let span = span!(Level::INFO, "Listening", listen_address = listen_address);
    let enter = span.enter();
    guard((), |_| drop(enter));

    HttpServer::new(|| App::new().route("/", web::get().to(index)))
        .bind(listen_address)?
        .run()
        .await
}
