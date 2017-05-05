extern crate location_history;
extern crate geo;
extern crate dbf;
extern crate shapefile_utils;

mod country;

use std::fs::File;
use std::path::Path;
use std::io::Read;
use location_history::Locations;
use geo::contains::Contains;
use country::Country;
use shapefile_utils::Shapefile;
use shapefile_utils::shape::{Shape, BoundingBox};
use geo::{Bbox, Point};
use dbf::Field;

fn main() {
    let mut contents = String::new();
    File::open("LocationHistory.json")
        .unwrap()
        .read_to_string(&mut contents)
        .unwrap();
    let locations: Locations = Locations::new(&contents);
    println!("  {} timestamps", locations.locations.len());
    println!("  from {} to {}",
             locations.locations[locations.locations.len() - 1]
                 .timestamp
                 .format("%Y-%m-%d %H:%M:%S"),
             locations.locations[0]
                 .timestamp
                 .format("%Y-%m-%d %H:%M:%S"));
    println!("  {} seconds average between timestamps\n",
             locations.average_time());

    let mut my_shapefile = Shapefile::new(Path::new("borders/TM_WORLD_BORDERS-0.3.shp"),
                                          Path::new("borders/TM_WORLD_BORDERS-0.3.shx"),
                                          Path::new("borders/TM_WORLD_BORDERS-0.3.dbf"))
            .unwrap();

    let mut countries = Vec::new();

    let mut last_country = Country {
        name: "".to_string(),
        bb: Bbox {
            xmin: 0.0,
            xmax: 0.0,
            ymin: 0.0,
            ymax: 0.0,
        },
        shapes: Vec::new(),
    };

    for record in my_shapefile.iter() {
        let mut name = String::new();
        if let Some(&Field::Character(ref x)) = record.metadata.get(&String::from("NAME")) {
            name = x.to_owned();
        }
        if let Shape::Polygon {
                   bounding_box: bb,
                   parts,
                   points,
               } = record.shape {
            countries.push(Country {
                               name: name,
                               bb: bounding_box_to_bbox(bb),
                               shapes: shape_poly_to_geo(&parts, &points),
                           });
        }
    }

    println!("Loaded data for {} Countries\n", countries.len());

    for loc in locations.locations.iter().rev() {
        let tmp = geo::Point::new(loc.longitude as f64, loc.latitude as f64);
        if last_country.bb.contains(&tmp) &&
           last_country.shapes.iter().any(|x| x.contains(&tmp)) {
            //println!("{:?} found in {}", tmp, last_country.name);
        } else {
            for country in &countries {
                if country.bb.contains(&tmp) &&
                   country.shapes.iter().any(|x| x.contains(&tmp)) {
                    println!("{} found in {}",
                             loc.timestamp.format("%Y-%m-%d").to_string(),
                             country.name);
                    last_country = country.clone();
                } else {
                    //println!("couldn't find {} {:?}",
                    //         loc.timestamp.format("%Y-%m-%d").to_string(), tmp);
                }
            }
        }
    }
}

fn bounding_box_to_bbox(bb: BoundingBox) -> Bbox<f64> {
    Bbox {
        xmin: bb.x_min as f64,
        xmax: bb.x_max as f64,
        ymin: bb.y_min as f64,
        ymax: bb.y_max as f64,
    }
}

fn shape_poly_to_geo(parts: &[i32],
                     points: &[shapefile_utils::shape::Point])
                     -> Vec<geo::Polygon<f64>> {
    let mut inside: Vec<geo::LineString<f64>> = Vec::new();
    for i in 0..parts.len() {
        let next_index = if i < parts.len() - 1 {
            parts[(i + 1) as usize] as usize
        } else {
            points.len() - 1
        };
        let tmp = &points[parts[i as usize] as usize..next_index];
        inside.push(geo::LineString(tmp.iter().map(|x| geo::Point::new(x.x, x.y)).collect()));
    }
    inside
        .iter()
        .cloned()
        .map(|x| geo::Polygon::new(x, Vec::new()))
        .collect()
}
