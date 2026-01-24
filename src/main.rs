use los::{DemDescriptor, DemHandle, DemReader, GeoTiffReader};
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let reader = GeoTiffReader;
    let path = PathBuf::from("/Users/christopher.olin/Code/test/personal/los/opentopo.tif");
    let descriptor = DemDescriptor {
        path,
    };
    let handle = reader.open(&descriptor)?;
    let elevation = handle.elevation_at(40.25, -105.6)?;
    println!("elevation: {}", elevation.height_m);

    // let source = OpenTopoSource::new(api_key);
    // let reader = GdalReader::new();
    // let svc = ElevationService { source, reader };
    //
    // let e = svc.elevation_at(33.12, -89.33)?;
    Ok(())
}
