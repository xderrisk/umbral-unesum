use crate::settings;
use adw::prelude::*;
use async_channel;
use gettextrs::gettext;
use gtk::glib;
use gtk::{Box, Builder, Grid, Label, Stack};
use std::collections::HashMap;

pub fn classrooms_section() -> Box {
    let builder = Builder::from_resource("/edu/unesum/umbral/ui/classrooms_section.ui");
    let container: Box = builder.object("classrooms_panel").unwrap();
    let classrooms_grid: Grid = builder.object("classrooms_grid").unwrap();
    let classrooms_stack: Stack = builder.object("classrooms_stack").unwrap();
    let empty_label: Label = builder.object("empty_label").unwrap();

    let mut labels_map: HashMap<String, (Label, Grid)> = HashMap::new();
    let (sender, receiver) = async_channel::unbounded::<crate::mqtt::AulaUpdate>();

    let aulas = settings::load_aulas();

    if !aulas.is_empty() {
        classrooms_stack.set_visible_child(&classrooms_grid);

        let max_columns = 1;
        let mut current_col = 0;
        let mut current_row = 0;

        for aula in aulas {
            let nombre = aula["nombre"].as_str().unwrap_or("Classroom");
            let mac = aula["mac"].as_str().unwrap_or("").to_string();
            let estado = aula["estado"].as_str().unwrap_or("Disconnected");
            let card_builder = Builder::from_resource("/edu/unesum/umbral/ui/classroom_card.ui");
            let card_grid: Grid = card_builder
                .object("aula_tarjeta_root")
                .expect("Error: No se encontró 'aula_tarjeta_root' en el archivo UI.");
            let name_label: Label = card_builder.object("name_label").unwrap();
            let status_label: Label = card_builder.object("status_label").unwrap();
            name_label.set_label(nombre);
            let initial_status = if estado == "1" {
                gettext("Occupied")
            } else {
                gettext("Available")
            };
            status_label.set_label(&initial_status);
            if estado == "1" {
                card_grid.add_css_class("aula-ocupada");
            } else {
                card_grid.add_css_class("aula-libre");
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
                let txt = if update.estado == "1" {
                    gettext("Occupied")
                } else {
                    gettext("Available")
                };
                label.set_label(&txt);

                if update.estado == "1" {
                    card.add_css_class("aula-ocupada");
                    card.remove_css_class("aula-libre");
                } else {
                    card.add_css_class("aula-libre");
                    card.remove_css_class("aula-ocupada");
                }
            }
        }
    });

    crate::mqtt::init(sender);
    container
}
