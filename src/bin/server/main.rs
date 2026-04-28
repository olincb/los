mod api;

use api::highlight;
use axum::{
    Json,
    Router,
    extract::Query,
    http::{StatusCode, header},
    response::IntoResponse,
    routing::get,
};
use tower_http::services::{ServeDir, ServeFile};
use image::ImageFormat;
use serde::{ Deserialize, Serialize };
use std::io::Cursor;

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
        .fallback(page_not_found);

    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn api_not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, Json(ApiError { error: "Endpoint not found" }))
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


async fn health() -> &'static str {
    println!("GET /api/v1/health");
    "ok"
}

async fn highlight(Query(params): Query<HighlightParams>) -> Result<impl IntoResponse, StatusCode> {
    println!(
        "GET /api/v1/highlight with lat={}, lon={}",
        params.lat, params.lon
    );
    let image =
        highlight::handle_highlight_endpoint_command(params.lat, params.lon).map_err(|e| {
            eprintln!("Error handling highlight endpoint command: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut buf = Cursor::new(Vec::new());
    image.write_to(&mut buf, ImageFormat::Png).map_err(|e| {
        eprintln!("Error encoding image to PNG: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/png")],
        buf.into_inner(),
    ))
}
