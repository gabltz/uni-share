mod convert;
mod error;
mod models;
mod providers;

use axum::{
    extract::{Query, State},
    http::{header, HeaderValue, Method, StatusCode},
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use models::{ConvertQuery, ConvertRequest, ConvertResponse, ErrorBody, HealthResponse};
use std::{env, net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    set_header::SetResponseHeaderLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub struct AppState {
    pub converter: Arc<convert::Converter>,
    pub shortcut_install_url: Option<String>,
    pub public_base_url: Option<String>,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "uni_share=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let converter = convert::Converter::from_env()
        .await
        .expect("failed to initialize converter");

    let state = AppState {
        converter: Arc::new(converter),
        shortcut_install_url: env::var("SHORTCUT_INSTALL_URL").ok().filter(|s| !s.is_empty()),
        public_base_url: env::var("PUBLIC_BASE_URL").ok().filter(|s| !s.is_empty()),
    };

    let static_dir = static_dir();
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/convert", get(convert_get).post(convert_post))
        .route("/api/v1/config", get(public_config))
        .route("/", get(index_page))
        .route("/index.html", get(index_page))
        .nest_service("/assets", ServeDir::new(static_dir.join("assets")))
        .nest_service("/shortcut", ServeDir::new(static_dir.join("shortcut")))
        .layer(cors)
        .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .with_state(state);

    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("UNI-SHARE listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind failed");
    axum::serve(listener, app).await.expect("server failed");
}

fn static_dir() -> PathBuf {
    let candidates = [
        PathBuf::from("static"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static"),
    ];
    for path in candidates {
        if path.exists() {
            return path;
        }
    }
    PathBuf::from("static")
}

async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
        service: "uni-share".into(),
        spotify_configured: state.converter.spotify_configured(),
        version: env!("CARGO_PKG_VERSION").into(),
    })
}

async fn public_config(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "shortcutInstallUrl": state.shortcut_install_url,
        "publicBaseUrl": state.public_base_url,
        "supportedTargets": ["spotify", "apple_music", "deezer", "tidal"],
    }))
}

async fn index_page() -> impl IntoResponse {
    let path = static_dir().join("index.html");
    match tokio::fs::read_to_string(&path).await {
        Ok(html) => Html(html).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "landing page missing",
        )
            .into_response(),
    }
}

async fn convert_get(
    State(state): State<AppState>,
    Query(query): Query<ConvertQuery>,
) -> impl IntoResponse {
    run_convert(&state, query.url, query.target, query.country).await
}

async fn convert_post(
    State(state): State<AppState>,
    Json(body): Json<ConvertRequest>,
) -> impl IntoResponse {
    run_convert(&state, body.url, body.target, body.country).await
}

async fn run_convert(
    state: &AppState,
    url: String,
    target: String,
    country: Option<String>,
) -> axum::response::Response {
    let started = std::time::Instant::now();
    match state
        .converter
        .convert(&url, &target, country.as_deref())
        .await
    {
        Ok(result) => {
            tracing::info!(
                target = %result.target,
                elapsed_ms = started.elapsed().as_millis() as u64,
                "converted"
            );
            (
                StatusCode::OK,
                Json(ConvertResponse {
                    url: result.url,
                    target: result.target,
                    title: result.title,
                    artist: result.artist,
                    isrc: result.isrc,
                    source_url: result.source_url,
                    thumbnail: result.thumbnail,
                }),
            )
                .into_response()
        }
        Err(err) => {
            let status = err.status_code();
            tracing::warn!(error = %err, status = %status, "convert failed");
            (
                status,
                Json(ErrorBody {
                    error: err.code().into(),
                    message: err.to_string(),
                }),
            )
                .into_response()
        }
    }
}

#[allow(dead_code)]
fn default_timeout() -> Duration {
    Duration::from_secs(20)
}
