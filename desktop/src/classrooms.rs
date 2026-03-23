use crate::config;
use adw::prelude::*;
use gtk::{Align, Box, Label, Orientation};

pub fn classroms_section() -> Box {
    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .hexpand(true)
        .valign(Align::Start)
        .build();

    let tittle = Label::builder()
        .label("Estado de Aulas")
        .css_classes(vec!["heading".to_string()])
        .build();
    container.append(&tittle);

    let group = adw::PreferencesGroup::builder().build();
    let aulas = config::load_aulas();
    for aula in aulas {
        let nombre = aula["nombre"].as_str().unwrap_or("Aula");
        let estado = aula["estado"].as_str().unwrap_or("Desconectado");

        let card = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(10)
            .halign(Align::Center)
            .valign(Align::Center)
            .margin_start(10)
            .margin_end(10)
            .margin_top(5)
            .margin_bottom(8)
            .build();
        card.add_css_class("aula-cuadro");
        let label = Label::builder()
            .label(nombre)
            .valign(Align::Center)
            .build();
        label.add_css_class("aula-nombre");

        let status_label = Label::builder()
            .label(estado)
            .valign(Align::Center)
            .css_classes(vec!["dim-label".to_string()])
            .build();
        card.append(&label);
        card.append(&status_label);
        group.add(&card);
    }
    container.append(&group);
    container
}
