use super::common::{ReaderType, SourceType, build_elevation_service};
use los::service::LineOfSightService;
use terminal_size::{Width, terminal_size};

pub fn handle_viewshed_command(
    lat: f64,
    lon: f64,
    width: f64,
    cols: Option<usize>,
    rows: Option<usize>,
) -> anyhow::Result<()> {
    let term_cols = if let Some((Width(w), _)) = terminal_size() {
        w as usize
    } else {
        80 // Default terminal size if it can't be determined.
    };
    let cols = cols.unwrap_or(term_cols);
    let char_aspect = 2.3; // Terminal characters are taller than they are wide.
    let rows = rows.unwrap_or((cols as f64 / char_aspect).round() as usize);
    // 1 degree of latitude is constant distance, but 1 degree of longitude
    // varies based on latitude.
    let lon_stretch = 1.0 / lat.to_radians().cos();
    let height = width / lon_stretch;
    let bbox = los::Bbox {
        min_lat: lat - height / 2.0,
        max_lat: lat + height / 2.0,
        min_lon: lon - width / 2.0,
        max_lon: lon + width / 2.0,
    };
    println!(
        "Displaying viewshed for ({}, {}) with width {} degrees in a grid of {} cols x {} rows.",
        lat, lon, width, cols, rows
    );
    let mut elevation_service =
        build_elevation_service(ReaderType::Gdal, SourceType::Usgs, None, None)?;
    println!("Prefetching elevation data for bounding box: {:?}", bbox);
    elevation_service.prefetch_region(&bbox)?;
    let los_service = LineOfSightService::new(Box::new(elevation_service));
    let viewer_height_m = 3.0; // Giving the caller the benefit of the doubt.
    println!("Calculating viewshed...");
    let result =
        los_service.viewshed_for_grid(lat, lon, bbox, cols, rows, Some(viewer_height_m))?;
    if result.width != cols || result.height != rows {
        println!(
            "Warning: Output size ({}, {}) does not match requested size ({}, {}).",
            result.width, result.height, cols, rows
        );
    }
    print!("{}", result);
    Ok(())
}
