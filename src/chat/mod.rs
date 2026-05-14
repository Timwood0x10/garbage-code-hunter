pub mod config;
pub mod persona;
mod ws;

use axum::extract::ws::WebSocketUpgrade;
use axum::response::Html;
use axum::routing::get;
use axum::Router;

pub async fn run_chat_server(port: u16) {
    let addr = format!("127.0.0.1:{}", port);
    let app = Router::new()
        .route("/", get(serve_html))
        .route("/ws", get(ws_handler));

    tracing::info!("Chat server starting on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");
    axum::serve(listener, app).await.expect("Server error");
}

async fn ws_handler(ws: WebSocketUpgrade) -> axum::response::Response {
    ws.on_upgrade(|socket| async move {
        ws::handle_socket(socket).await;
    })
}

async fn serve_html() -> Html<&'static str> {
    Html(include_str!("chat.html"))
}
