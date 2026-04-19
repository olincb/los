use crate::{Bbox, Elevation};
use std::fmt;

pub trait ElevationProvider {
    fn elevation_at(&self, lat: f64, lon: f64) -> Result<Elevation, anyhow::Error>;
}

pub enum LineOfSightResult {
    Clear,
    Blocked {
        lat: f64,
        lon: f64,
        terrain_m: f64,
        sightline_m: f64,
    },
}

impl LineOfSightResult {
    pub fn is_clear(&self) -> bool {
        matches!(self, LineOfSightResult::Clear)
    }
    pub fn is_blocked(&self) -> bool {
        !self.is_clear()
    }
}

pub struct ViewshedGrid {
    pub origin_lat: f64, // Viewer's latitude.
    pub origin_lon: f64, // Viewer's longitude.
    pub width: usize,
    pub height: usize,
    pub bbox: Bbox,
    pub data: Vec<bool>, // Row-major grid of whether each point is visible from the origin.
}

impl fmt::Display for ViewshedGrid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "ViewshedGrid (origin: ({}, {}), bbox: {}",
            self.origin_lat, self.origin_lon, self.bbox
        )?;
        let viewer_y_degrees = self.bbox.max_lat - self.origin_lat;
        let viewer_x_degrees = self.origin_lon - self.bbox.min_lon;
        let viewer_y_proportion = viewer_y_degrees / self.bbox.height();
        let viewer_x_proportion = viewer_x_degrees / self.bbox.width();
        let viewer_y = (viewer_y_proportion * (self.height - 1) as f64).round() as usize;
        let viewer_x = (viewer_x_proportion * (self.width - 1) as f64).round() as usize;
        for y in 0..self.height {
            for x in 0..self.width {
                let is_viewer = x == viewer_x && y == viewer_y;
                let is_visible = self.data[y * self.width + x];
                let ch = if is_viewer {
                    'ಠ'
                } else if is_visible {
                    '●'
                } else {
                    '◯'
                };
                write!(f, "{}", ch)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl ViewshedGrid {
    pub fn has_line_of_sight(&self, lat: f64, lon: f64) -> Option<bool> {
        if !self.bbox.contains(lat, lon) {
            return None;
        }
        let y_proportion = (self.bbox.max_lat - lat) / self.bbox.height();
        let x_proportion = (lon - self.bbox.min_lon) / self.bbox.width();
        let y = (y_proportion * (self.height - 1) as f64).round() as usize;
        let x = (x_proportion * (self.width - 1) as f64).round() as usize;
        Some(self.data[y * self.width + x])
    }
}

pub struct LineOfSightService {
    elevation_provider: Box<dyn ElevationProvider>,
    max_step_degrees: f64,
    tolerance_m: f64, // How much higher the terrain must be than the sightline
                      // to be considered a blocker, in meters. This is to account for noise in
                      // the elevation data and to avoid false positives where the terrain is
                      // just barely above the sightline.
}

impl LineOfSightService {
    pub fn new(elevation_provider: Box<dyn ElevationProvider>) -> Self {
        // TODO: configurable step size (based off DEM resolution?)
        LineOfSightService {
            elevation_provider,
            max_step_degrees: 1.0 / 3600.0 / 3.0, // 1/3 arcsecond in degrees
            tolerance_m: 2.0,                     // USGS 3DEP has ~1-2m vertical accuracy
        }
    }

    pub fn has_line_of_sight(
        &self,
        lat1: f64,
        lon1: f64,
        lat2: f64,
        lon2: f64,
    ) -> Result<LineOfSightResult, anyhow::Error> {
        self.has_line_of_sight_with_height(lat1, lon1, lat2, lon2, 0.0)
    }

    pub fn has_line_of_sight_with_height(
        &self,
        lat1: f64,
        lon1: f64,
        lat2: f64,
        lon2: f64,
        viewer_height_m: f64,
    ) -> Result<LineOfSightResult, anyhow::Error> {
        let elev1 = self.elevation_provider.elevation_at(lat1, lon1)?;
        let elev2 = self.elevation_provider.elevation_at(lat2, lon2)?;
        let adjusted_elev1_m = elev1.m + viewer_height_m;
        self.has_los_to_floating_point(lat1, lon1, adjusted_elev1_m, lat2, lon2, elev2.m)
    }

    /// Computes the viewshed grid for a given viewer location and bounding box.
    /// `max_resolution_degrees` specifies the maximum resolution of the grid in degrees. The
    /// actual resolution may be slightly higher to maintain a consistent resolution in meters
    /// across the viewshed, but it will not be lower than this value. `viewer_height_m` specifies
    /// the height of the viewer above the ground in meters, which is added to the elevation at the
    /// viewer's location when calculating line of sight. If `viewer_height_m` is None, it defaults
    /// to 0 (i.e. the viewer is at ground level).
    /// The resolution in degrees is applied as-is to the latitude direction, but is adjusted for
    /// the longitude direction based on the viewer's latitude to maintain a consistent resolution
    /// in meters across the viewshed.
    pub fn viewshed(
        &self,
        lat: f64,
        lon: f64,
        bbox: Bbox,
        max_resolution_degrees: f64,
        viewer_height_m: Option<f64>,
    ) -> Result<ViewshedGrid, anyhow::Error> {
        if max_resolution_degrees <= 0.0 {
            return Err(anyhow::anyhow!(
                "Max resolution degrees must be strictly positive, got {}",
                max_resolution_degrees
            ));
        }
        let lon_step = max_resolution_degrees / lat.to_radians().cos();
        let cols = (bbox.width() / lon_step).ceil() as usize + 1;
        let rows = (bbox.height() / max_resolution_degrees).ceil() as usize + 1;
        self.viewshed_for_grid(lat, lon, bbox, cols, rows, viewer_height_m)
    }

    pub fn viewshed_for_grid(
        &self,
        lat: f64,
        lon: f64,
        bbox: Bbox,
        cols: usize,
        rows: usize,
        viewer_height_m: Option<f64>,
    ) -> Result<ViewshedGrid, anyhow::Error> {
        if !bbox.contains(lat, lon) {
            return Err(anyhow::anyhow!(
                "Viewer location ({}, {}) is outside of viewshed bounding box: {:?}",
                lat,
                lon,
                bbox
            ));
        }
        let vh = viewer_height_m.unwrap_or(0.0);
        let lon_step = bbox.width() / (cols - 1) as f64;
        let lat_step = bbox.height() / (rows - 1) as f64;

        let mut data = Vec::with_capacity(rows * cols);
        for y in 0..rows {
            let point_lat = bbox.max_lat - y as f64 * lat_step;
            for x in 0..cols {
                let point_lon = bbox.min_lon + x as f64 * lon_step;
                let los = self.has_line_of_sight_with_height(lat, lon, point_lat, point_lon, vh)?;
                data.push(los.is_clear());
            }
        }
        Ok(ViewshedGrid {
            origin_lat: lat,
            origin_lon: lon,
            width: cols,
            height: rows,
            bbox,
            data,
        })
    }

    fn has_los_to_floating_point(
        &self,
        lat1: f64,
        lon1: f64,
        elev1_m: f64,
        lat2: f64,
        lon2: f64,
        elev2_m: f64,
    ) -> Result<LineOfSightResult, anyhow::Error> {
        if Self::degrees_between(lat1, lon1, lat2, lon2) <= self.max_step_degrees {
            // Base case: points are close enough to vacuously have LOS
            return Ok(LineOfSightResult::Clear);
        }
        let (mid_lat, mid_lon) = Self::midpoint(lat1, lon1, lat2, lon2);
        let mid_elev = self.elevation_provider.elevation_at(mid_lat, mid_lon)?;

        // Check if the midpoint elevation is above the line connecting the two points
        let expected_mid_elev = (elev1_m + elev2_m) / 2.0;
        if mid_elev.m > expected_mid_elev + self.tolerance_m {
            // Midpoint is above the line, so no LOS
            return Ok(LineOfSightResult::Blocked {
                lat: mid_lat,
                lon: mid_lon,
                terrain_m: mid_elev.m,
                sightline_m: expected_mid_elev,
            });
        }

        // Recurse on the two halves of the line, short-circuiting if possible.
        match self.has_los_to_floating_point(
            lat1,
            lon1,
            elev1_m,
            mid_lat,
            mid_lon,
            expected_mid_elev,
        )? {
            LineOfSightResult::Clear => self.has_los_to_floating_point(
                mid_lat,
                mid_lon,
                expected_mid_elev,
                lat2,
                lon2,
                elev2_m,
            ),
            blocked @ LineOfSightResult::Blocked { .. } => Ok(blocked),
        }
    }

    fn midpoint(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> (f64, f64) {
        ((lat1 + lat2) / 2.0, (lon1 + lon2) / 2.0)
    }

    fn degrees_between(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        let dlat = lat2 - lat1;
        let dlon = lon2 - lon1;
        (dlat.powi(2) + dlon.powi(2)).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Always returns the same elevation, regardless of lat/lon.
    struct FlatElevation;
    impl ElevationProvider for FlatElevation {
        fn elevation_at(&self, _lat: f64, _lon: f64) -> Result<Elevation, anyhow::Error> {
            Ok(Elevation::from_m(100.0))
        }
    }

    /// Puts a 1-degree-wide wall at the equator, doubling the elevation
    struct WallElevation;
    impl ElevationProvider for WallElevation {
        fn elevation_at(&self, lat: f64, _lon: f64) -> Result<Elevation, anyhow::Error> {
            if lat.abs() < 0.5 {
                Ok(Elevation::from_m(1000.0))
            } else {
                Ok(Elevation::from_m(100.0))
            }
        }
    }

    /// Equator has a v-shaped valley, with the lowest point at the equator itself.
    struct ValleyElevation;
    impl ElevationProvider for ValleyElevation {
        fn elevation_at(&self, lat: f64, _lon: f64) -> Result<Elevation, anyhow::Error> {
            Ok(Elevation::from_m(100.0 + lat.abs() * 1000.0))
        }
    }

    /// Ramp function that is 100m below the equator and 0m at 1 degree north, with a linear slope
    /// in between.
    struct RampElevation;
    impl ElevationProvider for RampElevation {
        fn elevation_at(&self, lat: f64, _lon: f64) -> Result<Elevation, anyhow::Error> {
            match lat {
                l if l < 0.0 => Ok(Elevation::from_m(100.0)),
                l if l > 1.0 => Ok(Elevation::from_m(0.0)),
                _ => Ok(Elevation::from_m(100.0 - lat * 100.0)),
            }
        }
    }

    fn basic_box() -> Bbox {
        Bbox {
            min_lat: -1.0,
            max_lat: 1.0,
            min_lon: -1.0,
            max_lon: 1.0,
        }
    }

    #[test]
    fn test_same_point() {
        let service = LineOfSightService::new(Box::new(FlatElevation));
        assert!(
            service
                .has_line_of_sight(0.0, 0.0, 0.0, 0.0)
                .unwrap()
                .is_clear()
        );
    }

    #[test]
    fn test_los_short() {
        let service = LineOfSightService::new(Box::new(FlatElevation));
        assert!(
            service
                .has_line_of_sight(0.0, 0.0, 0.00001, 0.0)
                .unwrap()
                .is_clear()
        );
    }

    #[test]
    fn test_los_flat() {
        let service = LineOfSightService::new(Box::new(FlatElevation));
        assert!(
            service
                .has_line_of_sight(0.0, 0.0, 1.0, 1.0)
                .unwrap()
                .is_clear()
        );
    }

    #[test]
    fn test_los_wall() {
        let service = LineOfSightService::new(Box::new(WallElevation));
        assert!(
            service
                .has_line_of_sight(-1.0, 0.0, 1.0, 0.0)
                .unwrap()
                .is_blocked()
        );
        assert!(
            service
                .has_line_of_sight(-1.0, 0.0, -1.0, 1.0)
                .unwrap()
                .is_clear()
        );
        assert!(
            service
                .has_line_of_sight(1.0, 0.0, 1.0, 1.0)
                .unwrap()
                .is_clear()
        );
        assert!(
            service
                .has_line_of_sight(0.0, 0.0, 0.0, 1.0)
                .unwrap()
                .is_clear()
        );
    }

    #[test]
    fn test_los_valley() {
        let service = LineOfSightService::new(Box::new(ValleyElevation));
        assert!(
            service
                .has_line_of_sight(-1.0, 0.0, 1.0, 0.0)
                .unwrap()
                .is_clear()
        );
        assert!(
            service
                .has_line_of_sight(-1.0, 0.0, 0.5, 1.0)
                .unwrap()
                .is_clear()
        );
    }

    #[test]
    fn test_los_ramp() {
        let service = LineOfSightService::new(Box::new(RampElevation));
        // LOS from top of ramp to distance should be clear, since the ramp is always below the
        // line connecting the two points.
        assert!(
            service
                .has_line_of_sight(0.0, 0.0, 2.0, 0.0)
                .unwrap()
                .is_clear()
        );

        // LOS from behind edge of ramp to bottom of ramp should be blocked, since the ramp is
        // above the line connecting the two points.
        assert!(
            service
                .has_line_of_sight(-1.0, 0.0, 1.0, 0.0)
                .unwrap()
                .is_blocked()
        );
    }

    #[test]
    fn test_viewshed_dimensions() {
        let service = LineOfSightService::new(Box::new(FlatElevation));
        let viewshed = service
            .viewshed(0.75, 0.0, basic_box(), 0.05, None)
            .unwrap();
        assert_eq!(viewshed.width, 41);
        assert_eq!(viewshed.height, 41);
    }

    #[test]
    fn test_viewshed_out_of_bounds() {
        let service = LineOfSightService::new(Box::new(FlatElevation));
        let result = service.viewshed(2.0, 0.0, basic_box(), 0.05, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_viewshed_invalid_resolution() {
        let service = LineOfSightService::new(Box::new(FlatElevation));
        let result = service.viewshed(0.75, 0.0, basic_box(), -0.05, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_viewshed_flat() {
        let service = LineOfSightService::new(Box::new(FlatElevation));
        let viewshed = service
            .viewshed(0.75, 0.0, basic_box(), 0.05, None)
            .unwrap();
        for cell in viewshed.data {
            assert!(cell); // All points should be visible in a flat world
        }
    }

    #[test]
    fn test_viewshed_wall() {
        let service = LineOfSightService::new(Box::new(WallElevation));
        let viewshed = service
            .viewshed(0.75, 0.0, basic_box(), 0.05, None)
            .unwrap();
        assert!(viewshed.data[0]); // Top-left corner should be visible
        assert!(viewshed.data[40]); // Top-right corner should be visible
        assert!(!viewshed.data[40 * 41]); // Bottom-left corner should be obscured by the wall
        assert!(!viewshed.data[40 * 41 + 40]); // Bottom-right corner should be obscured by the wall
    }
}
