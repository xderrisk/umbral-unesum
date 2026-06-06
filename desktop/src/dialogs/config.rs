use adw::prelude::*;
use gettextrs::gettext;

pub fn show_config(parent: &adw::ApplicationWindow) {
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

    if let Some(saved_key) = crate::settings::load_api_key() {
        entry_api_key.set_text(&saved_key);
    }

    entry_api_key.connect_changed(move |entry| {
        let current_key = entry.text().to_string();
        if let Err(e) = crate::settings::save_api_key(&current_key) {
            eprintln!("{}: {}", gettext("Error saving API Key"), e);
        } else {
            println!("{}", gettext("API Key updated in local configuration."));
        }
    });

    switch_news.connect_notify(Some("active"), move |sw, _| {
        if sw.is_active() {
            println!("{}", gettext("Hide news: ENABLED"));
        } else {
            println!("{}", gettext("Hide news: DISABLED"));
        }
    });

    dialog.present(Some(parent));
}
