use los::source::topo::{TopoSource, UsgsTopoMapSource};

pub fn handle_topo_command(lat: f64, lon: f64) -> anyhow::Result<()> {
    let source = UsgsTopoMapSource::fetch()?;
    let map_descriptor = source.get_map_descriptor(lat, lon)?;
    let map_path = source.fetch_map(&map_descriptor)?;
    println!(
        "Topo map {}, for ({}, {}), is located at: {}",
        map_descriptor.name.as_deref().unwrap_or(""),
        lat,
        lon,
        map_path.to_string_lossy()
    );
    Ok(())
}
