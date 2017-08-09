#![feature(proc_macro)]
#![windows_subsystem = "windows"]

#[macro_use]
extern crate serde_derive;
extern crate location_history;
extern crate geo;
extern crate dbf;
extern crate bincode;
extern crate gtk;
extern crate gdk_pixbuf;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use location_history::LocationsExt;
use geo::contains::Contains;
use country::{Country, Visits, Visit, VisitsMethods};
use bincode::deserialize;

use gtk::{BoxExt, CellLayoutExt, ContainerExt, FileChooserDialog, FileChooserExt, Dialog,
          DialogExt, Inhibit, Menu, MenuBar, MenuItem, MenuItemExt, MenuShellExt, OrientableExt,
          ProgressBar, TreeView, Viewport, WidgetExt, WindowExt};
use gtk::Orientation::Vertical;
use relm::{Relm, Update, Widget};
use relm_attributes::widget;

use self::Msg::*;
use self::ViewMsg::*;
use self::MenuMsg::*;

mod country;

// The messages that can be sent to the update function.
#[derive(Msg)]
enum MenuMsg {
    SelectFile,
    SortOrder(SortBy),
    MenuAbout,
    MenuQuit,
}

#[derive(Clone)]
struct MyMenuBar {
    bar: MenuBar,
}

/// all the events are handled in Win
impl Update for MyMenuBar {
    type Model = ();
    type ModelParam = ();
    type Msg = MenuMsg;

    fn model(_: &Relm<Self>, _: ()) {}

    fn update(&mut self, _event: MenuMsg) {}
}

impl Widget for MyMenuBar {
    type Root = MenuBar;

    fn root(&self) -> Self::Root {
        self.bar.clone()
    }

    fn view(relm: &Relm<Self>, _model: Self::Model) -> Self {
        let menu_file = Menu::new();
        let menu_sort = Menu::new();
        let menu_help = Menu::new();
        let menu_bar = MenuBar::new();

        let file = MenuItem::new_with_label("File");
        let quit = MenuItem::new_with_label("Quit");
        let file_item = MenuItem::new_with_label("Import LocationHistory");

        let sort = MenuItem::new_with_label("Sort");
        let year = MenuItem::new_with_label("Year");
        let country = MenuItem::new_with_label("Country");

        let help = MenuItem::new_with_label("Help");
        let about = MenuItem::new_with_label("About");

        connect!(relm, quit, connect_activate(_), MenuQuit);
        connect!(relm, file_item, connect_activate(_), SelectFile);
        connect!(relm, year, connect_activate(_), SortOrder(SortBy::Year));
        connect!(relm, country, connect_activate(_), SortOrder(SortBy::Country));
        connect!(relm, about, connect_activate(_), MenuAbout);

        menu_file.append(&file_item);
        menu_file.append(&quit);
        file.set_submenu(Some(&menu_file));

        menu_sort.append(&year);
        menu_sort.append(&country);
        sort.set_submenu(&menu_sort);

        menu_help.append(&about);
        help.set_submenu(&menu_help);

        menu_bar.append(&file);
        menu_bar.append(&sort);
        menu_bar.append(&help);
        menu_bar.show_all();

        MyMenuBar { bar: menu_bar }
    }
}

#[derive(Clone)]
struct MyViewPort {
    model: ViewModel,
    view: Viewport,
    tree: TreeView,
}

#[derive(Clone)]
pub struct ViewModel {
    visits: Visits,
    order: SortBy,
}

#[derive(Msg)]
pub enum ViewMsg {
    UpdateView(Visits),
    SortChanged(SortBy),
}

impl Update for MyViewPort {
    type Model = ViewModel;
    type ModelParam = ();
    type Msg = ViewMsg;

    fn model(_: &Relm<Self>, _: ()) -> ViewModel {
        ViewModel {
            visits: Visits::new(),
            order: SortBy::Year,
        }
    }

    fn update(&mut self, event: ViewMsg) {
        match event {
            UpdateView(visits) => {
                self.model.visits = visits;
                self.update_tree_model();
            }
            SortChanged(order) => {
                self.model.order = order;
                self.update_tree_model();
            }
        }
    }
}

impl Widget for MyViewPort {
    type Root = Viewport;

    fn root(&self) -> Self::Root {
        self.view.clone()
    }

