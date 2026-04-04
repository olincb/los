use crate::Elevation;
use crate::reader::{DemReader, DemReaderError};
use crate::source::{DemSource, DemSourceError, Location};

#[derive(thiserror::Error, Debug)]
pub enum ElevationServiceError {
    #[error("source error: {0}")]
    Source(#[from] DemSourceError),
    #[error("reader error: {0}")]
    Reader(#[from] DemReaderError),
}

pub struct ElevationService {
    pub source: Box<dyn DemSource>,
    pub reader: Box<dyn DemReader>,
    pub dem_location: Option<Location>,
}

impl ElevationService {
    pub fn elevation_at(&self, lat: f64, lon: f64) -> Result<Elevation, ElevationServiceError> {
        let dem_location = match &self.dem_location {
            Some(location) => location,
            None => &self.source.get_dem_for_point(lat, lon)?,
        };
        let handle = self.reader.open(dem_location)?;
        let elev = handle.elevation_at(lat, lon)?;

        Ok(elev)
    }
}
