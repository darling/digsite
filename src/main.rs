use anyhow::{anyhow, Ok, Result};
use axum::{
    http::{HeaderMap, HeaderValue},
    routing::get,
};
use reqwest::{header::AUTHORIZATION, Client};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use socketioxide::{
    extract::{Bin, Data, SocketRef},
    handler::ConnectHandler,
    SocketIo,
};
use tracing::info;
use tracing_subscriber::FmtSubscriber;

use crate::game::digsites::DigSite;

pub mod game;

fn on_connect(socket: SocketRef) {
    if let Some(query) = socket.extensions.get::<Connection>() {
        info!(
            "Socket.IO connected: {:?} {:?} {:?}",
            socket.ns(),
            socket.id,
            query.iid
        );

        // add the user to the room of their iid if possible
        let _ = socket.join(query.iid.clone());

        // TODO: Sync the user with the room's information

        socket.on(
            "message",
            |socket: SocketRef, Data::<Value>(data), Bin(bin)| {
                info!("Received event: {:?} {:?}", data, bin);
                socket.bin(bin).emit("message-back", data).ok();
            },
        );
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct DiscordUser {
    id: String,
    username: String,
    global_name: Option<String>,
    avatar: Option<String>,
}

impl DiscordUser {
    fn name(&self) -> String {
        return self.global_name.as_ref().unwrap_or(&self.username).clone();
    }
}

async fn auth_socket_middleware(s: SocketRef) -> Result<()> {
    let auth_header = s
        .req_parts()
        .headers
        .get("Authorization")
        .ok_or_else(|| anyhow!("invalid headers"))?
        .to_str()?;

    let client = Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_header)?);

    let user = client
        .get("https://discord.com/api/users/@me")
        .headers(headers)
        .send()
        .await?
        .error_for_status()?
        .json::<DiscordUser>()
        .await?;

    info!("Hello, {}!", user.name());

    s.extensions.insert(user);

    return Ok(());
}

#[derive(Debug, Serialize, Deserialize)]
struct Connection {
    iid: String,
}

fn shape_middleware(s: SocketRef) -> Result<()> {
    let qs = s
        .req_parts()
        .uri
        .query()
        .ok_or_else(|| anyhow!("uri contains invalid query string"))?;

    s.extensions.insert(serde_qs::from_str::<Connection>(qs)?);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing::subscriber::set_global_default(FmtSubscriber::default())?;

    let (layer, io) = SocketIo::new_layer();

    io.ns(
        "/",
        on_connect
            .with(auth_socket_middleware)
            .with(shape_middleware),
    );

    let app = axum::Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .layer(layer);

    info!("Starting server");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    // TODO: Remove this
    let _digsite = DigSite {};

    Ok(())
}
