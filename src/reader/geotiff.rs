use geo_types::Coord;
use geotiff;
use std::fs::File;

use crate::reader::{DemHandle, DemReader, DemReaderError};
use crate::source::Location;
use crate::{Bbox, Elevation};

pub struct GeoTiffReader;

impl DemReader for GeoTiffReader {
    fn open(&self, loc: &Location) -> Result<Box<dyn DemHandle>, DemReaderError> {
        let filepath = match loc {
            Location::LocalPath(path) => path,
            Location::RemoteUrl(url) => {
                return Err(DemReaderError::GeoTiff(format!(
                    "GeoTiffReader does not support URLs, but got {}",
                    url
                )));
            }
        };
        let file = match File::open(filepath) {
            Ok(file) => file,
            Err(e) => {
                return Err(DemReaderError::GeoTiff(format!(
                    "Could not open {} due to: {}",
                    filepath.to_string_lossy(),
                    e,
                )));
            }
        };
        let reader = match geotiff::GeoTiff::read(file) {
            Ok(reader) => reader,
            Err(e) => {
                return Err(DemReaderError::GeoTiff(format!(
                    "Could not read {} as Tiff due to: {}",
                    filepath.to_string_lossy(),
                    e,
                )));
            }
        };
        let bounds = reader.model_extent();
        let Coord {
            x: min_lon,
            y: min_lat,
        } = bounds.min();
        let Coord {
            x: max_lon,
            y: max_lat,
        } = bounds.max();
        Ok(Box::new(GeoTiffDemHandle {
            reader,
            bbox: Bbox {
                min_lon,
                min_lat,
                max_lon,
                max_lat,
            },
        }))
    }
}

pub struct GeoTiffDemHandle {
    reader: geotiff::GeoTiff,
    bbox: Bbox,
}

impl DemHandle for GeoTiffDemHandle {
    fn elevation_at(&self, lat: f64, lon: f64) -> Result<Elevation, DemReaderError> {
        match (lat, self.bbox.min_lat, self.bbox.max_lat) {
            (l, min, _) if l < min => {
                return Err(DemReaderError::OutOfBounds(format!(
                    "Latitude {} is less than minimum of {} for GeoTiffDemHandle.",
                    l, min
                )));
            }
            (l, _, max) if l > max => {
                return Err(DemReaderError::OutOfBounds(format!(
                    "Latitude {} is greater than maximum of {} for GeoTiffDemHandle.",
                    l, max
                )));
            }
            _ => (),
        }
        match (lon, self.bbox.min_lon, self.bbox.max_lon) {
            (l, min, _) if l < min => {
                return Err(DemReaderError::OutOfBounds(format!(
                    "Longitude {} is less than minimum of {} for GeoTiffDemHandle.",
                    l, min
                )));
            }
            (l, _, max) if l > max => {
                return Err(DemReaderError::OutOfBounds(format!(
                    "Longitude {} is greater than maximum of {} for GeoTiffDemHandle.",
                    l, max
                )));
            }
            _ => (),
        }
        let point = Coord { x: lon, y: lat };
        match self.reader.get_value_at(&point, 0) {
            Some(el) => Ok(Elevation::from_m(el)),
            None => Err(DemReaderError::GeoTiff(format!(
                "Could not read elevation at point {}, {} from GeoTiff",
                point.x, point.y
            ))),
        }
    }
}
