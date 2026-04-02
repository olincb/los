use los::source::Location;
use los::{DemHandle, DemReader, DemReaderError};
use std::path::PathBuf;

pub const SAMPLE_POINT_1: (f64, f64, f64) = (40.2468642, -105.5959595, 3897.397);
pub const SAMPLE_POINT_2: (f64, f64, f64) = (40.2828282, -105.6666666, 3330.404);

pub const SAMPLE_OB_POINT: (f64, f64) = (42.0, -101.0);

pub fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join(name)
}

pub fn local_dem_descriptor(name: &str) -> Location {
    Location::LocalPath(fixture_path(name))
}

pub fn assert_reader_returns_expected_elevation<R: DemReader>(
    reader: &R,
    lat: f64,
    lon: f64,
    expected_elevation: f64,
) {
    let desc = local_dem_descriptor("sample_dem.tif");
    let handle = reader.open(&desc).expect("Failed to open DEM");
    let elevation = handle
        .elevation_at(lat, lon)
        .expect("Failed to get elevation");
    assert!(
        (elevation.height_m - expected_elevation).abs() < 0.1,
        "Expected elevation around {}, got {}",
        expected_elevation,
        elevation.height_m
    );
}

pub fn assert_reader_returns_out_of_bounds<R: DemReader>(reader: &R, lat: f64, lon: f64) {
    let desc = local_dem_descriptor("sample_dem.tif");
    let handle = reader.open(&desc).expect("Failed to open DEM");
    let result = handle.elevation_at(lat, lon);
    assert!(
        matches!(result, Err(DemReaderError::OutOfBounds(_))),
        "Expected OutOfBounds error, got {:?}",
        result
    );
}
