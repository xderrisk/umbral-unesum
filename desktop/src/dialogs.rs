use adw::prelude::*;
use adw::{AboutDialog, ApplicationWindow, EntryRow, PreferencesDialog, Toast};
use gtk::{Builder, Button, glib};

pub fn show_add(parent: &ApplicationWindow) {
    let builder = Builder::from_resource("/edu/unesum/umbral/ui/add_classroom_dialog.ui");
    let dialog: PreferencesDialog = builder.object("add_dialog").unwrap();
    let name: EntryRow = builder.object("entry_name").unwrap();
    let mac: EntryRow = builder.object("entry_mac").unwrap();

    builder
        .object::<Button>("btn_dialog_add")
        .unwrap()
        .connect_clicked(glib::clone!(
            #[weak]
            name,
            #[weak]
            mac,
            #[weak]
            dialog,
            move |_| {
                let is_empty = name.text().is_empty() || mac.text().is_empty();
                let msg = if is_empty {
                    "Please fill in all fields"
                } else {
                    "Device added successfully"
                };
                dialog.add_toast(Toast::builder().title(msg).timeout(2).build());
            }
        ));
    dialog.present(Some(parent));
}

pub fn show_about(parent: &ApplicationWindow) {
    let builder = Builder::from_resource("/edu/unesum/umbral/ui/about_dialog.ui");
    let dialog: AboutDialog = builder
        .object("about_dialog")
        .expect("No about_dialog found");
    dialog.present(Some(parent));
}

pub fn show_config(parent: &ApplicationWindow) {
    let builder = Builder::from_resource("/edu/unesum/umbral/ui/settings_dialog.ui");
    let dialog: PreferencesDialog = builder
        .object("config_dialog")
        .expect("No config_dialog found");
    dialog.present(Some(parent));
}
