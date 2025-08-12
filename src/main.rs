use std::sync::Arc;

use axum::{
    Router,
    extract::State,
    http::Uri,
    response::{Html, IntoResponse, Redirect},
    routing::get,
};
use komga::KomgaClient;
use tokio::{net::TcpListener, sync::Mutex};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use urlencoding::encode;

include!(concat!(env!("OUT_DIR"), "/index_html.rs"));

mod config;
mod database;
mod invitee;
mod komga;
mod navidrome;
mod routes;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<database::LocalDatabase>,
    pub config: Arc<config::Config>,
    pub komga: Arc<KomgaClient>,
    pub navidrome: Option<Arc<Mutex<navidrome::NavidromeClient>>>,
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
    let config = config::Config::from_file("config.toml").unwrap_or_else(|e| {
        tracing::error!("💥 Failed to load configuration: {}", e);
        std::process::exit(1);
    });

    tracing::info!("🔧 Configuration loaded successfully, validating...");
    config.validate().unwrap_or_else(|e| {
        tracing::error!("💥 Configuration validation failed: {}", e);
        std::process::exit(1);
    });
    tracing::info!("  ✨ Configuration is valid");

    println!("🔌 Connecting to database at: {}", config.db_path.display());
    let db = database::LocalDatabase::new(&config.db_path)
        .await
        .unwrap_or_else(|e| {
            tracing::error!("💥 Failed to connect to database: {}", e);
            std::process::exit(1);
        });
    tracing::info!("  ✨ Connected to database, creating tables if needed...");
    db.setup().await.unwrap_or_else(|e| {
        tracing::error!("💥 Failed to setup database: {}", e);
        std::process::exit(1);
    });
    tracing::info!("  ✨ Database setup complete");

    let komga_client = KomgaClient::instance(&config.komga);
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

    let navidrome_client = match &config.navidrome {
        Some(config) => {
            tracing::info!("🔌 Connecting to Navidrome at: {}", config.host);
            match navidrome::NavidromeClient::new(config).await {
                Ok(client) => {
                    tracing::info!("  ✨ Connected to Navidrome");
                    if !client.claims().adm {
                        tracing::error!(
                            "  😔 Provided Navidrome user is not an ADMIN, please use an account with admin privilege!"
                        );
                        std::process::exit(1);
                    }
                    Some(Arc::new(Mutex::new(client)))
                }
                Err(e) => {
                    tracing::error!("  💥 Failed to connect to Navidrome: {}", e);
                    std::process::exit(1);
                }
            }
        }
        None => {
            tracing::info!("🔌 No Navidrome configuration found, skipping connection");
            None
        }
    };

    let state = AppState {
        db: Arc::new(db),
        config: Arc::new(config),
        komga: Arc::new(komga_client),
        navidrome: navidrome_client,
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
    let run_at = format!("{}:{}", host_at, port_at);
    tracing::info!("🚀 Starting K-Librarian at: http://{}", &run_at);
    let listener = TcpListener::bind(run_at).await.unwrap();
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
