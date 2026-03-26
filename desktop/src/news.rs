use crate::config;
use gtk::glib;
use adw::prelude::*;
use std::cell::RefCell;
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

    let images = config::list_news_images();
    if images.is_empty() {
        let placeholder = Label::builder()
            .label("No hay imágenes de noticias disponibles")
            .vexpand(true)
            .css_classes(vec!["dim-label".to_string()])
            .build();
        placeholder.set_margin_start(20);
        placeholder.set_margin_end(20);
        container.append(&placeholder);
        return container;
    }

    let textures: Vec<gtk::gdk::Texture> = images
        .iter()
        .filter_map(|path| {
            let file = gtk::gio::File::for_path(path);
            gtk::gdk::Texture::from_file(&file).ok()
        })
        .collect();

    if textures.is_empty() {
        let placeholder = Label::builder()
            .label("No se pudieron cargar las imágenes")
            .vexpand(true)
            .css_classes(vec!["dim-label".to_string()])
            .build();
        placeholder.set_margin_start(20);
        placeholder.set_margin_end(20);
        container.append(&placeholder);
        return container;
    }

    let picture = Picture::builder()
        .valign(Align::Center)
        .vexpand(true)
        .build();
    picture.set_margin_top(20);
    picture.set_margin_bottom(20);
    picture.set_margin_start(20);
    picture.set_margin_end(20);
    picture.add_css_class("news-image");

    let current_index = RefCell::new(0);
    picture.set_paintable(Some(&textures[0]));
    container.append(&picture);

    let textures_clone = textures.clone();
    glib::timeout_add_local(
        std::time::Duration::from_secs(10),
        move || {
            let mut idx = current_index.borrow_mut();
            *idx = (*idx + 1) % textures_clone.len();
            picture.set_paintable(Some(&textures_clone[*idx]));
            glib::ControlFlow::Continue
        },
    );
    container
}
