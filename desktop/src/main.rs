mod classrooms;
mod config;
mod news;

use adw::prelude::*;
use adw::{Application, ApplicationWindow, HeaderBar, ToolbarView};
use gtk::{Box, Button, Orientation, gdk};

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

    btn_add.connect_clicked(|_| {
        println!("boton presionado");
    });

    let toolbar_view = ToolbarView::new();
    let header_bar = HeaderBar::new();
    header_bar.pack_end(&btn_add);
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
