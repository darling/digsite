use socketioxide::handler::ConnectHandler;

use anyhow::{anyhow, Ok, Result};
use axum::{
    http::{HeaderMap, HeaderValue},
    routing::get,
};
use digsite::websocket::{
    lifecycle::on_connect,
    state::{Connection, ConnectionQueryString, DiscordUser, Parties},
};
use reqwest::{header::AUTHORIZATION, Client};
use socketioxide::{extract::SocketRef, SocketIo};
use tracing::info;
use tracing_subscriber::FmtSubscriber;

async fn auth_socket_middleware(s: SocketRef) -> Result<()> {
    let qs = s
        .req_parts()
        .uri
        .query()
        .ok_or_else(|| anyhow!("uri contains invalid query string"))?;

    let cqs = serde_qs::from_str::<ConnectionQueryString>(qs)?;

    let client = Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&cqs.bearer_token())?);

    let user = client
        .get("https://discord.com/api/users/@me")
        .headers(headers)
        .send()
        .await?
        .error_for_status()?
        .json::<DiscordUser>()
        .await?;

    info!("Hello, {}!", user.name());

    s.extensions.insert(Connection::new(cqs, user));

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing::subscriber::set_global_default(FmtSubscriber::default())?;

    let (layer, io) = SocketIo::builder()
        .with_state::<Parties>(Parties::new())
        .build_layer();

    io.ns("/", on_connect.with(auth_socket_middleware));
    io.ns("/api/backend", on_connect.with(auth_socket_middleware));

    let app = axum::Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .layer(layer);

    let port = std::env::var("PORT").unwrap_or("3000".to_string());

    let listener = tokio::net::TcpListener::bind(String::from("0.0.0.0:") + &port)
        .await
        .unwrap();

    info!("Starting server on port 0.0.0.0:{}", port);

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
