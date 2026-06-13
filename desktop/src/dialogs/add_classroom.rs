use adw::prelude::*;
use gettextrs::gettext;
use gtk::glib;

pub fn show_add(parent: &adw::ApplicationWindow) {
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

            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();

                let result = rt.block_on(register_camera_in_firebase(
                    mac_text.as_str(),
                    name_text.as_str(),
                ));
                let _ = rt.block_on(sender.send(result));
            });
        }
    ));

    dialog.present(Some(parent));
}

async fn register_camera_in_firebase(mac: &str, classroom_name: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let api_key = crate::settings::load_api_key().ok_or_else(|| {
        gettext("Firebase API Key is not configured in the application settings.")
    })?;

    let url_auth = format!(
        "https://identitytoolkit.googleapis.com/v1/accounts:signUp?key={}",
        api_key
    );

    let mock_email = format!("camera_{}@umbral.unesum.edu", mac);
    let mock_password = format!("Umbral.{}#", mac);

    let body_auth = serde_json::json!({
        "email": mock_email,
        "password": mock_password,
        "returnSecureToken": true
    });

    let response_auth = client
        .post(&url_auth)
        .json(&body_auth)
        .send()
        .await
        .map_err(|e| {
            format!(
                "{}: {}",
                gettext("Connection error during authentication"),
                e
            )
        })?;

    let (uid, id_token) = if response_auth.status().is_success() {
        let json_res: serde_json::Value = response_auth.json().await.map_err(|e| e.to_string())?;
        let local_id = json_res
            .get("localId")
            .and_then(|id| id.as_str())
            .unwrap_or("")
            .to_string();
        let token = json_res
            .get("idToken")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();
        (local_id, token)
    } else {
        let error_text = response_auth.text().await.unwrap_or_default();
        if error_text.contains("EMAIL_EXISTS") {
            return Err(gettext(
                "The device is already authenticated. Delete the user in the Firebase console to retry.",
            ));
        }
        return Err(format!(
            "{}: {}",
            gettext("Authentication error"),
            error_text
        ));
    };

    if uid.is_empty() || id_token.is_empty() {
        return Err(gettext("Failed to obtain credentials from Firebase Auth"));
    }

    let url_db = format!(
        "https://myapplication-65c31ca7-default-rtdb.firebaseio.com/cameras/{}.json?auth={}",
        uid, id_token
    );

    let body_db = serde_json::json!({
        "name": classroom_name,
    });

    let response_db = client
        .put(&url_db)
        .json(&body_db)
        .send()
        .await
        .map_err(|e| {
            format!(
                "{}: {}",
                gettext("Connection error during database write"),
                e
            )
        })?;

    if response_db.status().is_success() {
        Ok(uid)
    } else {
        let err_db = response_db.text().await.unwrap_or_default();
        Err(format!(
            "{}: {}",
            gettext("Authentication successful, but database rules rejected the write operation"),
            err_db
        ))
    }
}
