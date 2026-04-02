use crate::DemSource;
/// USGS 1/3 Arc-Second DEM implementation for retrieving elevation data.
use crate::source::{DemSourceError, Location};

pub struct UsgsSource;

const USGS_13_VRT_URL: &str =
    "https://prd-tnm.s3.amazonaws.com/StagedProducts/Elevation/13/TIFF/USGS_Seamless_DEM_13.vrt";
const USGS_LAT_MIN: f64 = -15.0;
const USGS_LAT_MAX: f64 = 72.0;
const USGS_LON_MIN: f64 = -180.0;
const USGS_LON_MAX: f64 = 180.0;

fn in_bounds(lat: f64, lon: f64) -> bool {
    (USGS_LAT_MIN..=USGS_LAT_MAX).contains(&lat) && (USGS_LON_MIN..=USGS_LON_MAX).contains(&lon)
}

impl DemSource for UsgsSource {
    fn get_dem_for_point(&self, lat: f64, lon: f64) -> Result<Location, DemSourceError> {
        // TODO: more precise bounds checking based on actual tile coverage
        if !in_bounds(lat, lon) {
            return Err(DemSourceError::OutOfCoverage);
        }

        Ok(Location::RemoteUrl(USGS_13_VRT_URL.into()))
    }
}
