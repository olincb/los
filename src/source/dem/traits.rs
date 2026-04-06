use crate::Bbox;
use crate::source::{Location, SourceError};

#[derive(thiserror::Error, Debug)]
pub enum DemSourceError {
    #[error(transparent)]
    Source(#[from] SourceError),
    #[error("bbox out of coverage area")]
    OutOfCoverage,
}

pub trait DemSource {
    fn get_dem_for_point(&self, lat: f64, lon: f64) -> Result<Location, DemSourceError>;
    fn get_dem_for_bbox(&self, bbox: &Bbox) -> Result<Location, DemSourceError>;
}
