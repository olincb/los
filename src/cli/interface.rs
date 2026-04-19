use super::common::{ReaderType, SourceType};
use super::{elevation, highlight, sightline, topo, viewshed};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

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

/// CLI for the Line of Sight library. Provides commands for elevation queries, topographical map
/// retrieval, sightline analysis, and visibility highlighting based on geographic coordinates.
#[derive(Subcommand, Debug)]
enum Commands {
    /// Find elevation at a given lat/lon using a specified reader and source.
    Elevation {
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
    /// Determine if the target point is visible from the observer point.
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
        #[arg(short, long, default_value = "map.png")]
        output: PathBuf,
    },
    /// For a given lat/lon, write a terminal-based map with visible area highlighted.
    Viewshed {
        /// Latitude of the observer point (e.g., 48.7766298)
        #[arg(long, allow_hyphen_values = true, value_parser = lat_validator)]
        lat: f64,

        /// Longitude of the observer point (e.g., -121.8144732)
        #[arg(long, allow_hyphen_values = true, value_parser = lon_validator)]
        lon: f64,

        /// Width of the area to cover in degrees. Defaults to 0.125 degrees (~6.5 miles).
        /// Height is determined based on width to maintain aspect ratio of the terminal output.
        #[arg(short, long, default_value = "0.125")]
        width: f64,

        /// Width of the output in characters. Defaults to width of the terminal.
        #[arg(long)]
        cols: Option<usize>,

        /// Height of the output in characters. Defaults to height of the terminal.
        #[arg(long)]
        rows: Option<usize>,
    },
}

#[derive(Parser)]
pub struct Cli {
    /// Type of reader to use
    #[arg(short, long, default_value = "gdal", global = true)]
    reader_type: ReaderType,

    /// Source for fetching the DEM (ignored if dem_path is provided)
    #[arg(short, long, default_value = "usgs", global = true)]
    source_type: SourceType,

    /// Path to a local GeoTIFF file to bypass using an automatic source
    /// (overrides source_type if provided)
    #[arg(short, long, conflicts_with = "url_dem", global = true)]
    local_dem: Option<String>,

    /// Remote URL to a DEM file to bypass using an automatic source
    /// (overrides source_type if provided)
    #[arg(short, long, conflicts_with = "local_dem", global = true)]
    url_dem: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

pub fn handle_main_cli(parser: Cli) -> anyhow::Result<()> {
    match parser.command {
        Commands::Elevation { lat, lon } => elevation::handle_elevation_command(
            parser.reader_type,
            parser.source_type,
            parser.local_dem,
            parser.url_dem,
            lat,
            lon,
        ),
        Commands::Topo { lat, lon } => topo::handle_topo_command(lat, lon),
        Commands::Sightline {
            lat,
            lon,
            target_lat,
            target_lon,
        } => sightline::handle_sightline_command(lat, lon, target_lat, target_lon),
        Commands::Highlight { lat, lon, output } => {
            highlight::handle_highlight_command(lat, lon, output)
        }
        Commands::Viewshed {
            lat,
            lon,
            width,
            cols,
            rows,
        } => viewshed::handle_viewshed_command(lat, lon, width, cols, rows),
    }
}
