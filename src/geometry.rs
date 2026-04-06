#[derive(Debug, Clone, Copy)]
pub struct Bbox {
    pub min_lat: f64,
    pub min_lon: f64,
    pub max_lat: f64,
    pub max_lon: f64,
}

impl Bbox {
    pub fn contains(&self, lat: f64, lon: f64) -> bool {
        lat >= self.min_lat && lat <= self.max_lat && lon >= self.min_lon && lon <= self.max_lon
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
