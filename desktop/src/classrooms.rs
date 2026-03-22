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
    container
}
