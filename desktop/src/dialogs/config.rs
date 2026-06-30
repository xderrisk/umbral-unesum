use crate::state::SharedState;
use adw::prelude::*;
use gettextrs::gettext;
use gtk::glib;
use std::cell::Cell;
use std::rc::Rc;

pub fn show_config(
    parent: &adw::ApplicationWindow,
    state: &SharedState,
    classrooms_relayout: Rc<dyn Fn()>,
    apply_visibility: Rc<dyn Fn()>,
) {
    let builder = gtk::Builder::from_resource("/edu/unesum/umbral/ui/settings_dialog.ui");
    let dialog: adw::PreferencesDialog = builder
        .object("settings_dialog")
        .expect("Settings dialog object not found in UI file");
    let switch_news: adw::SwitchRow = builder
        .object("switch_news")
        .expect("News switch row not found in UI file");
    let switch_unique_image: adw::SwitchRow = builder
        .object("switch_unique_image")
        .expect("Unique image switch row not found in UI file");
    let entry_api_key: adw::EntryRow = builder
        .object("entry_api_key")
        .expect("API key entry row not found in UI file");
    let btn_ui_browse: gtk::Button = builder
        .object("btn_ui_browse")
        .expect("Browse button not found");
    let btn_ui_clear: gtk::Button = builder
        .object("btn_ui_clear")
        .expect("Clear button not found");
    let ui_action_row: adw::ActionRow = builder
        .object("ui_action_row")
        .expect("UI action row not found");

    {
        let s = state.borrow();
        entry_api_key.set_text(&s.settings.api_key);
        switch_news.set_active(s.settings.news);
        switch_unique_image.set_active(s.settings.unique_image);
        switch_news.set_sensitive(!s.settings.unique_image);
        switch_unique_image.set_sensitive(!s.settings.news);
        update_path_subtitle(&ui_action_row, &s.settings.unique_image_path);
    }

    let changing = Rc::new(Cell::new(false));

    entry_api_key.connect_changed(glib::clone!(
        #[strong]
        state,
        move |entry| {
            let mut s = state.borrow_mut();
            s.settings.api_key = entry.text().trim().to_string();
            if let Err(e) = crate::settings::save(&s.settings) {
                eprintln!("{}: {}", gettext("Error saving API Key"), e);
            }
        }
    ));

    switch_news.connect_active_notify(glib::clone!(
        #[strong]
        state,
        #[weak]
        switch_unique_image,
        #[strong]
        classrooms_relayout,
        #[strong]
        apply_visibility,
        #[strong]
        changing,
        move |sw| {
            if changing.get() {
                return;
            }
            let is_active = sw.is_active();
            {
                let mut s = state.borrow_mut();
                s.settings.news = is_active;
                if is_active {
                    s.settings.unique_image = false;
                }
                if let Err(e) = crate::settings::save(&s.settings) {
                    eprintln!("Error saving setting: {}", e);
                }
            }
            changing.set(true);
            switch_unique_image.set_active(false);
            switch_unique_image.set_sensitive(!is_active);
            changing.set(false);
            classrooms_relayout();
            apply_visibility();
        }
    ));

    switch_unique_image.connect_active_notify(glib::clone!(
        #[strong]
        state,
        #[weak]
        switch_news,
        #[strong]
        classrooms_relayout,
        #[strong]
        apply_visibility,
        #[strong]
        changing,
        move |sw| {
            if changing.get() {
                return;
            }
            let is_active = sw.is_active();
            {
                let mut s = state.borrow_mut();
                s.settings.unique_image = is_active;
                if is_active {
                    s.settings.news = false;
                }
                if let Err(e) = crate::settings::save(&s.settings) {
                    eprintln!("Error saving setting: {}", e);
                }
            }
            changing.set(true);
            switch_news.set_active(false);
            switch_news.set_sensitive(!is_active);
            changing.set(false);
            classrooms_relayout();
            apply_visibility();
        }
    ));

    let parent_window = parent.clone();
    btn_ui_browse.connect_clicked(glib::clone!(
        #[strong]
        state,
        #[weak]
        ui_action_row,
        #[strong]
        apply_visibility,
        move |_| {
            let chooser = gtk::FileChooserNative::new(
                Some("Select unique image"),
                Some(&parent_window),
                gtk::FileChooserAction::Open,
                Some("Select"),
                Some("Cancel"),
            );
            let filter = gtk::FileFilter::new();
            filter.add_mime_type("image/jpeg");
            filter.add_mime_type("image/png");
            filter.add_mime_type("image/gif");
            filter.add_mime_type("image/webp");
            chooser.set_filter(&filter);

            let state = state.clone();
            let ui_row = ui_action_row.clone();
            let on_updated = apply_visibility.clone();
            chooser.connect_response(glib::clone!(
                #[strong]
                state,
                #[weak]
                ui_row,
                #[strong]
                on_updated,
                move |chooser, response| {
                    if response == gtk::ResponseType::Accept {
                        if let Some(file) = chooser.file() {
                            if let Some(path) = file.path() {
                                let path_str = path.to_string_lossy().to_string();
                                {
                                    let mut s = state.borrow_mut();
                                    s.settings.unique_image_path = path_str.clone();
                                    if let Err(e) = crate::settings::save(&s.settings) {
                                        eprintln!("Error saving image path: {}", e);
                                    }
                                }
                                update_path_subtitle(&ui_row, &path_str);
                                on_updated();
                            }
                        }
                    }
                }
            ));
            chooser.show();
        }
    ));

    btn_ui_clear.connect_clicked(glib::clone!(
        #[strong]
        state,
        #[weak]
        ui_action_row,
        #[strong]
        apply_visibility,
        move |_| {
            {
                let mut s = state.borrow_mut();
                s.settings.unique_image_path = String::new();
                if let Err(e) = crate::settings::save(&s.settings) {
                    eprintln!("Error clearing image path: {}", e);
                }
            }
            update_path_subtitle(&ui_action_row, "");
            apply_visibility();
        }
    ));

    dialog.present(Some(parent));
}

fn update_path_subtitle(row: &adw::ActionRow, path: &str) {
    if path.is_empty() {
        row.set_subtitle("No file selected");
    } else {
        let name = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path);
        row.set_subtitle(name);
    }
}
