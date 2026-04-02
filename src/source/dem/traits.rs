use crate::source::Location;

#[derive(thiserror::Error, Debug)]
pub enum DemSourceError {
    #[error("network error: {0}")]
    Network(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("bbox out of coverage area")]
    OutOfCoverage,
}

pub trait DemSource {
    // fn get_dem_for_bbox(&self, bbox: &Bbox) -> Result<Location, DemSourceError>;
    fn get_dem_for_point(&self, lat: f64, lon: f64) -> Result<Location, DemSourceError>;
}
