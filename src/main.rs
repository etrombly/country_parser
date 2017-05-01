extern crate location_history;
extern crate geo;

//mod data;
mod country;

use std::fs::File;
use std::io::Read;
use location_history::Locations;
use geo::contains::Contains;
use country::Country;
use geo::{Bbox, Point, Coordinate};

fn main() {
    let mut contents = String::new();
    File::open("LocationHistory.json").unwrap().read_to_string(&mut contents).unwrap();
    let locations: Locations = Locations::new(&contents);
    println!("  {} timestamps", locations.locations.len());
    println!("  from {} to {}", locations.locations[locations.locations.len() - 1].timestamp.format("%Y-%m-%d %H:%M:%S"), 
                                locations.locations[0].timestamp.format("%Y-%m-%d %H:%M:%S"));
    println!("  {} seconds average between timestamps\n", locations.average_time());


//    let countries = data::get_country_data();

    let mut last_country = Country{name: "".to_string(), bb: Bbox{xmin: 0.0, xmax: 0.0, ymin: 0.0, ymax: 0.0}, shapes: Vec::new()};
/*
    println!("Loaded data for {} Countries\n", countries.len());

    for loc in &locations.locations{
        let tmp = geo::Point::new(loc.longitude as f64, loc.latitude as f64);
        //if contains_point(&last_country.bb, &tmp) {
        //    println!("{:?} found in {}", tmp, last_country.name);
        //} else {
            for country in &countries{
                if contains_point(&country.bb, &tmp) {
                    println!("{} found in {}", loc.timestamp.format("%Y-%m-%d").to_string(), country.name);
                    last_country = country.clone();
                }
            }
        //}
    }
*/
}

fn contains_point (bb: &Bbox<f64>, p: &Point<f64>) -> bool{
        p.x() >= bb.xmin && p.x() <= bb.xmax && p.y() >= bb.ymin && p.y() <= bb.ymax
}