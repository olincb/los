use super::common::{ReaderType, SourceType, build_elevation_service};
use los::orchestrator::highlight;
use los::source::topo::UsgsTopoMapSource;
use std::path::PathBuf;

pub fn handle_highlight_command(lat: f64, lon: f64, output: PathBuf) -> anyhow::Result<()> {
    // TODO: reader and source selection should be configurable for this command as well, but for
    // now we'll just hardcode it to use USGS topo maps and GDAL reader.
    let t0 = std::time::Instant::now();
    let map_source = UsgsTopoMapSource::fetch()?;
    let elevation_service =
        build_elevation_service(ReaderType::Gdal, SourceType::Usgs, None, None)?;
    let viewer_height_m = 3.0;
    let resolution_deg = 0.0001;
    let highlighted_map = highlight(
        lat,
        lon,
        viewer_height_m,
        resolution_deg,
        elevation_service,
        &map_source,
    )?;
    println!("Saving highlighted map to {}", output.to_string_lossy());
    highlighted_map.save(&output)?;
    println!("Saved highlighted map to {}", output.to_string_lossy());
    println!("Total time: {:.3}s", t0.elapsed().as_secs_f32(),);

    Ok(())
}
