use crate::Elevation;

pub trait ElevationProvider {
    fn elevation_at(&self, lat: f64, lon: f64) -> Result<Elevation, anyhow::Error>;
}

pub struct LineOfSightService {
    elevation_provider: Box<dyn ElevationProvider>,
    max_step_degrees: f64,
}

impl LineOfSightService {
    pub fn new(elevation_provider: Box<dyn ElevationProvider>) -> Self {
        // TODO: configurable step size (based off DEM resolution?)
        LineOfSightService {
            elevation_provider,
            max_step_degrees: 1.0 / 3600.0 / 3.0, // 1/3 arcsecond in degrees
        }
    }

    pub fn has_line_of_sight(
        &self,
        lat1: f64,
        lon1: f64,
        lat2: f64,
        lon2: f64,
    ) -> Result<bool, anyhow::Error> {
        let elev1 = self.elevation_provider.elevation_at(lat1, lon1)?;
        let elev2 = self.elevation_provider.elevation_at(lat2, lon2)?;
        self.has_los_to_floating_point(lat1, lon1, elev1.m, lat2, lon2, elev2.m)
    }

    fn has_los_to_floating_point(
        &self,
        lat1: f64,
        lon1: f64,
        elev1_m: f64,
        lat2: f64,
        lon2: f64,
        elev2_m: f64,
    ) -> Result<bool, anyhow::Error> {
        if Self::degrees_between(lat1, lon1, lat2, lon2) <= self.max_step_degrees {
            // Base case: points are close enough to vacuously have LOS
            return Ok(true);
        }
        let (mid_lat, mid_lon) = Self::midpoint(lat1, lon1, lat2, lon2);
        let mid_elev = self.elevation_provider.elevation_at(mid_lat, mid_lon)?;

        // Check if the midpoint elevation is above the line connecting the two points
        let expected_mid_elev = (elev1_m + elev2_m) / 2.0;
        if mid_elev.m > expected_mid_elev {
            // Midpoint is above the line, so no LOS
            return Ok(false);
        }

        // Recurse on the two halves of the line, leveraging short-circuiting.
        Ok(self.has_los_to_floating_point(
            lat1,
            lon1,
            elev1_m,
            mid_lat,
            mid_lon,
            expected_mid_elev,
        )? && self.has_los_to_floating_point(
            mid_lat,
            mid_lon,
            expected_mid_elev,
            lat2,
            lon2,
            elev2_m,
        )?)
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

    #[test]
    fn test_same_point() {
        let service = LineOfSightService::new(Box::new(FlatElevation));
        assert!(service.has_line_of_sight(0.0, 0.0, 0.0, 0.0).unwrap());
    }

    #[test]
    fn test_los_short() {
        let service = LineOfSightService::new(Box::new(FlatElevation));
        assert!(service.has_line_of_sight(0.0, 0.0, 0.00001, 0.0).unwrap());
    }

    #[test]
    fn test_los_flat() {
        let service = LineOfSightService::new(Box::new(FlatElevation));
        assert!(service.has_line_of_sight(0.0, 0.0, 1.0, 1.0).unwrap());
    }

    #[test]
    fn test_los_wall() {
        let service = LineOfSightService::new(Box::new(WallElevation));
        assert!(!service.has_line_of_sight(-1.0, 0.0, 1.0, 0.0).unwrap());
        assert!(service.has_line_of_sight(-1.0, 0.0, -1.0, 1.0).unwrap());
        assert!(service.has_line_of_sight(1.0, 0.0, 1.0, 1.0).unwrap());
        assert!(service.has_line_of_sight(0.0, 0.0, 0.0, 1.0).unwrap());
    }

    #[test]
    fn test_los_valley() {
        let service = LineOfSightService::new(Box::new(ValleyElevation));
        assert!(service.has_line_of_sight(-1.0, 0.0, 1.0, 0.0).unwrap());
        assert!(service.has_line_of_sight(-1.0, 0.0, 0.5, 1.0).unwrap());
    }

    #[test]
    fn test_los_ramp() {
        let service = LineOfSightService::new(Box::new(RampElevation));
        // LOS from top of ramp to distance should be clear, since the ramp is always below the
        // line connecting the two points.
        assert!(service.has_line_of_sight(0.0, 0.0, 2.0, 0.0).unwrap());

        // LOS from behind edge of ramp to bottom of ramp should be blocked, since the ramp is
        // above the line connecting the two points.
        assert!(!service.has_line_of_sight(-1.0, 0.0, 1.0, 0.0).unwrap());
    }
}
