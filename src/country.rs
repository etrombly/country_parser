extern crate geo;
extern crate chrono;

use self::chrono::NaiveDateTime;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Country {
    pub name: String,
    pub bb: geo::Bbox<f64>,
    pub shapes: Vec<geo::Polygon<f64>>,
}

#[derive(Clone, Debug)]
pub struct Visit {
    pub country: Country,
    pub start: NaiveDateTime,
    pub end: Option<NaiveDateTime>,
}

impl Visit {
    pub fn new(
        country: Country,
        start: NaiveDateTime,
        end: Option<NaiveDateTime>,
    ) -> Visit {
        Visit {
            country,
            start,
            end,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Visits(Vec<Visit>);

impl Visits {
    pub fn new() -> Visits {
        Visits(Vec::new())
    }

    pub fn add(&mut self, visit: Visit) {
        self.0.push(visit);
    }

    pub fn last_mut(&mut self) -> Option<&mut Visit> {
        self.0.last_mut()
    }
}

impl<'a> IntoIterator for &'a Visits {
    type Item = &'a Visit;
    type IntoIter = ::std::slice::Iter<'a, Visit>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}