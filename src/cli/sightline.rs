use super::common::{ReaderType, SourceType, build_elevation_service};
use los::Bbox;
use los::service::{LineOfSightResult, LineOfSightService};

pub fn handle_sightline_command(
    lat: f64,
    lon: f64,
    target_lat: f64,
    target_lon: f64,
) -> anyhow::Result<()> {
    let epsilon = 0.0001;
    let mut elevation_service =
        build_elevation_service(ReaderType::Gdal, SourceType::Usgs, None, None)?;
    let bbox = Bbox {
        min_lat: lat.min(target_lat) - epsilon,
        max_lat: lat.max(target_lat) + epsilon,
        min_lon: lon.min(target_lon) - epsilon,
        max_lon: lon.max(target_lon) + epsilon,
    };
    elevation_service.prefetch_region(&bbox)?;
    let los_service = LineOfSightService::new(Box::new(elevation_service));
    let viewer_height_m = 2.0; // Giving the caller the benefit of the doubt.
    match los_service.has_line_of_sight_with_height(
        lat,
        lon,
        target_lat,
        target_lon,
        viewer_height_m,
    )? {
        LineOfSightResult::Clear => println!(
            "Line of sight from ({}, {}) to ({}, {}) is clear.",
            lat, lon, target_lat, target_lon
        ),
        LineOfSightResult::Blocked {
            lat: blocking_lat,
            lon: blocking_lon,
            terrain_m,
            sightline_m,
        } => println!(
            "Line of sight from ({}, {}) to ({}, {}) is blocked by terrain at ({:.7}, {:.7}) with elevation {:.2} m, which is {:.2} m above the sightline.",
            lat,
            lon,
            target_lat,
            target_lon,
            blocking_lat,
            blocking_lon,
            terrain_m,
            terrain_m - sightline_m
        ),
    }

    Ok(())
}
