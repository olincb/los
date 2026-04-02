use crate::Bbox;
use crate::source::{Location, SourceError};

pub struct TopoMapDescriptor {
    location: Location,
    bbox: Bbox,
}

pub trait TopoSource {
    /// Retrieves topographical map for the specified location.
    fn get_map_for_point(&self, lat: f64, lon: f64) -> Result<TopoMapDescriptor, SourceError>;
}
