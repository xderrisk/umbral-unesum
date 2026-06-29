use crate::settings;
use crate::state::SharedState;
use adw::prelude::*;
use async_channel;
use gettextrs::gettext;
use gtk::glib;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub fn classrooms_section(state: &SharedState) -> (gtk::Box, Rc<dyn Fn()>) {
    let api_key = state.borrow().settings.api_key.clone();
    let builder = gtk::Builder::from_resource("/edu/unesum/umbral/ui/classrooms_section.ui");
    let container: gtk::Box = builder.object("classrooms_panel").unwrap();
    let classrooms_flowbox: gtk::FlowBox = builder.object("classrooms_flowbox").unwrap();
    let classrooms_stack: gtk::Stack = builder.object("classrooms_stack").unwrap();
    let empty_label: gtk::Label = builder.object("empty_label").unwrap();

    let labels_map: Rc<RefCell<HashMap<String, (gtk::Label, gtk::Grid)>>> =
        Rc::new(RefCell::new(HashMap::new()));
    // ponytail: keeps last live status so a rebuild doesn't flash every card to Offline
    let status_cache: Rc<RefCell<HashMap<String, String>>> = Rc::new(RefCell::new(HashMap::new()));
    let (sender, receiver) = async_channel::unbounded::<crate::mqtt::ClassroomUpdate>();

    let populate = Rc::new({
        let container = container.clone();
        let classrooms_flowbox = classrooms_flowbox.clone();
        let classrooms_stack = classrooms_stack.clone();
        let empty_label = empty_label.clone();
        let labels_map = labels_map.clone();
        let status_cache = status_cache.clone();
        let state = state.clone();
        let api_key = api_key.clone();
        move || {
            while let Some(child) = classrooms_flowbox.first_child() {
                classrooms_flowbox.remove(&child);
            }
            labels_map.borrow_mut().clear();

            let classrooms = settings::load_classrooms();
            let total_classrooms = classrooms.len();
            if total_classrooms == 0 {
                classrooms_stack.set_visible_child(&empty_label);
                return;
            }
            classrooms_stack.set_visible_child(&classrooms_flowbox);
            let news_on = state.borrow().settings.news;
            // ponytail: 2 rows when news hidden → more columns, halign:fill+aspect-ratio scales cards up
            let max_columns = if news_on {
                (total_classrooms + 2) / 3
            } else {
                (total_classrooms + 1) / 2
            };
            classrooms_flowbox.set_max_children_per_line(max_columns as u32);
            if news_on {
                container.remove_css_class("news-hidden");
            } else {
                container.add_css_class("news-hidden");
            }

            for classroom in classrooms {
                let name = classroom["name"].as_str().unwrap_or("");
                let mac = classroom["mac"].as_str().unwrap_or("").to_string();
                let status = status_cache
                    .borrow()
                    .get(&mac)
                    .cloned()
                    .or_else(|| classroom["status"].as_str().map(|s| s.to_string()))
                    .unwrap_or_default();

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

                let initial_status = match status.as_str() {
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

                // delete + edit only touch firebase and devices.json; the file monitor rebuilds the UI
                let mac_clone = mac.clone();
                let api_key_clone = api_key.clone();
                btn_delete.connect_clicked(move |btn| {
                    btn.set_sensitive(false);
                    let mac_thread = mac_clone.clone();
                    let api_key_thread = api_key_clone.clone();
                    let (tx, rx) = async_channel::bounded::<Result<(), String>>(1);
                    let btn_weak = btn.downgrade();
                    let mac_ui = mac_thread.clone();

                    glib::MainContext::default().spawn_local(async move {
                        if let Ok(res) = rx.recv().await {
                            match res {
                                Ok(()) => {
                                    if let Err(e) = settings::delete_device(&mac_ui) {
                                        eprintln!("Error local: {}", e);
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
                        let mac_ui = mac_resp.clone();
                        let new_name_ui = new_name.clone();

                        glib::MainContext::default().spawn_local(async move {
                            if let Ok(res) = rx.recv().await {
                                match res {
                                    Ok(()) => {
                                        if let Err(e) =
                                            settings::rename_device(&mac_ui, &new_name_ui)
                                        {
                                            eprintln!("Error local: {}", e);
                                        }
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
                                    crate::firebase::login_camera(&mac_thread, &api_key_thread)
                                        .await?;
                                crate::firebase::update_camera_name(
                                    &uid,
                                    &id_token,
                                    &new_name_thread,
                                )
                                .await?;
                                Ok(())
                            };

                            let result = rt.block_on(operation);
                            let _ = rt.block_on(tx.send(result));
                        });
                    });

                    dialog.present(Some(btn));
                });

                labels_map
                    .borrow_mut()
                    .insert(mac.clone(), (status_label.clone(), card_grid.clone()));
                classrooms_flowbox.insert(&card_grid, -1);
            }
        }
    });

    populate();

    // Rebuild whenever devices.json changes (add/edit/delete, even from elsewhere).
    let devices_file = gtk::gio::File::for_path(settings::get_devices_path());
    if let Ok(monitor) =
        devices_file.monitor_file(gtk::gio::FileMonitorFlags::NONE, gtk::gio::Cancellable::NONE)
    {
        let populate_m = populate.clone();
        monitor.connect_changed(move |_, _, _, event| {
            if matches!(
                event,
                gtk::gio::FileMonitorEvent::ChangesDoneHint
                    | gtk::gio::FileMonitorEvent::Created
                    | gtk::gio::FileMonitorEvent::Deleted
            ) {
                populate_m();
            }
        });
        // ponytail: section is built once for the app's lifetime, so keep the monitor alive forever
        std::mem::forget(monitor);
    }

    let main_context = glib::MainContext::default();
    main_context.spawn_local(async move {
        while let Ok(update) = receiver.recv().await {
            status_cache
                .borrow_mut()
                .insert(update.mac.clone(), update.status.clone());
            if let Some((label, card)) = labels_map.borrow().get(&update.mac) {
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

    let relayout: Rc<dyn Fn()> = populate.clone();
    (container, relayout)
}
