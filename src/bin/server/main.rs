mod api;

use api::highlight;
use axum::{
    Json, Router,
    extract::{Query, Request},
    http::{StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use image::ImageFormat;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
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
        .fallback(page_not_found)
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn(log_request))
                .layer(middleware::from_fn(bump_timeout)),
        );

    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
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

async fn bump_timeout(request: Request, next: Next) -> Response {
    // TODO: mark that there is a request ongoing, then reset last completed at time
    //       then also a background task which shuts things down if there is no request ongoing for
    //       a while
    println!("Bumping timeout");
    let response = next.run(request).await;
    response
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
    println!("GET /api/v1/health");
    "ok"
}

async fn highlight(Query(params): Query<HighlightParams>) -> Result<impl IntoResponse, StatusCode> {
    println!(
        "GET /api/v1/highlight with lat={}, lon={}",
        params.lat, params.lon
    );
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
