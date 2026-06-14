use crate::state::SharedState;
use adw::prelude::*;
use gettextrs::gettext;
use gtk::glib;

pub fn show_add(parent: &adw::ApplicationWindow, state: &SharedState) {
    let builder = gtk::Builder::from_resource("/edu/unesum/umbral/ui/add_classroom_dialog.ui");
    let dialog: adw::PreferencesDialog = builder.object("add_dialog").unwrap();
    let name: adw::EntryRow = builder.object("entry_name").unwrap();
    let mac: adw::EntryRow = builder.object("entry_mac").unwrap();
    let btn_dialog_add: gtk::Button = builder.object("btn_dialog_add").unwrap();

    let validate_form = glib::clone!(
        #[weak]
        name,
        #[weak]
        mac,
        #[weak]
        btn_dialog_add,
        move || {
            let is_name_ready = !name.text().trim().is_empty();
            let is_mac_ready = mac.text().len() == 17;
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

    btn_dialog_add.connect_clicked(glib::clone!(
        #[weak]
        name,
        #[weak]
        mac,
        #[weak]
        dialog,
        #[strong]
        state,
        move |_| {
            let name_text = name.text().trim().to_string();
            let mac_text = mac.text().replace(":", "").trim().to_lowercase();
            name.set_sensitive(false);
            mac.set_sensitive(false);

            let (sender, receiver) = async_channel::bounded::<Result<String, String>>(1);
            let name_text_clone = name_text.clone();
            let mac_text_clone = mac_text.clone();

            glib::MainContext::default().spawn_local(glib::clone!(
                #[weak]
                name,
                #[weak]
                mac,
                #[weak]
                dialog,
                async move {
                    if let Ok(firebase_result) = receiver.recv().await {
                        match firebase_result {
                            Ok(uid) => {
                                match crate::settings::save_device(
                                    &uid,
                                    &name_text_clone,
                                    &mac_text_clone,
                                ) {
                                    Ok(()) => {
                                        let msg = gettext("Classroom added successfully");
                                        dialog.add_toast(
                                            adw::Toast::builder().title(&msg).timeout(2).build(),
                                        );
                                        name.set_text("");
                                        mac.set_text("");
                                    }
                                    Err(err) => {
                                        let msg = format!(
                                            "{}: {}",
                                            gettext("Error saving local configuration"),
                                            err
                                        );
                                        dialog.add_toast(
                                            adw::Toast::builder().title(&msg).timeout(3).build(),
                                        );
                                    }
                                }
                            }
                            Err(err) => {
                                let msg = format!("{}: {}", gettext("Firebase Auth Error"), err);
                                dialog.add_toast(
                                    adw::Toast::builder().title(&msg).timeout(3).build(),
                                );
                            }
                        }
                    }
                    name.set_sensitive(true);
                    mac.set_sensitive(true);
                }
            ));

            let api_key = state.borrow().settings.api_key.clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                let result = rt.block_on(crate::firebase::register_camera(
                    mac_text.as_str(),
                    name_text.as_str(),
                    &api_key,
                ));
                let _ = rt.block_on(sender.send(result));
            });
        }
    ));

    dialog.present(Some(parent));
}
