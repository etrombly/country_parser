#[macro_use]
extern crate serde_derive;
extern crate bincode;
extern crate geo;
extern crate dbf;
extern crate shapefile_utils;

use std::fs::File;
use std::path::Path;
use std::io::Write;
use std::process::Command;
use std::env;
use shapefile_utils::Shapefile;
use shapefile_utils::shape::{Shape, BoundingBox};
use geo::Bbox;
use dbf::Field;
use bincode::{serialize, Infinite};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Country {
    pub name: String,
    pub bb: geo::Bbox<f64>,
    pub shapes: Vec<geo::Polygon<f64>>,
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    Command::new("x86_64-w64-mingw32-windres")
        .args(&["src/program.rc"])
        .arg(&format!("{}/program.o", out_dir))
        .status()
        .unwrap();

    Command::new("x86_64-w64-mingw32-gcc-ar")
        .args(&["crus", "libprogram.a", "program.o"])
        .current_dir(&Path::new(&out_dir))
        .status()
        .unwrap();

    /*
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=program");
*/

    let mut my_shapefile = Shapefile::new(
        Path::new("src/borders/TM_WORLD_BORDERS-0.3.shp"),
        Path::new("src/borders/TM_WORLD_BORDERS-0.3.shx"),
        Path::new("src/borders/TM_WORLD_BORDERS-0.3.dbf"),
    ).unwrap();

    let mut countries = Vec::new();

    for record in my_shapefile.iter() {
        let mut name = String::new();
        if let Some(&Field::Character(ref x)) = record.metadata.get(&String::from("NAME")) {
            name = x.to_owned();
        }
        if let Shape::Polygon {
            bounding_box: bb,
            parts,
            points,
        } = record.shape
        {
            countries.push(Country {
                name: name,
                bb: bounding_box_to_bbox(bb),
                shapes: shape_poly_to_geo(&parts, &points),
            });
        }
    }

    let encoded: Vec<u8> = serialize(&countries, Infinite).unwrap();

    let mut bin = File::create("src/countries.bin").unwrap();
    bin.write_all(&encoded).unwrap();
    bin.sync_all().unwrap();
}

fn bounding_box_to_bbox(bb: BoundingBox) -> Bbox<f64> {
    Bbox {
        xmin: bb.x_min as f64,
        xmax: bb.x_max as f64,
        ymin: bb.y_min as f64,
        ymax: bb.y_max as f64,
    }
}

fn shape_poly_to_geo(
    parts: &[i32],
    points: &[shapefile_utils::shape::Point],
) -> Vec<geo::Polygon<f64>> {
    let mut inside: Vec<geo::LineString<f64>> = Vec::new();
    for i in 0..parts.len() {
        let next_index = if i < parts.len() - 1 {
            parts[(i + 1) as usize] as usize
        } else {
            points.len() - 1
        };
        let tmp = &points[parts[i as usize] as usize..next_index];
        inside.push(geo::LineString(
            tmp.iter().map(|x| geo::Point::new(x.x, x.y)).collect(),
        ));
    }
    inside
        .iter()
        .cloned()
        .map(|x| geo::Polygon::new(x, Vec::new()))
        .collect()
}
