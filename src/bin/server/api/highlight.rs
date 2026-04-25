use image::RgbaImage;
use los::orchestrator::highlight;
use los::reader::GdalReader;
use los::service::ElevationService;
use los::source;

pub fn handle_highlight_endpoint_command(lat: f64, lon: f64) -> anyhow::Result<RgbaImage> {
    let dem_source = Box::new(source::dem::UsgsSource);
    let reader = Box::new(GdalReader);
    let elevation_service = ElevationService::new(dem_source, reader, None);
    let map_source = source::topo::UsgsTopoMapSource::fetch()?;
    let viewer_height_m = 3.0;
    let resolution_deg = 0.0001;
    highlight(
        lat,
        lon,
        viewer_height_m,
        resolution_deg,
        elevation_service,
        &map_source,
    )
}
