use crate::Elevation;
use crate::source::Location;

#[derive(thiserror::Error, Debug)]
pub enum DemReaderError {
    #[error("gdal error: {0}")]
    Gdal(String),
    #[error("geotiff error: {0}")]
    GeoTiff(String),
    #[error("out of bounds: {0}")]
    OutOfBounds(String),
}

pub trait DemReader {
    fn open(&self, loc: &Location) -> Result<Box<dyn DemHandle>, DemReaderError>;
}

pub trait DemHandle {
    fn elevation_at(&self, lat: f64, lon: f64) -> Result<Elevation, DemReaderError>;
}
