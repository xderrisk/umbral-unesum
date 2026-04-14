use crate::settings;
use adw::prelude::*;
use gtk::{Align, Box, Label, Orientation};

pub fn classroms_section() -> Box {
    let container = Box::builder().orientation(Orientation::Vertical).build();
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
            let mac = aula["mac"].as_str().unwrap_or("");
            let estado = aula["estado"].as_str().unwrap_or("Desconectado");

            let card = Box::builder()
                .name(mac)
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
    container
}
