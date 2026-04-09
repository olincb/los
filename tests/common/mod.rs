#![allow(dead_code)]
use los::source::Location;
use los::{DemReader, DemReaderError, ElevationService};
use std::path::PathBuf;

pub const SAMPLE_POINT_1: (f64, f64, f64) = (40.2468642, -105.5959595, 3897.397);
pub const SAMPLE_POINT_2: (f64, f64, f64) = (40.2828282, -105.6666666, 3330.404);
pub const RIDGE_POINT_1: (f64, f64) = (40.2716666, -105.6580555);
pub const VALLEY_POINT_1: (f64, f64) = (40.2690363, -105.6582074);
pub const RIDGE_POINT_2: (f64, f64) = (40.267222, -105.6545037);
pub const VALLEY_POINT_2: (f64, f64) = (40.2625922, -105.6503903);

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
        (elevation.m - expected_elevation).abs() < 0.1,
        "Expected elevation around {}, got {}",
        expected_elevation,
        elevation.m
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

pub fn build_test_elevation_service() -> ElevationService {
    let loc = local_dem_descriptor("sample_dem.tif");
    ElevationService::new(
        Box::new(los::source::UsgsSource), // Won't be used b/c we provide a dem_location.
        Box::new(los::GdalReader),
        Some(loc),
    )
}