    fn view(_relm: &Relm<Self>, model: Self::Model) -> Self {
        let view = Viewport::new(None, None);
        let tree = TreeView::new();
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

        view.add(&tree);

        view.show_all();

        MyViewPort { model, view, tree }
    }
}

impl MyViewPort {
    fn update_tree_model(&self) {
        match self.model.order {
            SortBy::Year => self.model.visits.set_year_model(&self.tree),
            SortBy::Country => self.model.visits.set_country_model(&self.tree),
        }
    }
}

#[derive(Msg)]
pub enum Msg {
    FileDialog,
    AboutDialog,
    Quit,
}

#[derive(Clone)]
pub enum SortBy {
    Country,
    Year,
}

#[widget]
impl Widget for Win {
    // The initial model.
    fn model() -> () {}

    // Update the model according to the message received.
    fn update(&mut self, event: Msg) {
        match event {
            FileDialog => {
                if let Some(x) = self.file_dialog() {
                    self.view.emit(UpdateView(self.load_json(x)));
                };
            }
            AboutDialog => self.about_dialog(),
            Quit => gtk::main_quit(),
        }
    }

    view! {
        #[name="root"]
        gtk::Window {
            title: "Country Parser",
            gtk::Box {
                // Set the orientation property of the Box.
                orientation: Vertical,
                MyMenuBar {
                    SelectFile => FileDialog,
                    SortOrder(ref x) => view@SortChanged(x.clone()),
                    MenuAbout => AboutDialog,
                    MenuQuit => Quit,
                },
                gtk::ScrolledWindow {
                    packing: {
                        expand: true,
                    },
                    #[name="view"]
                    MyViewPort,
                },
            },
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}

impl Win {
    fn file_dialog(&self) -> Option<PathBuf> {
        let dialog = FileChooserDialog::new::<gtk::Window>(
            Some("Import File"),
            Some(&self.root()),
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

    fn about_dialog(&self) {
        let dialog = gtk::AboutDialog::new();
        dialog.set_transient_for(&self.root());
        dialog.set_modal(true);
        dialog.set_authors(&["Eric Trombly"]);
        dialog.set_program_name("Country Parser");
        dialog.set_comments("Find out where you've been");
        if let Ok(logo) = gdk_pixbuf::Pixbuf::new_from_file("Antu_map-globe.ico") {
            dialog.set_logo(Some(&logo));
        };
        dialog.run();
        dialog.destroy();
    }

    fn load_json(&self, path: PathBuf) -> Visits {
        // set up progress bar
        let mut visits = Visits::new();
        let dialog = Dialog::new_with_buttons(
            Some("Processing Location History"),
            Some(&self.root()),
            gtk::DIALOG_MODAL | gtk::DIALOG_DESTROY_WITH_PARENT,
            &[],
        );

        let content = dialog.get_content_area();
        let progress = ProgressBar::new();
        content.pack_start(&progress, true, true, 0);
        progress.set_fraction(0.0);
        dialog.show_all();

        // read json file
        let mut contents = String::new();
        File::open(path)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();
        let locations = location_history::deserialize(&contents).filter_outliers();

        // read country borders
        let encoded = include_bytes!("countries.bin");
        let countries: Vec<Country> = deserialize(&encoded[..]).unwrap();

        // empty country to start with
        let mut last_country = Country::default();

        let total = locations.len();

        for (i, loc) in locations.into_iter().enumerate() {
            progress.set_fraction(i as f64 / total as f64);
            gtk::main_iteration_do(false);
            let tmp = geo::Point::new(loc.longitude, loc.latitude);
            if !(last_country.bb.contains(&tmp) && last_country.shapes.iter().any(|x| x.contains(&tmp))) {
                for country in &countries {
                    if country.bb.contains(&tmp) && country.shapes.iter().any(|x| x.contains(&tmp)) {
                        if let Some(visit) = visits.last_mut() {
                            visit.end = Some(loc.timestamp);
                        }
                        visits.push(Visit::new(country.clone(), loc.timestamp, None));
                        last_country = country.clone();
                    } else {
                        println!("couldn't find {} {:?}",
                                loc.timestamp.format("%Y-%m-%d").to_string(), tmp);
                    }
                }
            }
        }

        dialog.destroy();
        visits
    }
}

fn main() {
    Win::run(()).unwrap();
}
