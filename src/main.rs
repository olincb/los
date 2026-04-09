use los::service::los::LineOfSightResult;
use los::service::{ElevationService, LineOfSightService};
use los::source::topo::{TopoSource, UsgsTopoMapSource};
use los::source::{DemSource, Location};
use los::{Bbox, DemReader, GdalReader, GeoTiffReader, UsgsSource};

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
enum Commands {
    /// Find elevation at a given lat/lon using a specified reader and source.
    Elevation {
        /// Type of reader to use
        #[arg(short, long, default_value = "gdal")]
        reader_type: ReaderType,

        /// Source for fetching the DEM (ignored if dem_path is provided)
        #[arg(short, long, default_value = "usgs")]
        source_type: SourceType,

        /// Path to a local GeoTIFF file to bypass using an automatic source
        /// (overrides source_type if provided)
        #[arg(short, long, conflicts_with = "url_dem")]
        local_dem: Option<String>,

        /// Remote URL to a DEM file to bypass using an automatic source
        /// (overrides source_type if provided)
        #[arg(short, long, conflicts_with = "local_dem")]
        url_dem: Option<String>,

        /// Latitude of the point to query (e.g., 48.7766298)
        #[arg(long, allow_hyphen_values = true, value_parser = lat_validator)]
        lat: f64,

        /// Longitude of the point to query (e.g., -121.8144732)
        #[arg(long, allow_hyphen_values = true, value_parser = lon_validator)]
        lon: f64,
    },
    /// Retrieve a topographical map for a given lat/lon.
    Topo {
        /// Latitude of the point to retrieve the topo map for (e.g., 48.7766298)
        #[arg(long, allow_hyphen_values = true, value_parser = lat_validator)]
        lat: f64,

        /// Longitude of the point to retrieve the topo map for (e.g., -121.8144732)
        #[arg(long, allow_hyphen_values = true, value_parser = lon_validator)]
        lon: f64,
    },
    Sightline {
        /// Latitude of the observer point (e.g., 48.7766298)
        #[arg(long, allow_hyphen_values = true, value_parser = lat_validator)]
        lat: f64,

        /// Longitude of the observer point (e.g., -121.8144732)
        #[arg(long, allow_hyphen_values = true, value_parser = lon_validator)]
        lon: f64,

        /// Latitude of the target point (e.g., 48.7766298)
        #[arg(long, allow_hyphen_values = true, value_parser = lat_validator)]
        target_lat: f64,

        /// Longitude of the target point (e.g., -121.8144732)
        #[arg(long, allow_hyphen_values = true, value_parser = lon_validator)]
        target_lon: f64,
    },
    /// For a given lat/lon, output a map with visible area highlighted.
    Highlight {
        /// Latitude of the point to create the map for (e.g., 48.7766298)
        #[arg(long, allow_hyphen_values = true, value_parser = lat_validator)]
        lat: f64,

        /// Longitude of the point to create the map for (e.g., -121.8144732)
        #[arg(long, allow_hyphen_values = true, value_parser = lon_validator)]
        lon: f64,

        /// File path the output file should be written to.
        #[arg(short, long, default_value = "map.pdf")]
        output: PathBuf,
    },
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Clone, ValueEnum)]
#[clap(rename_all = "lower")]
enum SourceType {
    Usgs,
    OpenTopo,
}

