use crate::config;
use adw::prelude::*;
use gtk::{Align, Box, Label, Orientation, Picture};

pub fn news_section() -> Box {
    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .hexpand(true)
        .build();

    let tittle = Label::builder()
        .label("Noticias")
        .css_classes(vec!["heading".to_string()])
        .build();
    container.append(&tittle);

    let picture = Picture::builder().build();
    let img = config::news_folder();
    picture.set_filename(Some(img));
    picture.set_margin_top(20);
    picture.set_margin_bottom(20);
    picture.set_margin_start(20);
    picture.set_margin_end(20);
    picture.add_css_class("news-image");
    let picture_container = Box::builder()
        .orientation(Orientation::Vertical)
        .vexpand(true)
        .valign(Align::Center)
        .build();
    picture_container.append(&picture);
    container.append(&picture_container);
    container
}
