/// Primary orchestration which glues services together and provides a single
/// entry point for the application, regardless of the originating source of
/// the request (e.g., CLI, API, etc.).
use crate::ElevationService;
use crate::service::{HighlighterService, LineOfSightService};
use crate::source::Location;
use crate::source::topo::{TopoMapDescriptor, TopoSource};
use image::RgbaImage;

pub fn highlight(
    lat: f64,
    lon: f64,
    viewer_height_m: f64,
    resolution_deg: f64,
    mut elevation_service: ElevationService,
    map_source: &dyn TopoSource,
) -> anyhow::Result<RgbaImage> {
    println!(
        "Highlighting visible area on topo map for ({}, {})",
        lat, lon
    );
    println!("Fetching topo map...");
    let t0 = std::time::Instant::now();
    let map_descriptor = map_source.get_map_descriptor(lat, lon)?;
    let local_path = map_source.fetch_map(&map_descriptor)?;
    let map_descriptor = TopoMapDescriptor {
        location: Location::LocalPath(local_path),
        ..map_descriptor
    };
    let bbox = map_descriptor.bbox;
    println!(
        "Fetched topo map and cached locally: {:?}",
        map_descriptor.location
    );
    println!(
        "{:.3}s ({:.3}s total)",
        t0.elapsed().as_secs_f32(),
        t0.elapsed().as_secs_f32()
    );
    println!("Prefetching elevation data for map bounding box...");
    let t = std::time::Instant::now();
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

    Ok(highlighted_map)
}
