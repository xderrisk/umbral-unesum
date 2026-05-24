mod classrooms;
mod mqtt;
mod news;
mod settings;
use adw::prelude::*;
use adw::{
    AboutDialog, Application, ApplicationWindow, EntryRow, HeaderBar, PreferencesDialog,
    PreferencesGroup, PreferencesPage, Toast, ToolbarView,
};
use gtk::{Box, Button, Orientation, gdk, glib};

fn main() {
    let app = Application::builder()
        .application_id("edu.unesum.umbral")
        .build();
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let display = gdk::Display::default().expect("No se pudo conectar al display");
    let provider = gtk::CssProvider::new();
    let priority = gtk::STYLE_PROVIDER_PRIORITY_APPLICATION;
    provider.load_from_data(include_str!("../assets/style.css"));
    gtk::style_context_add_provider_for_display(&display, &provider, priority);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Umbral - UNESUM")
        .build();

    let btn_add = Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text("Agregar Aula")
        .build();

    let btn_config = Button::builder()
        .icon_name("settings-symbolic")
        .tooltip_text("Configuracón")
        .build();

    let btn_about = Button::builder()
        .icon_name("help-about-symbolic")
        .tooltip_text("Sobre nosotros")
        .build();

    btn_add.connect_clicked(glib::clone!(
        #[weak]
        window,
        move |_| {
            show_add_dialog(&window);
        }
    ));

    btn_config.connect_clicked(glib::clone!(
        #[weak]
        window,
        move |_| {
            show_config_dialog(&window);
        }
    ));

    btn_about.connect_clicked(glib::clone!(
        #[weak]
        window,
        move |_| {
            show_about_dialog(&window);
        }
    ));

    let toolbar_view = ToolbarView::new();
    let header_bar = HeaderBar::new();
    header_bar.pack_start(&btn_add);
    header_bar.pack_end(&btn_about);
    header_bar.pack_end(&btn_config);
    toolbar_view.add_top_bar(&header_bar);

    let main_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(0)
        .build();

    // Noticias UNESUM
    let left_container = news::news_section();
    // Estado de Aulas
    let right_container = classrooms::classroms_section();
    right_container.set_hexpand(false);

    main_box.append(&left_container);
    let separator = gtk::Separator::new(Orientation::Vertical);
    main_box.append(&separator);
    main_box.append(&right_container);

    toolbar_view.set_content(Some(&main_box));
    window.set_content(Some(&toolbar_view));
    window.present();
}

fn show_about_dialog(parent: &ApplicationWindow) {
    let about = AboutDialog::builder()
        .application_name("Umbral")
        .application_icon("edu.unesum.umbral")
        .developer_name("Sergio Galarza")
        .developers(vec![
            "Sergio Galarza - Desarrollador principal",
            "Oscar Plua - Patrocinador",
            "Mafer Lucas - Patrocinadora",
        ])
        .build();
    about.present(Some(parent));
}
fn show_add_dialog(parent: &ApplicationWindow) {
    let dialog = PreferencesDialog::builder()
        .title("Añadir Dispositivo")
        .build();
    let page = PreferencesPage::builder().build();
    let add = Button::builder()
        .label("Agregar")
        .css_classes(["suggested-action"])
        .build();
    let group = PreferencesGroup::builder()
        .title("Datos del ESP32CAM")
        .header_suffix(&add)
        .build();
    let name = EntryRow::builder().title("Nombre").build();
    let mac = EntryRow::builder().title("MAC").build();
    group.add(&name);
    group.add(&mac);
    page.add(&group);
    dialog.add(&page);
    add.connect_clicked(glib::clone!(
        #[weak]
        name,
        #[weak]
        mac,
        #[weak]
        dialog,
        move |_| {
            let name = name.text().to_string();
            let mac = mac.text().to_string();
            if name.is_empty() || mac.is_empty() {
                let toast = Toast::new("Por favor rellena todos los campos");
                toast.set_timeout(1);
                dialog.add_toast(toast);
                return;
            }
            let toast = Toast::new("Dispositivo agregado exitosamente");
            toast.set_timeout(1);
            dialog.add_toast(toast);
        }
    ));
    dialog.present(Some(parent));
}
fn show_config_dialog(parent: &ApplicationWindow) {
    let dialog = PreferencesDialog::builder().title("Configuración").build();
    dialog.present(Some(parent));
}
