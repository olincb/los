use super::common::{ReaderType, SourceType, build_elevation_service};

pub fn handle_elevation_command(
    reader_type: ReaderType,
    source_type: SourceType,
    local_dem: Option<String>,
    url_dem: Option<String>,
    lat: f64,
    lon: f64,
) -> anyhow::Result<()> {
    let elevation_service = build_elevation_service(reader_type, source_type, local_dem, url_dem)?;
    let elevation = elevation_service.elevation_at(lat, lon)?;
    println!(
        "Elevation at ({}, {}): {:.2} m ({:.2} ft)",
        lat, lon, elevation.m, elevation.ft
    );
    Ok(())
}
