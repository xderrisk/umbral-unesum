use crate::settings;
use adw::prelude::*;
use async_channel;
use gettextrs::gettext;
use gtk::glib;
use std::collections::HashMap;

pub fn classrooms_section() -> gtk::Box {
    let builder = gtk::Builder::from_resource("/edu/unesum/umbral/ui/classrooms_section.ui");
    let container: gtk::Box = builder.object("classrooms_panel").unwrap();
    let classrooms_grid: gtk::Grid = builder.object("classrooms_grid").unwrap();
    let classrooms_stack: gtk::Stack = builder.object("classrooms_stack").unwrap();
    let empty_label: gtk::Label = builder.object("empty_label").unwrap();

    let mut labels_map: HashMap<String, (gtk::Label, gtk::Grid)> = HashMap::new();
    let (sender, receiver) = async_channel::unbounded::<crate::mqtt::ClassroomUpdate>();

    let classrooms = settings::load_classrooms();

    if !classrooms.is_empty() {
        classrooms_stack.set_visible_child(&classrooms_grid);

        let max_columns = 1;
        let mut current_col = 0;
        let mut current_row = 0;

        for classroom in classrooms {
            let name = classroom["name"].as_str().unwrap_or("Classroom");
            let mac = classroom["mac"].as_str().unwrap_or("").to_string();
            let status = classroom["status"].as_str().unwrap_or("Disconnected");

            let card_builder =
                gtk::Builder::from_resource("/edu/unesum/umbral/ui/classroom_card.ui");

            let card_grid: gtk::Grid = card_builder
                .object("classroom_card_root")
                .expect("Error: 'classroom_card_root' not found in the UI file.");

            let name_label: gtk::Label = card_builder.object("name_label").unwrap();
            let status_label: gtk::Label = card_builder.object("status_label").unwrap();

            name_label.set_label(name);

            let initial_status = if status == "1" {
                gettext("Occupied")
            } else {
                gettext("Available")
            };
            status_label.set_label(&initial_status);

            if status == "1" {
                card_grid.add_css_class("classroom-occupied");
            } else {
                card_grid.add_css_class("classroom-available");
            }

            labels_map.insert(mac.clone(), (status_label.clone(), card_grid.clone()));
            classrooms_grid.attach(&card_grid, current_col, current_row, 1, 1);

            current_col += 1;
            if current_col >= max_columns {
                current_col = 0;
                current_row += 1;
            }
        }
    } else {
        classrooms_stack.set_visible_child(&empty_label);
    }

    let main_context = glib::MainContext::default();
    main_context.spawn_local(async move {
        while let Ok(update) = receiver.recv().await {
            if let Some((label, card)) = labels_map.get(&update.mac) {
                let status_text = if update.status == "1" {
                    gettext("Occupied")
                } else {
                    gettext("Available")
                };
                label.set_label(&status_text);

                if update.status == "1" {
                    card.add_css_class("classroom-occupied");
                    card.remove_css_class("classroom-available");
                } else {
                    card.add_css_class("classroom-available");
                    card.remove_css_class("classroom-occupied");
                }
            }
        }
    });

    crate::mqtt::init(sender);
    container
}
