mod common;

use common::{
    SAMPLE_OB_POINT, SAMPLE_POINT_1, SAMPLE_POINT_2, assert_reader_returns_expected_elevation,
    assert_reader_returns_out_of_bounds,
};
use rstest::rstest;

use los::GeoTiffReader;

#[rstest]
#[case(SAMPLE_POINT_1)]
#[case(SAMPLE_POINT_2)]
fn test_geotiff_reader_returns_expected_elevation(#[case] point: (f64, f64, f64)) {
    let (lat, lon, expected_elevation) = point;
    assert_reader_returns_expected_elevation(&GeoTiffReader, lat, lon, expected_elevation);
}

#[rstest]
#[case(SAMPLE_OB_POINT)]
fn test_geotiff_reader_out_of_bounds(#[case] point: (f64, f64)) {
    let (lat, lon) = point;
    assert_reader_returns_out_of_bounds(&GeoTiffReader, lat, lon);
}
