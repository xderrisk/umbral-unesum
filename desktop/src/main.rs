mod classrooms;
mod dialogs;
mod firebase;
mod mqtt;
mod news;
mod settings;
mod state;
use adw::prelude::*;
use gettextrs::{bindtextdomain, setlocale, textdomain};
use gtk::{gdk, gio, glib};

fn main() {
    gio::resources_register_include!("resources.gresource")
        .expect("Failed to load embedded resources");

    setlocale(gettextrs::LocaleCategory::LcAll, "");
    bindtextdomain("umbral", "locale").expect("Failed to bind the translation domain");
    textdomain("umbral").expect("Failed to set the text domain");

    let app = adw::Application::builder()
        .application_id("edu.unesum.umbral")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &adw::Application) {
    let state = state::AppState::new();
    let builder = gtk::Builder::from_resource("/edu/unesum/umbral/ui/main_window.ui");

    let provider = gtk::CssProvider::new();
    provider.load_from_data(include_str!("assets/style.css"));
    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    let window: adw::ApplicationWindow = builder
        .object("window")
        .expect("Main window object not found in UI file");
    window.set_application(Some(app));

    let separator: gtk::Separator = builder.object("main_separator").unwrap();
    if state.borrow().settings.news {
        builder
            .object::<gtk::Box>("left_container")
            .unwrap()
            .append(&news::news_section());
    } else {
        separator.set_visible(false);
    }

    builder
        .object::<gtk::Box>("right_container")
        .unwrap()
        .append(&classrooms::classrooms_section());

    let connect_dialog = |btn_id: &str, show_fn: fn(&adw::ApplicationWindow)| {
        let btn: gtk::Button = builder.object(btn_id).unwrap();
        btn.connect_clicked(glib::clone!(
            #[weak]
            window,
            move |_| show_fn(&window)
        ));
    };

    let setting_dialog =
        |btn_id: &str, show_fn: fn(&adw::ApplicationWindow, &state::SharedState)| {
            let btn: gtk::Button = builder.object(btn_id).unwrap();
            btn.connect_clicked(glib::clone!(
                #[weak]
                window,
                #[strong]
                state,
                move |_| show_fn(&window, &state)
            ));
        };

    setting_dialog("btn_add", dialogs::show_add);
    setting_dialog("btn_config", dialogs::show_config);
    connect_dialog("btn_about", dialogs::show_about);

    window.present();
}
