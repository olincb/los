pub mod gdal;
pub mod geotiff;
pub mod traits;

pub use gdal::GdalReader;
pub use geotiff::GeoTiffReader;
pub use traits::{DemHandle, DemReader, DemReaderError};
