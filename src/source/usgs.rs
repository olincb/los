/// USGS 1/3 Arc-Second DEM implementation for retrieving elevation data.
use crate::source::DemSourceError;
use crate::{DemDescriptor, DemSource};

pub struct UsgsSource;

const USGS_13_VRT_URL: &str =
    "https://prd-tnm.s3.amazonaws.com/StagedProducts/Elevation/13/TIFF/USGS_Seamless_DEM_13.vrt";
const USGS_LAT_MIN: f64 = -15.0;
const USGS_LAT_MAX: f64 = 72.0;
const USGS_LON_MIN: f64 = -180.0;
const USGS_LON_MAX: f64 = 180.0;

fn in_bounds(lat: f64, lon: f64) -> bool {
    lat >= USGS_LAT_MIN && lat <= USGS_LAT_MAX && lon >= USGS_LON_MIN && lon <= USGS_LON_MAX
}

impl DemSource for UsgsSource {
    fn get_dem_for_point(&self, lat: f64, lon: f64) -> Result<DemDescriptor, DemSourceError> {
        // TODO: more precise bounds checking based on actual tile coverage
        if !in_bounds(lat, lon) {
            return Err(DemSourceError::OutOfCoverage);
        }

        Ok(DemDescriptor {
            location: crate::source::DemLocation::RemoteUrl(USGS_13_VRT_URL.into()),
        })
    }
}
