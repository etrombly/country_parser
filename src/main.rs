#[macro_use]
extern crate serde_derive;
extern crate location_history;
extern crate geo;
extern crate dbf;
extern crate bincode;

mod country;

use std::fs::File;
use std::io::Read;
use location_history::Locations;
use geo::contains::Contains;
use country::Country;
use geo::Bbox;
use bincode::deserialize;

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

    let encoded = include_bytes!("countries.bin");
    let countries: Vec<Country> = deserialize(&encoded[..]).unwrap();

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
