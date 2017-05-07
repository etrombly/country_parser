extern crate geo;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Country {
    pub name: String,
    pub bb: geo::Bbox<f64>,
    pub shapes: Vec<geo::Polygon<f64>>,
}
