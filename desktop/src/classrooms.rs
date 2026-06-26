use crate::settings;
use crate::state::SharedState;
use adw::prelude::*;
use async_channel;
use gettextrs::gettext;
use gtk::glib;
use std::collections::HashMap;

pub fn classrooms_section(state: &SharedState) -> gtk::Box {
    let api_key = state.borrow().settings.api_key.clone();
    let builder = gtk::Builder::from_resource("/edu/unesum/umbral/ui/classrooms_section.ui");
    let container: gtk::Box = builder.object("classrooms_panel").unwrap();
    let classrooms_flowbox: gtk::FlowBox = builder.object("classrooms_flowbox").unwrap();
    let classrooms_stack: gtk::Stack = builder.object("classrooms_stack").unwrap();
    let empty_label: gtk::Label = builder.object("empty_label").unwrap();

    let mut labels_map: HashMap<String, (gtk::Label, gtk::Grid)> = HashMap::new();
    let (sender, receiver) = async_channel::unbounded::<crate::mqtt::ClassroomUpdate>();

    let classrooms = settings::load_classrooms();
    let total_classrooms = classrooms.len();
    if total_classrooms > 0 {
        classrooms_stack.set_visible_child(&classrooms_flowbox);
        let max_columns = (total_classrooms + 2) / 3;
        classrooms_flowbox.set_max_children_per_line(max_columns as u32);

        for classroom in classrooms {
            let name = classroom["name"].as_str().unwrap_or("");
            let mac = classroom["mac"].as_str().unwrap_or("").to_string();
            let status = classroom["status"].as_str().unwrap_or("");

            let card_builder =
                gtk::Builder::from_resource("/edu/unesum/umbral/ui/classroom_card.ui");

            let card_grid: gtk::Grid = card_builder
                .object("classroom_card_root")
                .expect("Error: 'classroom_card_root' not found in the UI file.");

            let name_label: gtk::Label = card_builder.object("name_label").unwrap();
            let status_label: gtk::Label = card_builder.object("status_label").unwrap();
            let btn_delete: gtk::Button = card_builder.object("btn_delete").unwrap();
            let btn_edit: gtk::Button = card_builder.object("btn_edit").unwrap();

            name_label.set_label(&name);

            let initial_status = match status {
                "0" => {
                    card_grid.add_css_class("classroom-available");
                    gettext("Available")
                }
                "1" => {
                    card_grid.add_css_class("classroom-occupied");
                    gettext("Occupied")
                }
                _ => gettext("Offline"),
            };
            status_label.set_label(&initial_status);

            let mac_clone = mac.clone();
            let api_key_clone = api_key.clone();
            let card_grid_clone = card_grid.clone();
            let classrooms_flowbox_weak = classrooms_flowbox.downgrade();

            btn_delete.connect_clicked(move |btn| {
                btn.set_sensitive(false);
                let mac_thread = mac_clone.clone();
                let api_key_thread = api_key_clone.clone();
                let (tx, rx) = async_channel::bounded::<Result<(), String>>(1);
                let btn_weak = btn.downgrade();
                let card_grid_ui = card_grid_clone.clone();
                let flowbox_weak = classrooms_flowbox_weak.clone();
                let mac_ui = mac_thread.clone();

                glib::MainContext::default().spawn_local(async move {
                    if let Ok(res) = rx.recv().await {
                        match res {
                            Ok(()) => {
                                if let Err(e) = settings::delete_device(&mac_ui) {
                                    eprintln!("Error local: {}", e);
                                }
                                if let Some(flowbox) = flowbox_weak.upgrade() {
                                    flowbox.remove(&card_grid_ui);
                                }
                            }
                            Err(err_msg) => {
                                eprintln!("Error when deleting: {}", err_msg);
                                if let Some(b) = btn_weak.upgrade() {
                                    b.set_sensitive(true);
                                }
                            }
                        }
                    }
                });

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .unwrap();

                    let operation = async {
                        let (uid, id_token) =
                            crate::firebase::login_camera(&mac_thread, &api_key_thread).await?;
                        crate::firebase::delete_camera(&uid, &id_token, &api_key_thread).await?;
                        Ok(())
                    };

                    let result = rt.block_on(operation);
                    let _ = rt.block_on(tx.send(result));
                });
            });

            let name_label_edit = name_label.clone();
            let mac_edit = mac.clone();
            let api_key_edit = api_key.clone();

            btn_edit.connect_clicked(move |btn| {
                let entry = gtk::Entry::builder()
                    .text(name_label_edit.label())
                    .activates_default(true)
                    .build();
                let dialog = adw::AlertDialog::builder()
                    .heading(gettext("Edit classroom name"))
                    .extra_child(&entry)
                    .build();
                dialog.add_response("cancel", &gettext("Cancel"));
                dialog.add_response("save", &gettext("Save"));
                dialog.set_response_appearance("save", adw::ResponseAppearance::Suggested);
                dialog.set_default_response(Some("save"));
                dialog.set_close_response("cancel");

                let name_label_resp = name_label_edit.clone();
                let mac_resp = mac_edit.clone();
                let api_key_resp = api_key_edit.clone();
                dialog.connect_response(None, move |_, response| {
                    if response != "save" {
                        return;
                    }
                    let new_name = entry.text().trim().to_string();
                    if new_name.is_empty() {
                        return;
                    }

                    let (tx, rx) = async_channel::bounded::<Result<(), String>>(1);
                    let name_label_ui = name_label_resp.clone();
                    let mac_ui = mac_resp.clone();
                    let new_name_ui = new_name.clone();

                    glib::MainContext::default().spawn_local(async move {
                        if let Ok(res) = rx.recv().await {
                            match res {
                                Ok(()) => {
                                    if let Err(e) = settings::rename_device(&mac_ui, &new_name_ui) {
                                        eprintln!("Error local: {}", e);
                                    }
                                    name_label_ui.set_label(&new_name_ui);
                                }
                                Err(err_msg) => eprintln!("Error when editing: {}", err_msg),
                            }
                        }
                    });

                    let mac_thread = mac_resp.clone();
                    let api_key_thread = api_key_resp.clone();
                    let new_name_thread = new_name.clone();
                    std::thread::spawn(move || {
                        let rt = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .unwrap();

                        let operation = async {
                            let (uid, id_token) =
                                crate::firebase::login_camera(&mac_thread, &api_key_thread).await?;
                            crate::firebase::update_camera_name(&uid, &id_token, &new_name_thread)
                                .await?;
                            Ok(())
                        };

                        let result = rt.block_on(operation);
                        let _ = rt.block_on(tx.send(result));
                    });
                });

                dialog.present(Some(btn));
            });

            labels_map.insert(mac.clone(), (status_label.clone(), card_grid.clone()));
            classrooms_flowbox.insert(&card_grid, -1);
        }
    } else {
        classrooms_stack.set_visible_child(&empty_label);
    }

    let main_context = glib::MainContext::default();
    main_context.spawn_local(async move {
        while let Ok(update) = receiver.recv().await {
            if let Some((label, card)) = labels_map.get(&update.mac) {
                let status_text = match update.status.as_str() {
                    "0" => {
                        card.add_css_class("classroom-available");
                        card.remove_css_class("classroom-occupied");
                        gettext("Available")
                    }
                    "1" => {
                        card.add_css_class("classroom-occupied");
                        card.remove_css_class("classroom-available");
                        gettext("Occupied")
                    }
                    _ => {
                        card.remove_css_class("classroom-available");
                        card.remove_css_class("classroom-occupied");
                        gettext("Offline")
                    }
                };
                label.set_label(&status_text);
            }
        }
    });

    crate::mqtt::init(sender);
    container
}
