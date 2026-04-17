#[derive(Debug, Clone, Copy)]
pub struct Bbox {
    pub min_lat: f64,
    pub min_lon: f64,
    pub max_lat: f64,
    pub max_lon: f64,
}

impl std::fmt::Display for Bbox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Bbox(min_lat: {}, min_lon: {}, max_lat: {}, max_lon: {})",
            self.min_lat, self.min_lon, self.max_lat, self.max_lon
        )
    }
}

impl Bbox {
    pub fn contains(&self, lat: f64, lon: f64) -> bool {
        lat >= self.min_lat && lat <= self.max_lat && lon >= self.min_lon && lon <= self.max_lon
    }

    pub fn width(&self) -> f64 {
        self.max_lon - self.min_lon
    }

    pub fn height(&self) -> f64 {
        self.max_lat - self.min_lat
    }

    pub fn with_margin(&self, margin: f64) -> Self {
        Bbox {
            min_lat: self.min_lat - margin,
            min_lon: self.min_lon - margin,
            max_lat: self.max_lat + margin,
            max_lon: self.max_lon + margin,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Elevation {
    pub m: f64,
    pub ft: f64,
}

impl Elevation {
    pub fn from_m(m: f64) -> Self {
        Elevation { m, ft: m * 3.28084 }
    }
}
