use super::common::{ReaderType, SourceType, build_elevation_service};
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
    let t0 = std::time::Instant::now();
    let map_source = UsgsTopoMapSource::fetch()?;
    let map_descriptor = map_source.get_map_descriptor(lat, lon)?;
    let map_path = map_source.fetch_map(&map_descriptor)?;
    println!("Fetched topo map from USGS: {}", map_path.to_string_lossy());
    println!(
        "{:.3}s ({:.3}s total)",
        t0.elapsed().as_secs_f32(),
        t0.elapsed().as_secs_f32()
    );
    let t = std::time::Instant::now();
    let mut elevation_service =
        build_elevation_service(ReaderType::Gdal, SourceType::Usgs, None, None)?;
    elevation_service.prefetch_region(&map_descriptor.bbox)?;
    println!(
        "Prefetched elevation data for map bounding box: {:?}",
        map_descriptor.bbox
    );
    println!(
        "{:.3}s ({:.3}s total)",
        t.elapsed().as_secs_f32(),
        t0.elapsed().as_secs_f32()
    );
    let t = std::time::Instant::now();
    // LOS calc here
    let elvation_at_point = elevation_service.elevation_at(lat, lon)?;

    println!("Elevation at point: {} ft", elvation_at_point.ft);
    println!(
        "{:.3}s ({:.3}s total)",
        t.elapsed().as_secs_f32(),
        t0.elapsed().as_secs_f32()
    );
    let t = std::time::Instant::now();

    let elevation_top_left =
        elevation_service.elevation_at(map_descriptor.bbox.max_lat, map_descriptor.bbox.min_lon)?;
    let elevation_center_top = elevation_service.elevation_at(
        map_descriptor.bbox.max_lat,
        (map_descriptor.bbox.max_lon + map_descriptor.bbox.min_lon) / 2.0,
    )?;
    let elevation_top_right =
        elevation_service.elevation_at(map_descriptor.bbox.max_lat, map_descriptor.bbox.max_lon)?;

    println!(
        "{:.2}\t\t{:.2}\t\t{:.2}",
        elevation_top_left.ft, elevation_center_top.ft, elevation_top_right.ft,
    );
    println!(
        "{:.3}s ({:.3}s total)",
        t.elapsed().as_secs_f32(),
        t0.elapsed().as_secs_f32()
    );
    let t = std::time::Instant::now();

    let elevation_center_left = elevation_service.elevation_at(
        (map_descriptor.bbox.max_lat + map_descriptor.bbox.min_lat) / 2.0,
        map_descriptor.bbox.min_lon,
    )?;
    let elevation_center = elevation_service.elevation_at(
        (map_descriptor.bbox.max_lat + map_descriptor.bbox.min_lat) / 2.0,
        (map_descriptor.bbox.max_lon + map_descriptor.bbox.min_lon) / 2.0,
    )?;
    let elevation_center_right = elevation_service.elevation_at(
        (map_descriptor.bbox.max_lat + map_descriptor.bbox.min_lat) / 2.0,
        map_descriptor.bbox.max_lon,
    )?;

    println!(
        "{:.2}\t\t{:.2}\t\t{:.2}",
        elevation_center_left.ft, elevation_center.ft, elevation_center_right.ft,
    );
    println!(
        "{:.3}s ({:.3}s total)",
        t.elapsed().as_secs_f32(),
        t0.elapsed().as_secs_f32()
    );
    let t = std::time::Instant::now();

    let elevation_bottom_left =
        elevation_service.elevation_at(map_descriptor.bbox.min_lat, map_descriptor.bbox.min_lon)?;
    let elevation_center_bottom = elevation_service.elevation_at(
        map_descriptor.bbox.min_lat,
        (map_descriptor.bbox.max_lon + map_descriptor.bbox.min_lon) / 2.0,
    )?;
    let elevation_bottom_right =
        elevation_service.elevation_at(map_descriptor.bbox.min_lat, map_descriptor.bbox.max_lon)?;

    println!(
        "{:.2}\t\t{:.2}\t\t{:.2}",
        elevation_bottom_left.ft, elevation_center_bottom.ft, elevation_bottom_right.ft,
    );
    println!(
        "{:.3}s ({:.3}s total)",
        t.elapsed().as_secs_f32(),
        t0.elapsed().as_secs_f32()
    );

    Ok(())
}
