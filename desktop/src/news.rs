use crate::settings;
use adw::prelude::*;
use gettextrs::gettext;
use gtk::glib;
use std::cell::RefCell;

pub fn news_section() -> gtk::Box {
    let media = gtk::Builder::from_resource("/edu/unesum/umbral/ui/media_section.ui");

    let media_picture: gtk::Picture = media
        .object("media_picture")
        .expect("Error: 'media_picture' not found in the UI file.");

    let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    container.set_hexpand(true);

    let heading = gtk::Label::new(Some(&gettext("News")));
    heading.add_css_class("heading");
    container.append(&heading);

    container.append(&media_picture);

    let placeholder_label = gtk::Label::new(None);
    placeholder_label.add_css_class("dim-label");
    placeholder_label.set_vexpand(true);
    placeholder_label.set_margin_start(20);
    placeholder_label.set_margin_end(20);
    placeholder_label.set_visible(false);
    container.append(&placeholder_label);

    let images = settings::list_news_images();

    if images.is_empty() {
        placeholder_label.set_label(&gettext("No news images available"));
        placeholder_label.set_visible(true);
        media_picture.set_visible(false);
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
        media_picture.set_visible(false);
        return container;
    }

    let current_index = RefCell::new(0);
    media_picture.set_paintable(Some(&textures[0]));

    let textures_clone = textures.clone();
    glib::timeout_add_local(std::time::Duration::from_secs(10), move || {
        let mut idx = current_index.borrow_mut();
        *idx = (*idx + 1) % textures_clone.len();
        media_picture.set_paintable(Some(&textures_clone[*idx]));
        glib::ControlFlow::Continue
    });

    container
}
