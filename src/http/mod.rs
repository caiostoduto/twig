use axum::{
    Router,
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

mod discord;

#[derive(Clone)]
pub struct AppState {
    pub data: Arc<crate::Data>,
}

async fn catch_all() -> Response {
    Redirect::to("https://github.com/caiostoduto/twig").into_response()
}

pub async fn start_http_server(
    data: Arc<crate::Data>,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState { data };

    let app = Router::new()
        .route("/discord/callback", get(discord::oauth_callback))
        .fallback(catch_all)
        .with_state(state);

    info!("[HTTP] Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