impl SourceType {
    fn to_source(&self) -> Box<dyn DemSource> {
        match self {
            SourceType::Usgs => Box::new(UsgsSource),
            SourceType::OpenTopo => {
                // Placeholder for OpenTopo source implementation
                unimplemented!("OpenTopo source is not implemented yet");
            }
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
#[clap(rename_all = "lower")]
enum ReaderType {
    GeoTiff,
    Gdal,
}

impl ReaderType {
    fn to_reader(&self) -> Box<dyn DemReader> {
        match self {
            ReaderType::GeoTiff => Box::new(GeoTiffReader),
            ReaderType::Gdal => Box::new(GdalReader),
        }
    }
}

fn parse_dem_spec(local: Option<String>, url: Option<String>) -> anyhow::Result<Option<Location>> {
    match (local, url) {
        (Some(local_path), None) => Ok(Some(Location::LocalPath(PathBuf::from(local_path)))),
        (None, Some(remote_url)) => Ok(Some(Location::RemoteUrl(remote_url))),
        (Some(_), Some(_)) => Err(anyhow::anyhow!(
            "Cannot specify both local_dem and url_dem. Please choose one."
        )),
        (None, None) => Ok(None),
    }
}

fn build_elevation_service(
    reader_type: ReaderType,
    source_type: SourceType,
    local_dem: Option<String>,
    url_dem: Option<String>,
) -> anyhow::Result<ElevationService> {
    let source = source_type.to_source();
    let reader = reader_type.to_reader();
    let dem_location = parse_dem_spec(local_dem, url_dem)?;

    Ok(ElevationService::new(source, reader, dem_location))
}

fn lat_validator(s: &str) -> Result<f64, String> {
    match s.parse::<f64>() {
        Ok(val) if (-90.0..=90.0).contains(&val) => Ok(val),
        _ => Err(format!(
            "Invalid latitude value: {}. Must be a number between -90 and 90.",
            s
        )),
    }
}

fn lon_validator(s: &str) -> Result<f64, String> {
    match s.parse::<f64>() {
        Ok(val) if (-180.0..=180.0).contains(&val) => Ok(val),
        _ => Err(format!(
            "Invalid longitude value: {}. Must be a number between -180 and 180.",
            s
        )),
    }
}

fn handle_elevation_command(
    reader_type: ReaderType,
    source_type: SourceType,
    local_dem: Option<String>,
    url_dem: Option<String>,
    lat: f64,
    lon: f64,
) -> anyhow::Result<()> {
    let elevation_service = build_elevation_service(reader_type, source_type, local_dem, url_dem)?;
    let elevation = elevation_service.elevation_at(lat, lon)?;
    println!(
        "Elevation at ({}, {}): {:.2} m ({:.2} ft)",
        lat, lon, elevation.m, elevation.ft
    );
    Ok(())
}

fn handle_topo_command(lat: f64, lon: f64) -> anyhow::Result<()> {
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

fn handle_sightline_command(
    lat: f64,
    lon: f64,
    target_lat: f64,
    target_lon: f64,
) -> anyhow::Result<()> {
    let epsilon = 0.0001;
    let mut elevation_service =
        build_elevation_service(ReaderType::Gdal, SourceType::Usgs, None, None)?;
    let bbox = Bbox {
        min_lat: lat.min(target_lat) - epsilon,
        max_lat: lat.max(target_lat) + epsilon,
        min_lon: lon.min(target_lon) - epsilon,
        max_lon: lon.max(target_lon) + epsilon,
    };
    elevation_service.prefetch_region(&bbox)?;
    let los_service = LineOfSightService::new(Box::new(elevation_service));
    let viewer_height_m = 2.0; // Giving the caller the benefit of the doubt.
    match los_service.has_line_of_sight_with_height(
        lat,
        lon,
        target_lat,
        target_lon,
        viewer_height_m,
    )? {
        LineOfSightResult::Clear => println!(
            "Line of sight from ({}, {}) to ({}, {}) is clear.",
            lat, lon, target_lat, target_lon
        ),
        LineOfSightResult::Blocked {
            lat: blocking_lat,
            lon: blocking_lon,
            terrain_m,
            sightline_m,
        } => println!(
            "Line of sight from ({}, {}) to ({}, {}) is blocked by terrain at ({:.7}, {:.7}) with elevation {:.2} m, which is {:.2} m above the sightline.",
            lat,
            lon,
            target_lat,
            target_lon,
            blocking_lat,
            blocking_lon,
            terrain_m,
            terrain_m - sightline_m
        ),
    }

    Ok(())
}

fn handle_highlight_command(lat: f64, lon: f64, output: PathBuf) -> anyhow::Result<()> {
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

fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Commands::Elevation {
            reader_type,
            source_type,
            local_dem,
            url_dem,
            lat,
            lon,
        } => handle_elevation_command(reader_type, source_type, local_dem, url_dem, lat, lon),
        Commands::Topo { lat, lon } => handle_topo_command(lat, lon),
        Commands::Sightline {
            lat,
            lon,
            target_lat,
            target_lon,
        } => handle_sightline_command(lat, lon, target_lat, target_lon),
        Commands::Highlight { lat, lon, output } => handle_highlight_command(lat, lon, output),
    }
}
