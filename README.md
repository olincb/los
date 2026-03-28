# los
Line Of Sight GIS tool in Rust.

The goal of this project is to export a map for any given coordinate,
with the area of the map visible from that coordinate highlighted.

There are several phases to accomplish this:

1. Digital Elevation Model (DEM) retrieval
    - Digital Elevation Models are sourced from:
        - [USGS National Map](https://www.usgs.gov/core-science-systems/national-geospatial-program/national-map) (TODO): `src/source/usgs.rs`
            - Interval: ???
            - Accuracy: ???
        - [OpenTopography](https://opentopography.org/) ([API](https://portal.opentopography.org/apidocs/)) (TODO): `src/source/opentopo.rs`
            - Interval: ???
            - Accuracy: ???
        - A local GeoTiff DEM file
2. Elevation lookup from GeoTiff DEM files
    - DEM reader implementations using:
        - [GDAL](https://github.com/georust/gdal) (TODO): `src/reader/gdal.rs`
        - [geotiff](https://github.com/georust/geotiff): `src/reader/geotiff.rs`
    - Elevation service (`src/service/elevation.rs`) provides elevation lookup for any given coordinate, given a DEM Source and a DEM Reader implementation.
3. Line of sight calculation (see below)
4. Map retrieval
5. Map rendering
6. Map export

## Line of sight calculation

- Let `E(lat, lon)` be the elevation at a given latitude and longitude.
- Let `d(lat0, lon0, lat1, lon1)` be the distance between two coordinates.
- For a given coordinate `(lat0, lon0)` and a target coordinate `(lat1, lon1)`, we can determine if there is a clear line of sight by checking if the elevation at any point along the line between `(lat0, lon0)` and `(lat1, lon1)` is greater than the straight line between the points.
    1. Let `d0 = d(lat0, lon0, lat1, lon1)` be the distance between the two points.
    2. For each point `(lat, lon)` along the line between `(lat0, lon0)` and `(lat1, lon1)`, calculate the elevation `E(lat, lon)`.
    2. Calculate the elevation of the straight line between `(lat0, lon0)` and `(lat1, lon1)` at the point `(lat, lon)` using linear interpolation: `E_line = E(lat0, lon0) + (E(lat1, lon1) - E(lat0, lon0)) * (d(lat0, lon0, lat, lon) / d0)`.
    3. If `E(lat, lon) > E_line` for any point along the line, then there is no clear line of sight between `(lat0, lon0)` and `(lat1, lon1)`.

## TODO

- Implement GDAL GeoTiff reader
- Implement USGS DEM retrieval
- Implement OpenTopography DEM retrieval
- Implement line of sight calculation
- Implement map retrieval
- Implement map rendering
- Implement map export
- Spruce up local GeoTiff DEM retrieval
- Implement a CLI
- Look into ASTER GDEM
    - https://www.tellusxdp.com/en-us/catalog/data/aster_gdem_ver_3.html
    - Pixel interval: 1 sec (Approximately 30 m)
    - Height accuracy: 7-14 m

