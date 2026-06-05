mod classrooms;
mod dialogs;
mod mqtt;
mod news;
mod settings;
use adw::prelude::*;
use adw::{Application, ApplicationWindow};
use gettextrs::{LocaleCategory, bindtextdomain, setlocale, textdomain};
use gtk::{Box, Builder, Button, gdk, gio, glib};

fn main() {
    gio::resources_register_include!("resources.gresource").expect("Resources could not be loaded");
    setlocale(LocaleCategory::LcAll, "");
    bindtextdomain("umbral", "locale").expect("The translation domain could not be linked");
    textdomain("umbral").expect("The text domain could not be established");
    let app = Application::builder()
        .application_id("edu.unesum.umbral")
        .build();
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let builder = Builder::from_resource("/edu/unesum/umbral/ui/main_window.ui");
    let provider = gtk::CssProvider::new();
    provider.load_from_data(include_str!("assets/style.css"));
    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    let window: ApplicationWindow = builder.object("window").expect("No window found");
    window.set_application(Some(app));

    builder
        .object::<Box>("left_container")
        .unwrap()
        .append(&news::news_section());
    builder
        .object::<Box>("right_container")
        .unwrap()
        .append(&classrooms::classrooms_section());

    let connect_dialog = |btn_id: &str, show_fn: fn(&ApplicationWindow)| {
        let btn: Button = builder.object(btn_id).unwrap();
        btn.connect_clicked(glib::clone!(
            #[weak]
            window,
            move |_| show_fn(&window)
        ));
    };

    connect_dialog("btn_add", dialogs::show_add);
    connect_dialog("btn_config", dialogs::show_config);
    connect_dialog("btn_about", dialogs::show_about);

    window.present();
}
