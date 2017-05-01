extern crate shapefile_utils;
extern crate geo;
extern crate dbf;

use std::fs::File;
use std::path::Path;
use std::io::Write;
use shapefile_utils::Shapefile;
use shapefile_utils::shape::{Shape, Point, BoundingBox};
use dbf::Field;

fn main(){
    let mut f = File::create("src/data.rs").unwrap();

    let mut my_shapefile = Shapefile::new(
    &Path::new("src/borders/TM_WORLD_BORDERS-0.3.shp"),
    &Path::new("src/borders/TM_WORLD_BORDERS-0.3.shx"),
    &Path::new("src/borders/TM_WORLD_BORDERS-0.3.dbf")).unwrap();

    f.write_all(format!("use country::Country;
    use geo::{{Bbox, LineString, Polygon, Point}};
    pub fn get_country_data() -> [Country; {}] {{\n[", my_shapefile.num_records()).as_bytes()).unwrap();

    for record in my_shapefile.iter() {
        let mut name = String::new();
        if let Some(&Field::Character(ref x)) = record.metadata.get(&String::from("NAME")){
            name = x.to_owned();
        }
        if let Shape::Polygon{bounding_box: bb, parts, points} = record.shape {
            f.write_all(format!("Country{{name: \"{}\".to_string(),\nbb: {},\nshapes: vec![{}]}},\n", 
                name, bb_string(bb), shape_to_geo(&parts, &points)).as_bytes()).unwrap();
            //, shape_poly_to_geo(&parts, &points));
        }
    }
    f.write_all(b"]}").unwrap();
}

fn shape_to_geo(parts: &Vec<i32>, points: &Vec<Point>) -> String {
    let length = parts.len();
    let mut inside: Vec<geo::LineString<f64>> = Vec::new();
    let mut result = String::new();
    //let mut external: Vec<geo::LineString<f64>> = Vec::new();
    //let outside = geo::LineString(points.iter().map(|x| geo::Point::new(x.x, x.y)).collect());
    for i in 0 .. (length - 1){
        let tmp: Vec<shapefile_utils::shape::Point> = points[parts[i] as usize .. parts[i + 1] as usize].iter().cloned().collect();
        let tmp: String = tmp.iter().map(|x| format!("Point::new({} as f64, {} as f64),", x.x, x.y)).collect();
        result += &format!("Polygon::new(LineString(vec![{}]), Vec::new()),",tmp); 
    }
    //for poly in &inside{
    //    let mut count = 0;
    //   for poly_inner in &inside{
    //        let tmp = geo::Polygon::new(poly_inner.clone(), Vec::new());
    //        if tmp.contains(poly){
    //            count += 1;
    //        }
    //    }
    //    if count == 0 {
    //        external.push(poly.clone());
    //    }
    //}
    //inside.iter().cloned().map(|x| geo::Polygon::new(x, Vec::new())).collect()
    result
}

fn bb_string(bb: BoundingBox) -> String {
    format!("Bbox{{xmin: {} as f64, xmax: {} as f64, ymin: {} as f64, ymax: {} as f64}}",
    bb.x_min, bb.x_max, bb.y_min, bb.y_max).to_string()
}