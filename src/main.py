import gi

gi.require_version("Gtk", "4.0")
from gi.repository import Gtk

class HolaMundoApp(Gtk.Application):

    def __init__(self):
        super().__init__(application_id="com.xderrisk.umbral-unesum")

    def do_activate(self):

        window = Gtk.ApplicationWindow(application=self, title="Dashboard - Umbral")
        window.set_default_size(400, 200)
        window.set_size_request(400, 200)

        header_bar = Gtk.HeaderBar()
        window.set_titlebar(header_bar)
        info_button = Gtk.Button.new_from_icon_name("dialog-information-symbolic")
        header_bar.pack_end(info_button)
        header_bar.pack_end(info_button)

        
        label = Gtk.Label(label="¡Hola Mundo!")
        
        label.set_margin_top(20)
        label.set_margin_bottom(20)
        label.set_margin_start(20)
        label.set_margin_end(20)
        
        window.set_child(label)
        
        window.present()

if __name__ == "__main__":
    app = HolaMundoApp()
    
    exit_status = app.run(None)
    import sys
    sys.exit(exit_status)