use super::los::ElevationProvider;
use crate::reader::{DemHandle, DemReader, DemReaderError};
use crate::source::{DemSource, DemSourceError, Location};
use crate::{Bbox, Elevation};

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
    handle: Option<Box<dyn DemHandle>>,
    prefetch_margin: f64,
}

impl ElevationService {
    pub fn new(
        source: Box<dyn DemSource>,
        reader: Box<dyn DemReader>,
        dem_location: Option<Location>,
    ) -> Self {
        ElevationService {
            source,
            reader,
            dem_location,
            handle: None,
            prefetch_margin: 0.0001,
        }
    }

    pub fn elevation_at(&self, lat: f64, lon: f64) -> Result<Elevation, ElevationServiceError> {
        if let Some(handle) = &self.handle {
            return Ok(handle.elevation_at(lat, lon)?);
        }
        let dem_location = match &self.dem_location {
            Some(location) => location,
            None => &self.source.get_dem_for_point(lat, lon)?,
        };
        let handle = self.reader.open(dem_location)?;
        Ok(handle.elevation_at(lat, lon)?)
    }

    /// Prefetches a region of elevation data around the given bounding box, with an additional
    /// margin. This prevents multiple round-trips to the source and reader when requesting
    /// elevations for points that fall within the margin of the bounding box.
    pub fn prefetch_region(&mut self, bbox: &Bbox) -> Result<(), ElevationServiceError> {
        // TODO: Save to disk by bbox and load from disk if already fetched, to avoid repeated
        // fetching and reading of the same data across multiple runs of the program.
        let bbox = bbox.with_margin(self.prefetch_margin);
        if self.dem_location.is_none() {
            self.dem_location = Some(self.source.get_dem_for_bbox(&bbox)?);
        }
        let loc = self
            .dem_location
            .as_ref()
            .expect("Unexpected error: dem_location should have been set by this point.");
        if self.handle.is_none() {
            self.handle = Some(self.reader.open(loc)?);
        }
        let handle = self
            .handle
            .as_mut()
            .expect("Unexpected error: handle should have been set by this point.");
        handle.prefetch_region(bbox)?;

        Ok(())
    }
}

impl ElevationProvider for ElevationService {
    fn elevation_at(&self, lat: f64, lon: f64) -> Result<Elevation, anyhow::Error> {
        Ok(self.elevation_at(lat, lon)?)
    }
}
