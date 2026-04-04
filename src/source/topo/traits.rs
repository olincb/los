use crate::Bbox;
use crate::source::{Location, SourceError};
use std::path::PathBuf;

#[derive(Debug)]
pub struct TopoMapDescriptor {
    pub name: Option<String>,
    pub location: Location,
    pub bbox: Bbox,
}

pub trait TopoSource {
    /// Retrieves descriptor of topographical map for the specified location.
    fn get_map_descriptor(&self, lat: f64, lon: f64) -> Result<TopoMapDescriptor, SourceError>;

    /// Fetches the topographical map described by the descriptor and returns a local path to the file.
    fn fetch_map(&self, descriptor: &TopoMapDescriptor) -> Result<PathBuf, SourceError>;
}
