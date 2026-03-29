use crate::reader::{DemHandle, DemReader, DemReaderError};
use crate::source::{DemSource, DemSourceError};

#[derive(thiserror::Error, Debug)]
pub enum ElevationServiceError {
    #[error("source error: {0}")]
    Source(#[from] DemSourceError),
    #[error("reader error: {0}")]
    Reader(#[from] DemReaderError),
}

pub struct ElevationService<S, R> {
    pub source: S,
    pub reader: R,
}

impl<S, R> ElevationService<S, R>
where
    S: DemSource,
    R: DemReader,
{
    pub fn elevation_at(&self, lat: f64, lon: f64) -> Result<f64, ElevationServiceError> {
        // let bbox = Bbox {
        //     min_lat: lat,
        //     max_lat: lat,
        //     min_lon: lon,
        //     max_lon: lon,
        // };

        let desc = self.source.get_dem_for_point(lat,  lon )?;
        let handle = self.reader.open(&desc)?;
        let elev = handle.elevation_at(lat, lon)?;

        Ok(elev.height_m)
    }
}
