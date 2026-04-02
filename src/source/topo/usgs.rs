/// USGS GeoPDF topographic map retrieval
use super::traits::{TopoMapDescriptor, TopoSource};
use crate::source::SourceError;
use std::path::PathBuf;
use std::sync::OnceLock;

pub struct UsgsTopoMapSource {
    zip_path: PathBuf,
    sqlite_path: PathBuf,
}

const USGS_USTOPO_CURRENT_METADATA_ZIP_URL: &str =
    "https://prd-tnm.s3.amazonaws.com/StagedProducts/Maps/Metadata/ustopo_current.zip";
const USTOPO_CSV_FILENAME: &str = "ustopo_current.csv";

#[derive(Debug, serde::Deserialize)]
struct UsgsTopoMapMetadata {
    map_name: String,
    westbc: f64,
    eastbc: f64,
    southbc: f64,
    northbc: f64,
    product_url: String,
}

/// Find the location of the cache directory. Should be `~/.cache/los`.
fn cache_directory() -> &'static Option<PathBuf> {
    static CACHE_DIR: OnceLock<Option<PathBuf>> = OnceLock::new();
    CACHE_DIR.get_or_init(|| {
        home::home_dir()
            .filter(|path| !path.as_os_str().is_empty())
            .map(|path| path.join(".cache").join("los"))
    })
}

impl UsgsTopoMapSource {

    pub fn fetch() -> Result<Self, SourceError> {
        // TODO: optional flag to force refresh of metadata
        let cache_dir = cache_directory().as_ref().ok_or_else(|| {
            SourceError::Data("Unable to determine cache directory for USGS metadata".into())
        })?;
        std::fs::create_dir_all(cache_dir)?;
        let instance = Self {
            zip_path: cache_dir.join("ustopo_current.zip"),
            sqlite_path: cache_dir.join("ustopo_current.db"),
        };
        // Check if metadata DB file already exists
        if instance.sqlite_path.exists() {
            return Ok(instance);
        }
        let tmp_sqlite_path = instance.sqlite_path.with_extension("db.tmp");
        // Initialize DB file for storing metadata
        let mut conn = rusqlite::Connection::open(&tmp_sqlite_path).map_err(|e| {
            SourceError::Data(format!(
                "Failed to create SQLite database for USGS metadata: {e}"
            ))
        })?;
        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS map_tiles USING rtree(
                id,
                min_lon, max_lon,
                min_lat, max_lat
            )",
            [],
        ).map_err(|e| {
            SourceError::Data(format!(
                "Failed to create map_tiles R-tree table in SQLite database: {e}"
            ))
        })?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS map_meta (
                map_name TEXT,
                product_url TEXT
            )",
            [],
        ).map_err(|e| {
            SourceError::Data(format!(
                "Failed to create mapmeta table in SQLite database: {e}"
            ))
        })?;
        // Check if zip file exists, if not download it
        if !instance.zip_path.exists() {
            // TODO: download zip
            let resp = ureq::get(USGS_USTOPO_CURRENT_METADATA_ZIP_URL)
                .call()
                .map_err(|e| {
                    SourceError::Network(format!("Failed to download USGS metadata zip: {e}"))
                })?;
            let mut file = std::fs::File::create(&instance.zip_path)?;
            std::io::copy(&mut resp.into_body().as_reader(), &mut file)?;
        }
        // Unzip and extract CSV
        let zip_file = std::fs::File::open(&instance.zip_path)?;
        let mut zip = zip::ZipArchive::new(zip_file).map_err(|e| {
            SourceError::Data(format!("Failed to read USGS metadata zip file: {e}"))
        })?;
        let csv_file = zip.by_name(USTOPO_CSV_FILENAME).map_err(|e| {
            SourceError::Data(format!(
                "Failed to find {} in USGS metadata zip: {e}",
                USTOPO_CSV_FILENAME
            ))
        })?;
        // Parse CSV and save metadata into SQLite database for lookups
        let mut csv_reader = csv::Reader::from_reader(csv_file);
        let transaction = conn.transaction().map_err(|e| {
            SourceError::Data(format!(
                "Failed to start transaction for inserting USGS metadata: {e}"
            ))
        })?;
        for result in csv_reader.deserialize::<UsgsTopoMapMetadata>() {
            let record = result.map_err(|e| {
                SourceError::Data(format!("Failed to parse USGS metadata CSV record: {e}"))
            })?;
            transaction.execute(
                "INSERT INTO map_meta (map_name, product_url) VALUES (?1, ?2)",
                rusqlite::params![ &record.map_name, &record.product_url],
            ).map_err(|e| {
                SourceError::Data(format!(
                    "Failed to insert USGS metadata record into SQLite database: {e}"
                ))
            })?;
            let rowid = transaction.last_insert_rowid();
            transaction.execute(
                "INSERT INTO map_tiles (id, min_lon, max_lon, min_lat, max_lat) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![ rowid, record.westbc, record.eastbc, record.southbc, record.northbc],
            ).map_err(|e| {
                SourceError::Data(format!(
                    "Failed to insert USGS metadata tile record into SQLite database: {e}"
                ))
            })?;
        }
        transaction.commit().map_err(|e| {
            SourceError::Data(format!(
                "Failed to commit transaction for inserting USGS metadata: {e}"
            ))
        })?;
        drop(conn);
        // Rename temp DB file to final location
        std::fs::rename(tmp_sqlite_path, &instance.sqlite_path)?;
        Ok(instance)
    }
}


impl TopoSource for UsgsTopoMapSource {
    fn get_map_for_point(&self, lat: f64, lon: f64) -> Result<TopoMapDescriptor, SourceError> {
        // TODO: look up lat/lon in csv to find matching map
        // TODO: return map descriptor with URL to GeoPDF
        panic!("USGS topo map retrieval not implemented yet");
    }
}
