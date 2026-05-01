use los::service::LineOfSightService;
mod common;

use common::{
    RIDGE_POINT_1, RIDGE_POINT_2, VALLEY_POINT_1, VALLEY_POINT_2, build_test_elevation_service,
};

#[test]
fn test_line_of_sight_ridge_to_ridge() {
    let service = build_test_elevation_service();
    let (lat1, lon1) = RIDGE_POINT_1;
    let (lat2, lon2) = RIDGE_POINT_2;
    let elevation_provider = service
        .fetch_region(los::Bbox {
            min_lat: lat1.min(lat2),
            max_lat: lat1.max(lat2),
            min_lon: lon1.min(lon2),
            max_lon: lon1.max(lon2),
        })
        .unwrap();
    let los_service = LineOfSightService::new(elevation_provider);
    assert!(
        los_service
            .has_line_of_sight(lat1, lon1, lat2, lon2)
            .unwrap()
            .is_clear()
    );
}

#[test]
fn test_line_of_sight_valley_to_valley() {
    let service = build_test_elevation_service();
    let (lat1, lon1) = VALLEY_POINT_1;
    let (lat2, lon2) = VALLEY_POINT_2;
    let elevation_provider = service
        .fetch_region(los::Bbox {
            min_lat: lat1.min(lat2),
            max_lat: lat1.max(lat2),
            min_lon: lon1.min(lon2),
            max_lon: lon1.max(lon2),
        })
        .unwrap();
    let los_service = LineOfSightService::new(elevation_provider);
    assert!(
        los_service
            .has_line_of_sight(lat1, lon1, lat2, lon2)
            .unwrap()
            .is_blocked()
    );
}
