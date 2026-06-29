use crate::state::SharedState;
use adw::prelude::*;
use gettextrs::gettext;
use gtk::glib;
use std::rc::Rc;

pub fn show_config(
    parent: &adw::ApplicationWindow,
    state: &SharedState,
    left_container: gtk::Box,
    separator: gtk::Separator,
    classrooms_relayout: Rc<dyn Fn()>,
) {
    let builder = gtk::Builder::from_resource("/edu/unesum/umbral/ui/settings_dialog.ui");
    let dialog: adw::PreferencesDialog = builder
        .object("settings_dialog")
        .expect("Settings dialog object not found in UI file");
    let switch_news: adw::SwitchRow = builder
        .object("switch_news")
        .expect("News switch row not found in UI file");
    let entry_api_key: adw::EntryRow = builder
        .object("entry_api_key")
        .expect("API key entry row not found in UI file");

    {
        let current_settings = &state.borrow().settings;
        entry_api_key.set_text(&current_settings.api_key);
        switch_news.set_active(current_settings.news);
    }

    entry_api_key.connect_changed(glib::clone!(
        #[strong]
        state,
        move |entry| {
            let mut current_state = state.borrow_mut();
            current_state.settings.api_key = entry.text().trim().to_string();
            if let Err(e) = crate::settings::save(&current_state.settings) {
                eprintln!("{}: {}", gettext("Error saving API Key"), e);
            }
        }
    ));

    switch_news.connect_active_notify(glib::clone!(
        #[strong]
        state,
        #[weak]
        parent,
        #[weak]
        left_container,
        #[weak]
        separator,
        #[strong]
        classrooms_relayout,
        move |sw| {
            let is_active = sw.is_active();
            let mut current_state = state.borrow_mut();
            current_state.settings.news = is_active;
            if let Err(e) = crate::settings::save(&current_state.settings) {
                eprintln!("Error saving setting: {}", e);
            }
            drop(current_state);
            classrooms_relayout();
            glib::MainContext::default().invoke_local(glib::clone!(move || {
                let (current_width, current_height) = parent.default_size();
                left_container.set_visible(is_active);
                separator.set_visible(is_active);
                if is_active {
                    parent.set_default_size(current_width + 360, current_height);
                } else {
                    if current_width > 360 {
                        parent.set_default_size(current_width - 360, current_height);
                    }
                }
            }));
        }
    ));

    dialog.present(Some(parent));
}
