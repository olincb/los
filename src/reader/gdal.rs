use crate::reader::{DemHandle, DemReader, DemReaderError};
use crate::source::{DemDescriptor, DemLocation};

pub struct GdalReader;

impl DemReader for GdalReader {
    fn open(&self, desc: &DemDescriptor) -> Result<impl DemHandle, DemReaderError> {
        // GDAL can handle both local paths and remote URLs.
        let dataset = match desc.location {
            DemLocation::LocalPath(ref path) => gdal::Dataset::open(path),
            DemLocation::RemoteUrl(ref url) => gdal::Dataset::open(url),
        }.map_err(|e| {
            DemReaderError::Gdal(format!(
                "Could not open {} due to: {}",
                match desc.location {
                    DemLocation::LocalPath(ref path) => path.to_string_lossy(),
                    DemLocation::RemoteUrl(ref url) => url.as_str().into(),
                },
                e
            ))
        })?;
        let geo_transform = match dataset.geo_transform() {
            Ok(gt) => gt,
            Err(e) => {
                return Err(DemReaderError::Gdal(format!(
                    "Could not get geo transform from dataset due to: {}",
                    e,
                )));
            }
        };
        Ok(GdalDemHandle { dataset, geo_transform })
    }
}

pub struct GdalDemHandle {
    dataset: gdal::Dataset,
    geo_transform: [f64; 6],
}

fn coord_to_px(lon: f64, lat: f64, gt: &[f64; 6]) -> (isize, isize) {
    let x = ((lon - gt[0]) / gt[1]).floor() as isize;
    let y = ((lat - gt[3]) / gt[5]).floor() as isize;
    (x, y)
}

impl DemHandle for GdalDemHandle {
    fn elevation_at(&self, lat: f64, lon: f64) -> Result<super::Elevation, DemReaderError> {
        let band = match self.dataset.rasterband(1) {
            Ok(band) => band,
            Err(e) => {
                return Err(DemReaderError::Gdal(format!(
                    "Could not get raster band from dataset due to: {}",
                    e,
                )));
            }
        };
        let (px, py) = coord_to_px(lon, lat, &self.geo_transform);
        let buf = match band.read_as::<f32>((px, py), (1, 1), (1, 1), None) {
            Ok(buf) => buf,
            Err(e) => {
                return Err(DemReaderError::Gdal(format!(
                    "Could not read raster data at ({}, {}) due to: {}",
                    lat, lon, e,
                )));
            }
        };
        Ok(super::Elevation {
            height_m: buf.data()[0] as f64,
        })
    }
}
