#![feature(proc_macro)]
#[macro_use]
extern crate serde_derive;
extern crate location_history;
extern crate geo;
extern crate dbf;
extern crate bincode;
extern crate gtk;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::collections::BTreeMap;
use location_history::Locations;
use geo::contains::Contains;
use country::{Country, Visits, Visit};
use geo::Bbox;
use bincode::deserialize;

use gtk::{BoxExt, CellLayoutExt, FileChooserDialog, FileChooserExt, Dialog, DialogExt, Inhibit, Menu, MenuBar,
          MenuItem, MenuItemExt, MenuShellExt, OrientableExt, ProgressBar, WidgetExt};
use gtk::Orientation::Vertical;
use relm::Widget;
use relm_attributes::widget;
use relm::RemoteRelm;

use self::Msg::*;
use self::MenuMsg::*;

mod country;

// Define the structure of the model.
#[derive(Clone)]
pub struct Model {
    visits: Visits,
}

// The messages that can be sent to the update function.
#[derive(Msg)]
enum MenuMsg {
    FileSelected(PathBuf),
    MenuQuit,
}

#[derive(Clone)]
struct MyMenuBar {
    bar: MenuBar,
}

impl Widget for MyMenuBar {
    type Model = ();
    type ModelParam = ();
    type Msg = MenuMsg;
    type Root = MenuBar;

    fn model(_: ()) {}

    fn root(&self) -> &Self::Root {
        &self.bar
    }

    fn update(&mut self, _event: MenuMsg, _model: &mut Self::Model) {}

    fn view(relm: &RemoteRelm<Self>, _model: &Self::Model) -> Self {
        let menu = Menu::new();
        let menu_bar = MenuBar::new();
        let file = MenuItem::new_with_label("File");
        let about = MenuItem::new_with_label("About");
        let quit = MenuItem::new_with_label("Quit");
        let file_item = MenuItem::new_with_label("Import LocationHistory");

        connect!(relm, quit, connect_activate(_), MenuQuit);
        connect!(relm, file_item, connect_activate(_) {
            let result = file_dialog();
            match result {
                Some(x) => (Some(FileSelected(x)), ()),
                _ => (None, ()),
            }
        });

        menu.append(&file_item);
        menu.append(&about);
        menu.append(&quit);
        file.set_submenu(Some(&menu));
        menu_bar.append(&file);
        menu_bar.show_all();

        MyMenuBar { bar: menu_bar }
    }
}

fn file_dialog() -> Option<PathBuf> {
    let dialog = FileChooserDialog::new::<gtk::Window>(
        Some("Import File"),
        None,
        gtk::FileChooserAction::Open,
    );
    let filter = gtk::FileFilter::new();
    filter.set_name("json");
    filter.add_pattern("*.json");
    dialog.add_filter(&filter);
    dialog.add_button("Ok", gtk::ResponseType::Ok.into());
    dialog.add_button("Cancel", gtk::ResponseType::Cancel.into());
    let response_ok: i32 = gtk::ResponseType::Ok.into();
    if dialog.run() == response_ok {
        let path = dialog.get_filename();
        dialog.destroy();
        return path;
    }
    dialog.destroy();
    None
}

#[derive(Msg)]
pub enum Msg {
    LoadFile(PathBuf),
    Quit,
}

#[widget]
impl Widget for Win {
    // The initial model.
    fn model() -> Model {
        Model {
            visits: Visits::new(),
        }
    }

    // Update the model according to the message received.
    fn update(&mut self, event: Msg, model: &mut Model) {
        match event {
            Quit => gtk::main_quit(),
            LoadFile(x) => load_json(&self.root, x, &mut model.visits, &self.tree),
        }
    }

