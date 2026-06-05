use crate::settings::save_device;
use adw::prelude::*;
use adw::{AboutDialog, ApplicationWindow, EntryRow, PreferencesDialog, Toast};
use gettextrs::gettext;
use gtk::{Builder, Button, glib};

pub fn show_add(parent: &ApplicationWindow) {
    let builder = Builder::from_resource("/edu/unesum/umbral/ui/add_classroom_dialog.ui");
    let dialog: PreferencesDialog = builder.object("add_dialog").unwrap();
    let name: EntryRow = builder.object("entry_name").unwrap();
    let mac: EntryRow = builder.object("entry_mac").unwrap();
    let btn_dialog_add: Button = builder.object("btn_dialog_add").unwrap();

    let validate_form = glib::clone!(
        #[weak]
        name,
        #[weak]
        mac,
        #[weak]
        btn_dialog_add,
        move || {
            let is_name_ready = !name.text().to_string().trim().is_empty();
            let is_mac_ready = mac.text().to_string().len() == 17;
            btn_dialog_add.set_sensitive(is_name_ready && is_mac_ready);
        }
    );

    name.connect_changed(glib::clone!(
        #[strong]
        validate_form,
        move |_| {
            validate_form();
        }
    ));

    mac.connect_changed(glib::clone!(
        #[weak]
        mac,
        #[strong]
        validate_form,
        move |_| {
            let original_text = mac.text().to_string();
            let g_editable = mac.upcast_ref::<gtk::Editable>();
            let original_position = g_editable.position();
            let mut clean_text = String::with_capacity(12);
            for c in original_text.chars().filter(|c| c.is_ascii_hexdigit()) {
                if clean_text.len() == 12 {
                    break;
                }
                clean_text.push(c.to_ascii_uppercase());
            }

            let mut formatted_text = String::with_capacity(17);
            for (i, ch) in clean_text.chars().enumerate() {
                if i > 0 && i % 2 == 0 {
                    formatted_text.push(':');
                }
                formatted_text.push(ch);
            }

            if original_text != formatted_text {
                let mut chars_before: usize = 0;
                for (i, ch) in original_text.chars().enumerate() {
                    if (i as i32) >= original_position {
                        break;
                    }
                    if ch.is_ascii_hexdigit() {
                        chars_before += 1;
                    }
                }

                let mut new_position = chars_before + (chars_before.saturating_sub(1) / 2);
                let formatted_len = formatted_text.chars().count();

                if new_position > 0 && new_position % 3 == 2 && new_position == formatted_len - 1 {
                    new_position += 1;
                }

                if new_position > 0 && original_text.len() > formatted_text.len() {
                    if formatted_text.chars().nth(new_position - 1) == Some(':') {
                        new_position -= 1;
                    }
                }

                glib::idle_add_local({
                    let mac = mac.clone();
                    let formatted_text = formatted_text.clone();
                    move || {
                        let g_editable = mac.upcast_ref::<gtk::Editable>();
                        mac.set_text(&formatted_text);
                        g_editable.set_position(new_position as i32);
                        glib::ControlFlow::Break
                    }
                });
            }

            validate_form();
        }
    ));

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
                let name_text = name.text().to_string().trim().to_string();
                let mac_text = mac.text().to_string().trim().to_string();
                match save_device(&name_text, &mac_text) {
                    Ok(()) => {
                        let msg = gettext("Classroom added successfully");
                        dialog.add_toast(Toast::builder().title(&msg).timeout(2).build());
                        name.set_text("");
                        mac.set_text("");
                    }
                    Err(err) => {
                        let msg = format!("{}: {}", gettext("Error saving"), err);
                        dialog.add_toast(Toast::builder().title(&msg).timeout(2).build());
                    }
                }
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
