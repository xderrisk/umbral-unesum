use crate::settings;
use adw::prelude::*;
use async_channel;
use gettextrs::gettext;
use gtk::glib;
use std::collections::HashMap;

pub fn classrooms_section() -> gtk::Box {
    let builder = gtk::Builder::from_resource("/edu/unesum/umbral/ui/classrooms_section.ui");
    let container: gtk::Box = builder.object("classrooms_panel").unwrap();
    let classrooms_flowbox: gtk::FlowBox = builder.object("classrooms_flowbox").unwrap();
    let classrooms_stack: gtk::Stack = builder.object("classrooms_stack").unwrap();
    let empty_label: gtk::Label = builder.object("empty_label").unwrap();

    let mut labels_map: HashMap<String, (gtk::Label, gtk::Grid)> = HashMap::new();
    let (sender, receiver) = async_channel::unbounded::<crate::mqtt::ClassroomUpdate>();

    let classrooms = settings::load_classrooms();
    let total_classrooms = classrooms.len();
    if total_classrooms > 0 {
        classrooms_stack.set_visible_child(&classrooms_flowbox);
        let max_columns = (total_classrooms + 2) / 3;
        classrooms_flowbox.set_max_children_per_line(max_columns as u32);

        for classroom in classrooms {
            let name = classroom["name"].as_str().unwrap_or("");
            let mac = classroom["mac"].as_str().unwrap_or("").to_string();
            let status = classroom["status"].as_str().unwrap_or("");

            let card_builder =
                gtk::Builder::from_resource("/edu/unesum/umbral/ui/classroom_card.ui");

            let card_grid: gtk::Grid = card_builder
                .object("classroom_card_root")
                .expect("Error: 'classroom_card_root' not found in the UI file.");

            let name_label: gtk::Label = card_builder.object("name_label").unwrap();
            let status_label: gtk::Label = card_builder.object("status_label").unwrap();

            name_label.set_label(name);

            let initial_status = match status {
                "0" => {
                    card_grid.add_css_class("classroom-available");
                    gettext("Available")
                }
                "1" => {
                    card_grid.add_css_class("classroom-occupied");
                    gettext("Occupied")
                }
                _ => gettext("Offline"),
            };
            status_label.set_label(&initial_status);

            labels_map.insert(mac.clone(), (status_label.clone(), card_grid.clone()));
            classrooms_flowbox.insert(&card_grid, -1);
        }
    } else {
        classrooms_stack.set_visible_child(&empty_label);
    }

    let main_context = glib::MainContext::default();
    main_context.spawn_local(async move {
        while let Ok(update) = receiver.recv().await {
            if let Some((label, card)) = labels_map.get(&update.mac) {
                let status_text = match update.status.as_str() {
                    "0" => {
                        card.add_css_class("classroom-available");
                        card.remove_css_class("classroom-occupied");
                        gettext("Available")
                    }
                    "1" => {
                        card.add_css_class("classroom-occupied");
                        card.remove_css_class("classroom-available");
                        gettext("Occupied")
                    }
                    _ => {
                        card.remove_css_class("classroom-available");
                        card.remove_css_class("classroom-occupied");
                        gettext("Offline")
                    }
                };
                label.set_label(&status_text);
            }
        }
    });

    crate::mqtt::init(sender);
    container
}
