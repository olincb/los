# los
Line Of Sight GIS tool in Rust.

The goal of this project is to export a map for any given coordinate,
with the area of the map visible from that coordinate highlighted.

System overview:

1. [Digital Elevation Model (DEM) retrieval](#dem-retrieval)
2. [Elevation lookup from DEM datasets](#dem-reader)
3. [Line of sight calculation](#line-of-sight-calculation)
4. Map retrieval (TODO)
5. Map rendering (TODO)
6. Map export (TODO)

The current default implementation uses [USGS 3DEP data streamed via GDAL from AWS-hosted VRT datasets](#usgs-source).

## Usage

### Quick Start

Query elevation at a given coordinate:

```bash
cargo run -- --lat 48.7766298 --lon -121.8144732
```

```text
Elevation at (48.7766298, -121.8144732): 3281.13 m (10764.87 ft)
```

### Help
```bash
cargo run -- --help
```
<details>
<summary>CLI Help</summary>


```text
CLI for finding elevation at a given lat/lon using a specified reader and source

Usage: los [OPTIONS] --lat <LAT> --lon <LON>

Options:
  -r, --reader-type <READER_TYPE>  Type of reader to use [default: gdal] [possible values: geotiff, gdal]
  -s, --source-type <SOURCE_TYPE>  Source for fetching the DEM (ignored if dem_path is provided) [default: usgs] [possible values: usgs, opentopo]
  -l, --local-dem <LOCAL_DEM>      Path to a local GeoTIFF file to bypass using an automatic source (overrides source_type if provided)
  -u, --url-dem <URL_DEM>          Remote URL to a DEM file to bypass using an automatic source (overrides source_type if provided)
      --lat <LAT>                  Latitude of the point to query (e.g., 48.7766298)
      --lon <LON>                  Longitude of the point to query (e.g., -121.8144732)
  -h, --help                       Print help
  -V, --version                    Print version
```
</details>

### Release build
```bash
cargo build --release
target/release/los --lat 48.7766298 --lon -121.8144732
```

```text
Elevation at (48.7766298, -121.8144732): 3281.13 m (10764.87 ft)
```

## Architecture

The system is designed to separate *where DEM data comes from* from *how it is read*.

### DEM Retrieval

`src/source/`

Responsible for resolving which DEM dataset should be used for a given coordinate.

#### USGS Source

The current implementation uses the USGS 3DEP seamless DEM VRT hosted on AWS:

- https://prd-tnm.s3.amazonaws.com/StagedProducts/Elevation/13/TIFF/USGS_Seamless_DEM_13.vrt

This VRT acts as a global index over many GeoTIFF tiles.

Using GDAL, this enables:

- global DEM access without downloading full datasets
- on-demand retrieval of only the required GeoTIFF tiles
- efficient streaming via HTTP range requests (`/vsicurl/`)

In practice:

- a query for a single `(lat, lon)` is resolved to a specific underlying GeoTIFF tile
- GDAL fetches only the required byte ranges from that tile

**Note:**  
Reading a single point may still trigger a full internal block read (e.g., 128×128 pixels), resulting in ~10s–100s of KB of network transfer per lookup.

#### Future Sources

- OpenTopography (planned)

### DEM Reader 

`src/reader/`

Responsible for opening a DEM and providing elevation lookup.

- `DemReader`: opens a dataset and returns a `DemHandle`
- `DemHandle`: provides `elevation_at(lat, lon)`

#### GDAL Reader
`GdalReader` supports remote and local datasets. It depends on a local installation of GDAL.

#### geotiff Reader
`GeoTiffReader` supports local GeoTIFF files only. It is a lightweight implementation using the `geotiff` crate, without external dependencies.

### Elevation Service (`src/service/`)
Combines a `DemSource` and `DemReader` to provide elevation lookup.

## Dependencies
At the moment, the default CLI path depends on GDAL for remote USGS access.
A lightweight local-only `geotiff` reader also exists, but the project currently assumes [GDAL is installed](https://gdal.org/en/stable/download.html).

`brew install gdal` on macOS.

GDAL is a large native dependency, but it enables remote DEM access via VRT and HTTP range requests.

A long-term goal is to make this optional and support a pure-Rust `geotiff` implementation for local datasets or for use with OpenTopo.

## Line of sight calculation

(TODO)

- Let `E(lat, lon)` be the elevation at a given latitude and longitude.
- Let `d(lat0, lon0, lat1, lon1)` be the distance between two coordinates.
- For a given coordinate `(lat0, lon0)` and a target coordinate `(lat1, lon1)`, we can determine if there is a clear line of sight by checking if the elevation at any point along the line between `(lat0, lon0)` and `(lat1, lon1)` is greater than the straight line between the points.
    1. Let `d0 = d(lat0, lon0, lat1, lon1)` be the distance between the two points.
    2. For each point `(lat, lon)` along the line between `(lat0, lon0)` and `(lat1, lon1)`, calculate the elevation `E(lat, lon)`.
    2. Calculate the elevation of the straight line between `(lat0, lon0)` and `(lat1, lon1)` at the point `(lat, lon)` using linear interpolation: `E_line = E(lat0, lon0) + (E(lat1, lon1) - E(lat0, lon0)) * (d(lat0, lon0, lat, lon) / d0)`.
    3. If `E(lat, lon) > E_line` for any point along the line, then there is no clear line of sight between `(lat0, lon0)` and `(lat1, lon1)`.

## TODO

### Core functionality
- Implement OpenTopography DEM retrieval
- Implement line of sight calculation

### Performance / usability improvements
- Caching of DEM blocks to avoid repeated retrievals for nearby coordinates
- Make GDAL an optional dependency

### Map features
- Map retrieval
- Map rendering
- Map export

### Research
- Look into ASTER GDEM
    - https://www.tellusxdp.com/en-us/catalog/data/aster_gdem_ver_3.html
    - Pixel interval: 1 sec (Approximately 30 m)
    - Height accuracy: 7-14 m
