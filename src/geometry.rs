#[derive(Debug, Clone, Copy)]
pub struct Bbox {
    pub min_lat: f64,
    pub min_lon: f64,
    pub max_lat: f64,
    pub max_lon: f64,
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
