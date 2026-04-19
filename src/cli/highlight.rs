use super::common::{ReaderType, SourceType, build_elevation_service};
use los::service::{HighlighterService, LineOfSightService};
use los::source::topo::{TopoSource, UsgsTopoMapSource};
use std::path::PathBuf;

pub fn handle_highlight_command(lat: f64, lon: f64, output: PathBuf) -> anyhow::Result<()> {
    // TODO: reader and source selection should be configurable for this command as well, but for
    // now we'll just hardcode it to use USGS topo maps and GDAL reader.
    println!(
        "Highlighting visible area on topo map for ({}, {}) and saving to {}",
        lat,
        lon,
        output.to_string_lossy()
    );
    println!("Fetching topo map from USGS...");
    let t0 = std::time::Instant::now();
    let map_source = UsgsTopoMapSource::fetch()?;
    let map_descriptor = map_source.get_map_descriptor(lat, lon)?;
    let bbox = map_descriptor.bbox;
    let map_path = map_source.fetch_map(&map_descriptor)?;
    println!("Fetched topo map from USGS: {}", map_path.to_string_lossy());
    println!(
        "{:.3}s ({:.3}s total)",
        t0.elapsed().as_secs_f32(),
        t0.elapsed().as_secs_f32()
    );
    println!("Prefetching elevation data for map bounding box...");
    let t = std::time::Instant::now();
    let mut elevation_service =
        build_elevation_service(ReaderType::Gdal, SourceType::Usgs, None, None)?;
    elevation_service.prefetch_region(&bbox)?;
    println!("Prefetched elevation data for map bounding box: {:?}", bbox);
    println!(
        "{:.3}s ({:.3}s total)",
        t.elapsed().as_secs_f32(),
        t0.elapsed().as_secs_f32()
    );
    println!("Calculating viewshed...");
    let t = std::time::Instant::now();
    let los_service = LineOfSightService::new(Box::new(elevation_service));
    let viewer_height_m = 3.0; // They're on their tip toes.
    let resolution_deg = 0.0001; // TODO: make configurable
    let viewshed_result =
        los_service.viewshed(lat, lon, bbox, resolution_deg, Some(viewer_height_m))?;
    println!(
        "Calculated viewshed with resolution {} degrees, resulting in grid of {} cols x {} rows.",
        resolution_deg, viewshed_result.width, viewshed_result.height
    );
    println!(
        "{:.3}s ({:.3}s total)",
        t.elapsed().as_secs_f32(),
        t0.elapsed().as_secs_f32()
    );
    println!("Highlighting viewshed on topo map...");
    let t = std::time::Instant::now();
    let highlighter = HighlighterService::default();
    let highlighted_map = highlighter.highlight_viewshed(&map_descriptor, &viewshed_result)?;
    println!("Highlighted viewshed on topo map.");
    println!(
        "{:.3}s ({:.3}s total)",
        t.elapsed().as_secs_f32(),
        t0.elapsed().as_secs_f32()
    );
    println!("Saving highlighted map to {}", output.to_string_lossy());
    highlighted_map.save(&output)?;
    println!("Saved highlighted map to {}", output.to_string_lossy());
    println!("Total time: {:.3}s", t0.elapsed().as_secs_f32(),);

    Ok(())
}
