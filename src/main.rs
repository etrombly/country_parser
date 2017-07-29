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
use location_history::Locations;
use geo::contains::Contains;
use country::Country;
use geo::Bbox;
use bincode::deserialize;

use gtk::{
    BoxExt,
    FileChooserDialog,
    FileChooserExt,
    DialogExt,
    Inhibit,
    Menu,
    MenuBar,
    MenuItem,
    MenuItemExt,
    MenuShellExt,
    OrientableExt,
    WidgetExt,
};
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
    text: String,
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

    fn model(_: ()) {
    }

    fn root(&self) -> &Self::Root {
        &self.bar
    }

    fn update(&mut self, _event: MenuMsg, _model: &mut Self::Model) {
    }

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

        MyMenuBar {
            bar: menu_bar,
        }
    }
}

fn file_dialog() -> Option<PathBuf> {
    let dialog = FileChooserDialog::new::<FileChooserDialog>(Some("Import File"), None, gtk::FileChooserAction::Open);
    let filter = gtk::FileFilter::new();
    filter.set_name("json");
    filter.add_pattern("*.json");
    dialog.add_filter(&filter);
    dialog.add_button("Ok", gtk::ResponseType::Ok.into());
    dialog.add_button("Cancel", gtk::ResponseType::Cancel.into());
    let response_ok: i32 = gtk::ResponseType::Ok.into();
    if dialog.run() ==  response_ok {
            let path = dialog.get_filename();
            dialog.destroy();
            return path;
    }
    dialog.destroy();
    return None;
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
            text: "".to_string(),
        }
    }

    // Update the model according to the message received.
    fn update(&mut self, event: Msg, model: &mut Model) {
        match event {
            Quit => gtk::main_quit(),
            LoadFile(x) => model.text = load_json(x),
        }
    }

    view! {
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
                        gtk::Label {
                            halign: gtk::Align::Start,
                            // Bind the text property of the label to the counter attribute of the model.
                            text: &model.text,
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

fn load_json(path: PathBuf) -> String {
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

    for loc in locations.locations.iter() {
        gtk::main_iteration_do(false);
        let tmp = geo::Point::new(loc.longitude, loc.latitude);
        if last_country.bb.contains(&tmp) &&
           last_country.shapes.iter().any(|x| x.contains(&tmp)) {
            //println!("{:?} found in {}", tmp, last_country.name);
        } else {
            for country in &countries {
                if country.bb.contains(&tmp) &&
                   country.shapes.iter().any(|x| x.contains(&tmp)) {
                    results.push(format!("{} found in {}\n",
                             loc.timestamp.format("%Y-%m-%d").to_string(),
                             country.name));
                    last_country = country.clone();
                } else {
                    //println!("couldn't find {} {:?}",
                    //         loc.timestamp.format("%Y-%m-%d").to_string(), tmp);
                }
            }
        }
    }
    results.into_iter().collect()
}