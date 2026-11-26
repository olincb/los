mod api;

use api::highlight;
use axum::{
    Json, Router,
    extract::{Query, Request, State},
    http::{StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use image::ImageFormat;
use serde::{Deserialize, Serialize};
use std::{
    io::Cursor,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering::SeqCst},
    },
    time::{Duration, Instant},
};
use tower::ServiceBuilder;
use tower_http::services::{ServeDir, ServeFile};

#[derive(Deserialize)]
struct HighlightParams {
    lat: f64,
    lon: f64,
}

#[derive(Serialize)]
struct ApiError {
    error: &'static str,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let activity = Arc::new(Activity {
        active_requests: AtomicUsize::new(0),
        last_static_completed: Mutex::new(None),
        last_api_completed: Mutex::new(None),
    });
    let api = Router::new()
        .route("/health", get(health))
        .route("/highlight", get(highlight))
        .fallback(api_not_found);
    let static_dir = std::env::var("STATIC_DIR").unwrap_or_else(|_| "static".to_string());
    let app = Router::new()
        .nest("/api/v1", api)
        .route_service("/", ServeFile::new(format!("{static_dir}/index.html")))
        .route_service("/coffee", get(im_a_teapot))
        .nest_service("/static", ServeDir::new(static_dir))
        .layer(middleware::from_fn_with_state(
            activity.clone(),
            set_last_completion_time,
        )) // Before fallback to avoid keeping server alive for 404s (scrapers)
        .fallback(page_not_found)
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn(log_request))
                .layer(middleware::from_fn_with_state(
                    activity.clone(),
                    prevent_exit,
                )),
        );

    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, app)
        .with_graceful_shutdown(idle_too_long(activity.clone()))
        .await?;
    Ok(())
}

// Auto-retire the server after a period of inactivity.
// This runs on an expensive compute tier and idle time adds up quickly.
// On next request, Fly.io will spin up a new instance.

const API_KEEPALIVE: Duration = Duration::from_secs(240);
const STATIC_KEEPALIVE: Duration = Duration::from_secs(30);

struct Activity {
    active_requests: AtomicUsize,
    last_static_completed: Mutex<Option<Instant>>,
    last_api_completed: Mutex<Option<Instant>>,
}

impl Activity {
    fn api_idle(&self) -> bool {
        match *self.last_api_completed.lock().unwrap() {
            None => true, // No API requests have completed; vacuously idle
            Some(at) => at.elapsed() > API_KEEPALIVE,
        }
    }

    fn static_idle(&self) -> bool {
        match *self.last_static_completed.lock().unwrap() {
            None => true, // No static requests have completed; vacuously idle
            Some(at) => at.elapsed() > STATIC_KEEPALIVE,
        }
    }

    fn is_idle(&self) -> bool {
        self.active_requests.load(SeqCst) == 0 && self.api_idle() && self.static_idle()
    }
}

struct RequestGuard {
    activity: Arc<Activity>,
}

impl RequestGuard {
    fn new(activity: Arc<Activity>) -> Self {
        activity.active_requests.fetch_add(1, SeqCst);
        Self { activity }
    }
}

impl Drop for RequestGuard {
    fn drop(&mut self) {
        self.activity.active_requests.fetch_sub(1, SeqCst);
    }
}

/// Returning from this function shuts the server down
async fn idle_too_long(activity: Arc<Activity>) {
    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;
        if activity.is_idle() {
            println!("Shutting down idle server");
            return;
        }
    }
}

// Middleware

fn header_text(request: &Request, name: &str) -> String {
    request
        .headers()
        .get(name)
        .map(|v| v.to_str().unwrap_or("<invalid>"))
        .unwrap_or("<none>")
        .to_owned()
}

async fn log_request(request: Request, next: Next) -> Response {
    let started = std::time::Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let host = header_text(&request, header::HOST.as_str());
    let user_agent = header_text(&request, header::USER_AGENT.as_str());
    let ip = header_text(&request, "Fly-Client-IP");
    let region = header_text(&request, "Fly-Region");
    let referer = header_text(&request, header::REFERER.as_str());
    let response = next.run(request).await;
    println!(
        "Handled request method:{method} host:{host:?} uri:{uri:?} status:{} duration:{:.3}s user-agent:{user_agent:?} ip:{ip:?} region:{region:?} referer:{referer:?}",
        response.status().as_u16(),
        started.elapsed().as_secs_f32()
    );
    response
}

async fn set_last_completion_time(
    State(activity): State<Arc<Activity>>,
    request: Request,
    next: Next,
) -> Response {
    let is_api = request.uri().path().starts_with("/api/");
    let response = next.run(request).await;

    // Early return if request failed
    if !response.status().is_success() {
        return response;
    }

    let last_completed = if is_api {
        &activity.last_api_completed
    } else {
        &activity.last_static_completed
    };
    *last_completed.lock().unwrap() = Some(Instant::now());
    response
}

async fn prevent_exit(
    State(activity): State<Arc<Activity>>,
    request: Request,
    next: Next,
) -> Response {
    // Activity::active_requests is incremented by RequestGuard::new,
    // and decremented by RequestGuard::drop, which is called when this function
    // returns, either normally or if the future is canceled.
    let _guard = RequestGuard::new(activity);
    next.run(request).await
}

// Error handlers

async fn api_not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(ApiError {
            error: "Endpoint not found",
        }),
    )
}

async fn page_not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        [(header::CONTENT_TYPE, "text/html")],
        include_str!("static/404.html"),
    )
}

async fn im_a_teapot() -> impl IntoResponse {
    (
        StatusCode::IM_A_TEAPOT,
        [(header::CONTENT_TYPE, "text/html")],
        include_str!("static/418.html"),
    )
}

// API handlers

async fn health() -> &'static str {
    "ok"
}

async fn highlight(Query(params): Query<HighlightParams>) -> Result<impl IntoResponse, StatusCode> {
    let lat = params.lat;
    let lon = params.lon;
    let image =
        tokio::task::spawn_blocking(move || highlight::handle_highlight_endpoint_command(lat, lon))
            .await
            .map_err(|e| {
                eprintln!("Highlight worker task failed: {e}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .map_err(|e| {
                eprintln!("Error generating highlight image: {e}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

    println!("Writing image to PNG format...");
    let t = std::time::Instant::now();
    let mut buf = Cursor::new(Vec::new());
    image.write_to(&mut buf, ImageFormat::Png).map_err(|e| {
        eprintln!("Error encoding image to PNG: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    println!(
        "Encoded image to PNG format in {:.3}s",
        t.elapsed().as_secs_f32()
    );
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/png")],
        buf.into_inner(),
    ))
}
