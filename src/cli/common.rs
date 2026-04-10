use clap::ValueEnum;
use los::reader::{GdalReader, GeoTiffReader};
use los::source::{Location, UsgsSource};
use los::{DemReader, DemSource, ElevationService};
use std::path::PathBuf;

#[derive(Debug, Clone, ValueEnum)]
#[clap(rename_all = "lower")]
pub enum SourceType {
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
pub enum ReaderType {
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

pub fn parse_dem_spec(
    local: Option<String>,
    url: Option<String>,
) -> anyhow::Result<Option<Location>> {
    match (local, url) {
        (Some(local_path), None) => Ok(Some(Location::LocalPath(PathBuf::from(local_path)))),
        (None, Some(remote_url)) => Ok(Some(Location::RemoteUrl(remote_url))),
        (Some(_), Some(_)) => Err(anyhow::anyhow!(
            "Cannot specify both local_dem and url_dem. Please choose one."
        )),
        (None, None) => Ok(None),
    }
}

pub fn build_elevation_service(
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
