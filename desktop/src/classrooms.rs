use crate::settings;
use adw::prelude::*;
use async_channel;
use gtk::glib;
use gtk::{Align, Box, Label, Orientation};
use std::collections::HashMap;

pub fn classrooms_section() -> Box {
    let container = Box::builder().orientation(Orientation::Vertical).build();
    let mut labels_map: HashMap<String, (Label, Box)> = HashMap::new();
    let (sender, receiver) = async_channel::unbounded::<crate::mqtt::AulaUpdate>();
    let title = Label::builder()
        .label("Estado de Aulas")
        .css_classes(vec!["heading".to_string()])
        .margin_bottom(20)
        .build();
    container.append(&title);

    let aulas = settings::load_aulas();
    if !aulas.is_empty() {
        for aula in aulas {
            let nombre = aula["nombre"].as_str().unwrap_or("Aula");
            let mac = aula["mac"].as_str().unwrap_or("").to_string();
            let estado = aula["estado"].as_str().unwrap_or("Desconectado");

            let card = Box::builder()
                .name(&mac)
                .orientation(Orientation::Vertical)
                .margin_start(20)
                .margin_end(20)
                .margin_bottom(20)
                .build();
            card.add_css_class("aula-cuadro");

            let content = Box::builder()
                .orientation(Orientation::Vertical)
                .valign(Align::Center)
                .vexpand(true)
                .build();

            let label = Label::builder().label(nombre).build();
            label.add_css_class("aula-nombre");

            let status_label = Label::builder()
                .name("status-label")
                .label(estado)
                .css_classes(vec!["dim-label".to_string()])
                .build();
            labels_map.insert(mac.clone(), (status_label.clone(), card.clone()));
            content.append(&label);
            content.append(&status_label);
            card.append(&content);
            container.append(&card);
        }
    } else {
        let label = Label::builder()
            .label("Agregue Aulas")
            .margin_start(50)
            .margin_end(50)
            .build();
        container.append(&label);
    }
    let main_context = glib::MainContext::default();
    main_context.spawn_local(async move {
        while let Ok(update) = receiver.recv().await {
            if let Some((label, card)) = labels_map.get(&update.mac) {
                let txt = if update.estado == "1" {
                    "Ocupado"
                } else {
                    "Libre"
                };
                label.set_label(txt);

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
