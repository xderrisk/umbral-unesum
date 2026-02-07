use adw::prelude::*;
use adw::{Application, ApplicationWindow, HeaderBar, ToolbarView};
use gtk::{Align, Box, Label, Orientation, Button};

fn main() {
    let app = Application::builder()
        .application_id("ec.edu.unesum.umbral")
        .build();
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
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
    let left_container = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .hexpand(true)
        .valign(Align::Start)
        .build();

    let tittle_left = Label::builder()
        .label("Noticias")
        .css_classes(vec!["heading".to_string()])
        .build();

    left_container.append(&tittle_left);

    //Estado de Aulas
    let right_container = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .hexpand(true)
        .valign(Align::Start)
        .build();

    let tittle_right = Label::builder()
        .label("Estado de Aulas")
        .css_classes(vec!["heading".to_string()])
        .build();
    right_container.append(&tittle_right);

    main_box.append(&left_container);
    
    // Separador visual
    let separator = gtk::Separator::new(Orientation::Vertical);
    main_box.append(&separator);
    
    main_box.append(&right_container);

    toolbar_view.set_content(Some(&main_box));
    window.set_content(Some(&toolbar_view));

    window.present();
}