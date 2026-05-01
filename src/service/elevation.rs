use crate::Bbox;
use crate::reader::{DemHandle, DemReader, DemReaderError};
use crate::source::{DemSource, DemSourceError, Location};

#[derive(thiserror::Error, Debug)]
pub enum ElevationServiceError {
    #[error("source error: {0}")]
    Source(#[from] DemSourceError),
    #[error("reader error: {0}")]
    Reader(#[from] DemReaderError),
}

pub struct ElevationService {
    source: Box<dyn DemSource>,
    reader: Box<dyn DemReader>,
    dem_location: Option<Location>,
    fetch_margin_deg: f64,
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
            fetch_margin_deg: 0.0001,
        }
    }

    /// Fetches a region of elevation data around the given bounding box, with an additional
    /// margin. This prevents multiple round-trips to the source and reader when requesting
    /// elevations for points that fall within the margin of the bounding box.
    pub fn fetch_region(&self, bbox: Bbox) -> Result<Box<dyn DemHandle>, ElevationServiceError> {
        // TODO: Save to disk by bbox and load from disk if already fetched, to avoid repeated
        // fetching and reading of the same data across multiple runs of the program.
        let bbox = bbox.with_margin(self.fetch_margin_deg);
        let loc = match &self.dem_location {
            Some(location) => location,
            None => &self.source.get_dem_for_bbox(&bbox)?,
        };
        Ok(self.reader.open(loc, bbox)?)
    }
}
