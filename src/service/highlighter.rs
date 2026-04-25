use super::los::ViewshedGrid;
use crate::reader::gdal::GeoPixelMapper;
use crate::source::Location;
use crate::source::topo::TopoMapDescriptor;
use gdal::Dataset;
use gdal::spatial_ref::{CoordTransform, SpatialRef};
use image::{Rgba, RgbaImage};

pub struct HighlighterService {
    image_dpi: u32,
}

impl Default for HighlighterService {
    fn default() -> Self {
        HighlighterService { image_dpi: 200 }
    }
}

impl HighlighterService {
    pub fn new(image_dpi: u32) -> Self {
        HighlighterService { image_dpi }
    }

    #[allow(clippy::too_many_arguments)] // Private helper, clippy can relax.
    fn bilinearly_interpolate(
        &self,
        row: usize,
        col: usize,
        top_left: (f64, f64),
        top_right: (f64, f64),
        bottom_left: (f64, f64),
        bottom_right: (f64, f64),
        width: usize,
        height: usize,
    ) -> (f64, f64) {
        let x_ratio = col as f64 / (width - 1) as f64;
        let y_ratio = row as f64 / (height - 1) as f64;

        let lat_top = top_left.0 * (1.0 - x_ratio) + top_right.0 * x_ratio;
        let lon_top = top_left.1 * (1.0 - x_ratio) + top_right.1 * x_ratio;
        let lat_bottom = bottom_left.0 * (1.0 - x_ratio) + bottom_right.0 * x_ratio;
        let lon_bottom = bottom_left.1 * (1.0 - x_ratio) + bottom_right.1 * x_ratio;

        let lat = lat_top * (1.0 - y_ratio) + lat_bottom * y_ratio;
        let lon = lon_top * (1.0 - y_ratio) + lon_bottom * y_ratio;

        (lat, lon)
    }

    pub fn highlight_viewshed(
        &self,
        topo_map: &TopoMapDescriptor,
        viewshed: &ViewshedGrid,
    ) -> anyhow::Result<RgbaImage> {
        // TODO: Ability to use different rasterization libraries in case of non-PDF map.
        //       May want fully different highlight implementations for different libraries.

        gdal::config::set_config_option("GDAL_PDF_DPI", &self.image_dpi.to_string())?;
        gdal::config::set_config_option("CPL_LOG", "/dev/null")?;

        // Step 1: Get topo map dataset and geotransform
        println!(
            "Rasterizing topo map from location: {:?}...",
            topo_map.location
        );
        let dataset = match &topo_map.location {
            Location::LocalPath(path) => Dataset::open(path)?,
            Location::RemoteUrl(url) => Dataset::open(url)?,
        };
        let gt = dataset.geo_transform()?;
        let wgs84 = SpatialRef::from_epsg(4326)?;
        let dataset_srs = dataset.spatial_ref()?;
        let wgs84_to_dataset_srs = CoordTransform::new(&wgs84, &dataset_srs)?;
        let dataset_to_wgs84 = CoordTransform::new(&dataset_srs, &wgs84)?;
        let geo_pixel_mapper = GeoPixelMapper::new(gt, dataset_to_wgs84, wgs84_to_dataset_srs);
        println!(
            "Working on topo map with size {}x{}...",
            dataset.raster_size().0,
            dataset.raster_size().1
        );

        // Step 2: Get the RGB bands and hold them in memory.
        println!("Reading RGB bands from topo map into memory...");
        let mut image = RgbaImage::new(
            dataset.raster_size().0 as u32,
            dataset.raster_size().1 as u32,
        );
        let red_band = dataset.rasterband(1)?;
        let red_data =
            red_band.read_as::<u8>((0, 0), dataset.raster_size(), dataset.raster_size(), None)?;
        let green_band = dataset.rasterband(2)?;
        let green_data =
            green_band.read_as::<u8>((0, 0), dataset.raster_size(), dataset.raster_size(), None)?;
        let blue_band = dataset.rasterband(3)?;
        let blue_data =
            blue_band.read_as::<u8>((0, 0), dataset.raster_size(), dataset.raster_size(), None)?;

        // Step 3: For each pixel, determine if it's visible in the viewshed. If not, darken the pixel.
        println!("Applying viewshed to topo map...");
        let darken_factor = 0.6; // How much to darken non-visible pixels (0.0 = completely black, 1.0 = no change)
        let raster_size = dataset.raster_size();
        let tl = geo_pixel_mapper.pixel_to_lat_lon(0, 0)?;
        let tr = geo_pixel_mapper.pixel_to_lat_lon(raster_size.0 as isize - 1, 0)?;
        let bl = geo_pixel_mapper.pixel_to_lat_lon(0, raster_size.1 as isize - 1)?;
        let br = geo_pixel_mapper
            .pixel_to_lat_lon(raster_size.0 as isize - 1, raster_size.1 as isize - 1)?;
        for col in 0..raster_size.0 {
            for row in 0..raster_size.1 {
                let (lat, lon) = self.bilinearly_interpolate(
                    row,
                    col,
                    tl,
                    tr,
                    bl,
                    br,
                    raster_size.0,
                    raster_size.1,
                );
                let mut r = red_data[(row, col)];
                let mut g = green_data[(row, col)];
                let mut b = blue_data[(row, col)];
                if let Some(false) = viewshed.has_line_of_sight(lat, lon) {
                    // Not visible - darken the pixel.
                    r = (r as f64 * darken_factor).round() as u8;
                    g = (g as f64 * darken_factor).round() as u8;
                    b = (b as f64 * darken_factor).round() as u8;
                }

                image.put_pixel(col as u32, row as u32, Rgba([r, g, b, 255]));
            }
        }
        // Step 4: Put origin dot on map
        println!("Marking origin point on map...");
        let (origin_x, origin_y) =
            geo_pixel_mapper.lat_lon_to_pixel(viewshed.origin_lat, viewshed.origin_lon)?;

        let r = 10; // radius of origin dot in pixels
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r * r {
                    let x = origin_x + dx;
                    let y = origin_y + dy;
                    if x >= 0
                        && x < dataset.raster_size().0 as isize
                        && y >= 0
                        && y < dataset.raster_size().1 as isize
                    {
                        image.put_pixel(x as u32, y as u32, Rgba([255, 0, 0, 255]));
                    }
                }
            }
        }

        Ok(image)
    }
}
