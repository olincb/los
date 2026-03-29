use std::path::PathBuf;

// TODO: just use a point instead of a bbox for now
#[derive(Debug, Clone, Copy)]
pub struct Bbox {
    pub min_lat: f64,
    pub min_lon: f64,
    pub max_lat: f64,
    pub max_lon: f64,
}

#[derive(Debug, Clone)]
pub enum DemLocation {
    LocalPath(PathBuf),
    RemoteUrl(String),
}

#[derive(Debug, Clone)]
pub struct DemDescriptor {
    pub location: DemLocation,
}

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
    // fn get_dem_for_bbox(&self, bbox: &Bbox) -> Result<DemDescriptor, DemSourceError>;
    fn get_dem_for_point(&self, lat: f64, lon: f64) -> Result<DemDescriptor, DemSourceError>;
}
