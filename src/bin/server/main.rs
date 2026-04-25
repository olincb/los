mod api;

use api::highlight;
use axum::{
    Router,
    extract::Query,
    http::{StatusCode, header},
    response::IntoResponse,
    routing::get,
};
use image::ImageFormat;
use serde::Deserialize;
use std::io::Cursor;

#[derive(Deserialize)]
struct HighlightParams {
    lat: f64,
    lon: f64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/highlight", get(highlight));

    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
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
