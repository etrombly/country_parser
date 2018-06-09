extern crate chrono;
extern crate geo;
extern crate gtk;

use std::collections::BTreeMap;
use self::chrono::NaiveDateTime;
use gtk::{TreeStoreExt, TreeStoreExtManual, TreeViewExt};

#[derive(Clone, Debug)]
pub struct Visit {
    pub country: String,
    pub start: NaiveDateTime,
    pub end: Option<NaiveDateTime>,
}

impl Visit {
    pub fn new(country: String, start: NaiveDateTime, end: Option<NaiveDateTime>) -> Visit {
        Visit {
            country,
            start,
            end,
        }
    }

    pub fn start_to_string(&self) -> String {
        self.start.format("%d %b %Y").to_string()
    }

    pub fn end_to_string(&self) -> String {
        match self.end {
            Some(x) => x.format("%d %b %Y").to_string(),
            _ => "".to_string(),
        }
    }
}


pub trait VisitsMethods {
    fn set_country_model(&self, tree: &gtk::TreeView);
    fn set_year_model(&self, tree: &gtk::TreeView);
}

pub type Visits = Vec<Visit>;

impl VisitsMethods for Visits {
    fn set_country_model(&self, tree: &gtk::TreeView) {
        let sorted = self.iter().fold(BTreeMap::new(), |mut m, c| {
            m.entry(c.country.clone())
                .or_insert_with(Vec::new)
                .push(c);
            m
        });

        let model = gtk::TreeStore::new(&[gtk::Type::String, gtk::Type::String, gtk::Type::String]);

        for (key, visits) in sorted {
            let top = model.append(None);
            model.set(&top, &[0], &[&key]);
            for visit in visits {
                let entries = model.append(&top);
                model.set(
                    &entries,
                    &[1, 2],
                    &[&visit.start_to_string(), &visit.end_to_string()],
                );
            }
        }

        tree.set_model(&model);
    }

    fn set_year_model(&self, tree: &gtk::TreeView) {
        let sorted = self.iter().fold(BTreeMap::new(), |mut m, c| {
            m.entry(c.start.format("%Y").to_string())
                .or_insert_with(Vec::new)
                .push(c);
            m
        });

        let model = gtk::TreeStore::new(&[gtk::Type::String, gtk::Type::String, gtk::Type::String]);

        for (key, visits) in sorted {
            let top = model.append(None);
            model.set(&top, &[0], &[&key]);
            for visit in visits {
                let entries = model.append(&top);
                model.set(
                    &entries,
                    &[0, 1, 2],
                    &[
                        &visit.country.clone(),
                        &visit.start_to_string(),
                        &visit.end_to_string(),
                    ],
                );
            }
        }

        tree.set_model(&model);
    }
}
