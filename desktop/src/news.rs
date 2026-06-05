use crate::settings;
use adw::prelude::*;
use gettextrs::gettext;
use gtk::glib;
use gtk::{Box, Builder, Label, Picture};
use std::cell::RefCell;

pub fn news_section() -> Box {
    let builder = Builder::from_resource("/edu/unesum/umbral/ui/news_section.ui");

    let container: Box = builder
        .object("news_dialog")
        .expect("Error: 'news_dialog' not found in the UI file.");

    let news_picture: Picture = builder
        .object("news_picture")
        .expect("Error: 'news_picture' not found in the UI file.");

    let placeholder_label: Label = builder
        .object("placeholder_label")
        .expect("Error: 'placeholder_label' not found in the UI file.");

    let images = settings::list_news_images();

    if images.is_empty() {
        placeholder_label.set_label(&gettext("No news images available"));
        placeholder_label.set_visible(true);
        news_picture.set_visible(false);
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
        placeholder_label.set_label(&gettext("Failed to load images"));
        placeholder_label.set_visible(true);
        news_picture.set_visible(false);
        return container;
    }

    let current_index = RefCell::new(0);
    news_picture.set_paintable(Some(&textures[0]));

    let textures_clone = textures.clone();
    glib::timeout_add_local(std::time::Duration::from_secs(10), move || {
        let mut idx = current_index.borrow_mut();
        *idx = (*idx + 1) % textures_clone.len();
        news_picture.set_paintable(Some(&textures_clone[*idx]));
        glib::ControlFlow::Continue
    });

    container
}
