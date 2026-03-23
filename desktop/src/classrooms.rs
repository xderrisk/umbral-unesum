use crate::config;
use adw::prelude::*;
use gtk::{Align, Box, Label, Orientation};

pub fn classroms_section() -> Box {
    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    let title = Label::builder()
        .label("Estado de Aulas")
        .css_classes(vec!["heading".to_string()])
        .build();
    container.append(&title);

    let aulas = config::load_aulas();
    for aula in aulas {
        let nombre = aula["nombre"].as_str().unwrap_or("Aula");
        let estado = aula["estado"].as_str().unwrap_or("Desconectado");

        let card = Box::builder()
            .orientation(Orientation::Vertical)
            .margin_start(10)
            .margin_end(10)
            .margin_top(5)
            .margin_bottom(8)
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
            .label(estado)
            .css_classes(vec!["dim-label".to_string()])
            .build();

        content.append(&label);
        content.append(&status_label);
        card.append(&content);
        container.append(&card);
    }
    container
}