    view! {
        #[name="root"]
        gtk::Window {
            gtk::Box {
                // Set the orientation property of the Box.
                orientation: Vertical,
                // Create a Button inside the Box.
                MyMenuBar {
                    FileSelected(path_buf) => LoadFile(path_buf),
                    MenuQuit => Quit,
                },
                gtk::ScrolledWindow {
                    packing: {
                        expand: true,
                    },
                    gtk::Viewport{
                        #[name="tree"]
                        gtk::TreeView{
                        },
                    },
                },
            },
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}

fn main() {
    Win::run(()).unwrap();
}

fn load_json(parent: &gtk::Window, path: PathBuf, visits: &mut Visits, tree: &gtk::TreeView) {
    let dialog = Dialog::new_with_buttons(
        Some("Processing Location History"),
        Some(parent),
        gtk::DIALOG_MODAL | gtk::DIALOG_DESTROY_WITH_PARENT,
        &[],
    );
    let content = dialog.get_content_area();
    let progress = ProgressBar::new();
    content.pack_start(&progress, true, true, 0);
    progress.set_fraction(0.0);
    dialog.show_all();

    let mut contents = String::new();
    File::open(path)
        .unwrap()
        .read_to_string(&mut contents)
        .unwrap();
    let mut locations: Locations = Locations::new(&contents);
    println!("Loaded  {} timestamps", locations.locations.len());
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

    locations.filter_outliers();

    let mut results: Vec<String> = Vec::new();

    let total = locations.locations.len();

    for (i, loc) in locations.locations.iter().enumerate() {
        progress.set_fraction(i as f64 / total as f64);
        gtk::main_iteration_do(false);
        let tmp = geo::Point::new(loc.longitude, loc.latitude);
        if last_country.bb.contains(&tmp) && last_country.shapes.iter().any(|x| x.contains(&tmp)) {
            // do nothing
        } else {
            for country in &countries {
                if country.bb.contains(&tmp) && country.shapes.iter().any(|x| x.contains(&tmp)) {
                    if let Some(visit) = visits.last_mut() {
                        visit.end = Some(loc.timestamp);
                    }
                    visits.add(Visit::new(country.clone(), loc.timestamp, None));
                    results.push(format!("{} found in {}\n",
                             loc.timestamp.format("%Y-%m-%d").to_string(),
                             country.name));
                    last_country = country.clone();
                } else {
                    println!("couldn't find {} {:?}",
                             loc.timestamp.format("%Y-%m-%d").to_string(), tmp);
                }
            }
        }
    }

    let test = visits.into_iter().fold(BTreeMap::new(), |mut m, c| {
        m.entry(c.start.format("%Y").to_string())
            .or_insert_with(Vec::new)
            .push(c);
        m
    });

    for (key, visits) in test {
        let temp: Vec<&String> = visits.iter().map(|x| &x.country.name).collect();
        println!("{} {:?}", key, temp);
    }

    let test = visits.into_iter().fold(BTreeMap::new(), |mut m, c| {
        m.entry(c.country.name.clone())
            .or_insert_with(Vec::new)
            .push(c);
        m
    });

    let country_column = gtk::TreeViewColumn::new();
    let country_column_cell = gtk::CellRendererText::new();
    country_column.set_title("Country");
    country_column.pack_start(&country_column_cell, true);

    let start_column = gtk::TreeViewColumn::new();
    let start_column_cell = gtk::CellRendererText::new();
    start_column.set_title("Start date");
    start_column.pack_start(&start_column_cell, true);

    let end_column = gtk::TreeViewColumn::new();
    let end_column_cell = gtk::CellRendererText::new();
    end_column.set_title("End date");
    end_column.pack_start(&end_column_cell, true);

    tree.append_column(&country_column);
    tree.append_column(&start_column);
    tree.append_column(&end_column);

    country_column.add_attribute(&country_column_cell, "text", 0);
    start_column.add_attribute(&start_column_cell, "text", 1);
    end_column.add_attribute(&end_column_cell, "text", 2);

    let model = gtk::TreeStore::new(&[gtk::Type::String,gtk::Type::String,gtk::Type::String]);
    
    for (key, visits) in test {
        let top = model.append(None);
        model.set(&top, &[0], &[&key]);
        for visit in visits {
            let entries = model.append(&top);
            model.set(&entries, &[1,2], &[
                        &visit.start_to_string(),
                        &visit.end_to_string(),
                      ]);
        }
    }

    tree.set_model(&model);

    dialog.destroy();
}
