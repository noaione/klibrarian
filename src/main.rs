use std::sync::Arc;

use axum::{
    Router,
    extract::State,
    http::Uri,
    response::{Html, IntoResponse, Redirect},
    routing::get,
};
use komga::KomgaClient;
use tokio::net::TcpListener;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use urlencoding::encode;

include!(concat!(env!("OUT_DIR"), "/index_html.rs"));

mod komga;
mod routes;

#[derive(Clone)]
pub struct AppState {
    pub redis: Arc<redis::Client>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "k_librarian=debug,tower_http=debug,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let version = env!("CARGO_PKG_VERSION");

    tracing::info!("📚 K-Librarian v{}", version);
    dotenv::dotenv().ok();

    match std::env::var("TOKEN") {
        Ok(_) => {}
        Err(_) => {
            tracing::error!("💥 `TOKEN` environment variable is not set!");
            tracing::error!("💥 Please set it as it's used as your login token to the dashboard");
            std::process::exit(1);
        }
    }

    let komga_client = KomgaClient::instance();

    let redis_host = match std::env::var("REDIS_HOST") {
        Ok(host) => host,
        Err(_) => {
            tracing::error!("💥 `REDIS_HOST` environment variable is not set!");
            tracing::error!("    Please set it as it's used to connect to Redis");
            std::process::exit(1);
        }
    };
    let redis_port = match std::env::var("REDIS_PORT") {
        Ok(port) => port,
        Err(_) => {
            tracing::error!("💥 `REDIS_PORT` environment variable is not set!");
            tracing::error!("    Please set it as it's used to connect to Redis");
            std::process::exit(1);
        }
    };
    let redis_pass = std::env::var("REDIS_PASS").unwrap_or("".to_string());

    let mut redis_url = format!("redis://{redis_host}:{redis_port}");
    if !redis_pass.is_empty() {
        redis_url = format!("redis://:{redis_pass}@{redis_host}:{redis_port}");
    }

    tracing::info!("🔌 Connecting to Redis at: {}", redis_url);
    let redis_client = redis::Client::open(redis_url).unwrap();

    // Test Redis connection
    match redis_client.get_multiplexed_async_connection().await {
        Ok(_) => {
            tracing::info!("  ✨ Connected to Redis");
        }
        Err(e) => {
            tracing::error!("  💥 Failed to connect to Redis: {}", e);
            std::process::exit(1);
        }
    }

    tracing::info!("🔌 Connecting to Komga at: {}", komga_client.get_host());
    match komga_client.get_me().await {
        Ok(user) => {
            // Check if ADMIN role
            if !user.roles.contains(&"ADMIN".to_string()) {
                tracing::error!(
                    "  😔 Provided Komga user is not an ADMIN, please use an account with admin privilege!"
                );
                std::process::exit(1);
            }
            tracing::info!("  ✨ Connected to Komga");
        }
        Err(e) => {
            tracing::error!("  💥 Failed to connect to Komga: {}", e);
            std::process::exit(1);
        }
    };

    let state = AppState {
        redis: Arc::new(redis_client),
    };

    let assets_dir = ServeDir::new("assets/assets");

    let app: Router = Router::new()
        .route("/", get(index))
        .route(
            "/favicon.ico",
            get(|_: State<AppState>| async { include_bytes!("../assets/favicon.ico").to_vec() }),
        )
        .route("/_/health", get(|| async { "ok" }))
        .nest("/api", routes::api(state.clone()))
        .nest_service("/assets", assets_dir)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::new().allow_origin(Any))
        .with_state(state);

    let app = app.fallback(handle_404);

    let host_at = std::env::var("HOST").unwrap_or("127.0.0.1".to_string());
    let port_at = std::env::var("PORT").unwrap_or("5148".to_string());

    // run it
    let listener = TcpListener::bind(format!("{host_at}:{port_at}"))
        .await
        .unwrap();
    tracing::info!(
        "🚀 Fast serving at: http://{}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app).await.unwrap()
}

async fn handle_404(url: Uri) -> Redirect {
    let path = url.to_string();
    tracing::info!("404: {:?}", url);

    let redirect_url = format!("/?redirect={}", encode(&path));
    Redirect::to(&redirect_url)
}

async fn index(_: State<AppState>) -> impl IntoResponse {
    Html(INDEX_HTML)
}
