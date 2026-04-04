use los::source::Location;
use los::source::topo::{TopoSource, UsgsTopoMapSource};
use los::{DemHandle, DemReader, DemSource, GdalReader, GeoTiffReader, UsgsSource};

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
    Topo {
        /// Latitude of the point to retrieve the topo map for (e.g., 48.7766298)
        #[arg(long, allow_hyphen_values = true, value_parser = lat_validator)]
        lat: f64,

        /// Longitude of the point to retrieve the topo map for (e.g., -121.8144732)
        #[arg(long, allow_hyphen_values = true, value_parser = lon_validator)]
        lon: f64,
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

#[derive(Debug, Clone, ValueEnum)]
#[clap(rename_all = "lower")]
enum ReaderType {
    GeoTiff,
    Gdal,
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

/// CLI for finding elevation at a given lat/lon using a specified reader and source.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct CliArgs {}

fn handle_elevation_command(
    reader_type: ReaderType,
    source_type: SourceType,
    local_dem: Option<String>,
    url_dem: Option<String>,
    lat: f64,
    lon: f64,
) -> anyhow::Result<()> {
    let dem_descriptor = match (local_dem, url_dem) {
        (Some(local_path), None) => Location::LocalPath(PathBuf::from(local_path)),
        (None, Some(remote_url)) => Location::RemoteUrl(remote_url),
        (Some(_), Some(_)) => {
            return Err(anyhow::anyhow!(
                "Cannot specify both local_dem and url_dem. Please choose one."
            ));
        }
        (None, None) => {
            match source_type {
                SourceType::Usgs => UsgsSource.get_dem_for_point(lat, lon)?,
                SourceType::OpenTopo => {
                    return Err(anyhow::anyhow!("OpenTopo source is not implemented yet"));
                } // SourceType::OpenTopo => OpenTopoSource::new(api_key).get_dem_for_point(lat, lon)?,
            }
        }
    };
    let elevation = match reader_type {
        ReaderType::GeoTiff => {
            let reader = GeoTiffReader;
            let handle = reader.open(&dem_descriptor)?;
            handle.elevation_at(lat, lon)?
        }
        ReaderType::Gdal => {
            let reader = GdalReader;
            let handle = reader.open(&dem_descriptor)?;
            handle.elevation_at(lat, lon)?
        }
    };
    println!(
        "Elevation at ({}, {}): {:.2} m ({:.2} ft)",
        lat,
        lon,
        elevation.height_m,
        elevation.height_m * 3.28084
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
    }
}
