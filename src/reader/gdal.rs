use crate::reader::{DemHandle, DemReader, DemReaderError};
use crate::source::Location;
use crate::{Bbox, Elevation};
use gdal::spatial_ref::CoordTransform;

pub struct GdalReader;

impl DemReader for GdalReader {
    fn open(&self, loc: &Location, bbox: Bbox) -> Result<Box<dyn DemHandle>, DemReaderError> {
        // GDAL can handle both local paths and remote URLs.
        let dataset = match loc {
            Location::LocalPath(path) => gdal::Dataset::open(path),
            Location::RemoteUrl(url) => gdal::Dataset::open(url),
        }
        .map_err(|e| {
            DemReaderError::Gdal(format!(
                "Could not open {} due to: {}",
                match loc {
                    Location::LocalPath(path) => path.to_string_lossy(),
                    Location::RemoteUrl(url) => url.as_str().into(),
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
        Ok(Box::new(self.prefetch_region(
            bbox,
            &dataset,
            geo_transform,
        )?))
    }
}

impl GdalReader {
    fn prefetch_region(
        &self,
        bbox: Bbox,
        dataset: &gdal::Dataset,
        geo_transform: [f64; 6],
    ) -> Result<GdalDemHandle, DemReaderError> {
        let band = match dataset.rasterband(1) {
            Ok(band) => band,
            Err(e) => {
                return Err(DemReaderError::Gdal(format!(
                    "Could not get raster band from dataset due to: {}",
                    e,
                )));
            }
        };
        let (origin_px, origin_py) = geo_coord_to_pixel(bbox.min_lon, bbox.max_lat, &geo_transform);
        let (max_px, max_py) = geo_coord_to_pixel(bbox.max_lon, bbox.min_lat, &geo_transform);
        // If width or height are 0, it means the bbox is smaller than the pixel size of
        // the dataset, so we need to read at least 1 pixel in each dimension to get an
        // elevation value.
        let width = (max_px - origin_px).max(1) as usize;
        let height = (max_py - origin_py).max(1) as usize;
        let (raster_width, raster_height) = dataset.raster_size();
        if origin_px < 0
            || origin_py < 0
            || (origin_px as usize) + width > raster_width
            || (origin_py as usize) + height > raster_height
        {
            return Err(DemReaderError::OutOfBounds(format!(
                "Calculated pixel coordinates for bbox are out of bounds of the dataset raster size (width: {}, height: {}). Calculated origin pixel: ({}, {}), max pixel: ({}, {}).",
                raster_width, raster_height, origin_px, origin_py, max_px, max_py,
            )));
        }
        let buf = match band.read_as::<f32>(
            (origin_px, origin_py),
            (width, height),
            (width, height),
            None,
        ) {
            Ok(buf) => buf,
            Err(e) => {
                return Err(DemReaderError::Gdal(format!(
                    "Could not read raster data for prefetch region due to: {}",
                    e,
                )));
            }
        };
        // TODO: Use disk to cache prefetched regions by bbox, to avoid repeated fetching and
        // reading of the same data across multiple runs of the program.
        Ok(GdalDemHandle {
            geo_transform,
            bbox,
            width,
            height,
            data: buf.data().to_vec(),
        })
    }
}

pub struct GdalDemHandle {
    geo_transform: [f64; 6],
    bbox: Bbox,
    width: usize,
    height: usize,
    data: Vec<f32>,
}

pub struct GeoPixelMapper {
    gt: [f64; 6],
    dataset_to_wgs84: CoordTransform,
    wgs84_to_dataset: CoordTransform,
}

impl GeoPixelMapper {
    pub fn new(
        gt: [f64; 6],
        dataset_to_wgs84: CoordTransform,
        wgs84_to_dataset: CoordTransform,
    ) -> Self {
        GeoPixelMapper {
            gt,
            dataset_to_wgs84,
            wgs84_to_dataset,
        }
    }

    /// Converts pixel coordinates to lat/lon by first converting to georeferenced coordinates using the
    /// GDAL geotransform, and then transforming to lat/lon using the appropriate coordinate transform.
    pub fn pixel_to_lat_lon(&self, x: isize, y: isize) -> anyhow::Result<(f64, f64)> {
        let (geo_x, geo_y) = pixel_to_geo_coord(x, y, &self.gt);
        apply_coord_transform(geo_x, geo_y, &self.dataset_to_wgs84)
    }

    /// Converts lat/lon to pixel coordinates by first transforming to georeferenced coordinates using the
    /// appropriate coordinate transform, and then converting to pixel coordinates using the GDAL geotransform.
    pub fn lat_lon_to_pixel(&self, lat: f64, lon: f64) -> anyhow::Result<(isize, isize)> {
        let (geo_x, geo_y) = apply_coord_transform(lat, lon, &self.wgs84_to_dataset)?;
        Ok(geo_coord_to_pixel(geo_x, geo_y, &self.gt))
    }
}

/// Converts pixel coordinates to georeferenced coordinates using the GDAL geotransform.
/// The output may need further transformation to lat/lon depending on the dataset's CRS.
fn pixel_to_geo_coord(x: isize, y: isize, gt: &[f64; 6]) -> (f64, f64) {
    let geo_coord_x = gt[0] + x as f64 * gt[1] + y as f64 * gt[2];
    let geo_coord_y = gt[3] + x as f64 * gt[4] + y as f64 * gt[5];
    (geo_coord_x, geo_coord_y)
}

/// Converts georeferenced coordinates to pixel coordinates using the GDAL geotransform.
fn geo_coord_to_pixel(geo_x: f64, geo_y: f64, gt: &[f64; 6]) -> (isize, isize) {
    let det = gt[1] * gt[5] - gt[2] * gt[4];
    let x = ((gt[5] * (geo_x - gt[0]) - gt[2] * (geo_y - gt[3])) / det).floor() as isize;
    let y = ((gt[1] * (geo_y - gt[3]) - gt[4] * (geo_x - gt[0])) / det).floor() as isize;
    (x, y)
}

/// Applies a CoordTransform to a single (x, y) point.
///
/// IMPORTANT: GDAL CoordTransform uses axis order defined by the CRS.
/// For EPSG:4326 (WGS84), x = latitude, y = longitude (not the
/// intuitive order). After transform, the output axes follow the
/// *target* CRS axis order.
pub fn apply_coord_transform(
    x: f64,
    y: f64,
    transform: &CoordTransform,
) -> anyhow::Result<(f64, f64)> {
    let mut xs = [x];
    let mut ys = [y];
    transform.transform_coords(&mut xs, &mut ys, &mut [])?;
    Ok((xs[0], ys[0]))
}

impl DemHandle for GdalDemHandle {
    fn elevation_at(&self, lat: f64, lon: f64) -> Result<Elevation, DemReaderError> {
        if !self.bbox.contains(lat, lon) {
            return Err(DemReaderError::OutOfBounds(format!(
                "Coordinates ({}, {}) are out of bounds for prefetched region with bbox {}.",
                lat, lon, self.bbox,
            )));
        }
        let (px, py) = geo_coord_to_pixel(lon, lat, &self.geo_transform);
        let (origin_px, origin_py) =
            geo_coord_to_pixel(self.bbox.min_lon, self.bbox.max_lat, &self.geo_transform);
        let local_px = px - origin_px;
        let local_py = py - origin_py;
        if local_px < 0
            || local_py < 0
            || (local_px as usize) >= self.width
            || (local_py as usize) >= self.height
        {
            return Err(DemReaderError::OutOfBounds(format!(
                "Calculated local pixel coordinates ({}, {}) are out of bounds for prefetched region with width {} and height {}.",
                local_px, local_py, self.width, self.height,
            )));
        }
        let idx = (local_py as usize) * self.width + (local_px as usize);
        if idx >= self.data.len() {
            return Err(DemReaderError::OutOfBounds(format!(
                "Calculated index {} is out of bounds for prefetched data with length {}.",
                idx,
                self.data.len()
            )));
        }
        Ok(Elevation::from_m(self.data[idx] as f64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_handle() -> GdalDemHandle {
        // 4x3 grid, origin at (-90.0, 45.0), 1 degree per pixel
        GdalDemHandle {
            geo_transform: [-90.0, 1.0, 0.0, 45.0, 0.0, -1.0],
            bbox: Bbox {
                min_lon: -90.0,
                max_lon: -86.0,
                min_lat: 42.0,
                max_lat: 45.0,
            },
            width: 4,
            height: 3,
            data: vec![
                100.0, 200.0, 300.0, 400.0, 500.0, 600.0, 700.0, 800.0, 900.0, 1000.0, 1100.0,
                1200.0,
            ],
        }
    }

    #[test]
    fn cache_hit() {
        let handle = make_test_handle();
        let elev = handle.elevation_at(44.5, -89.5);
        assert!(elev.is_ok());
    }

    #[test]
    fn cache_miss_out_of_bounds() {
        let handle = make_test_handle();
        let elev = handle.elevation_at(50.0, -89.0);
        assert!(elev.is_err());
    }
}
