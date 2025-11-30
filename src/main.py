#!/usr/bin/env python3
import gi
import os

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
from gi.repository import Gtk, Adw, Gdk

class Umbral(Adw.Application):

    def __init__(self):
        super().__init__(application_id="com.xderrisk.umbral-unesum")

    def do_activate(self):

        window = Adw.ApplicationWindow(application=self)
        window.set_default_size(800, 400)
        window.set_size_request(400, 200)
        window.set_title("Umbral")

        toolbar = Adw.ToolbarView()
        header = Adw.HeaderBar()
        
        scan_button = Gtk.Button.new_from_icon_name("list-add-symbolic")
        header.pack_start(scan_button)

        option_button = Gtk.Button.new_from_icon_name("open-menu-symbolic")
        header.pack_start(option_button)

        toolbar.add_top_bar(header)
        toolbar.set_content(self.dashboard())
        window.set_content(toolbar)
        window.present()

    def dashboard(self):
        box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        box.set_margin_top(20)
        box.set_margin_bottom(20)
        box.set_margin_start(20)
        box.set_margin_end(20)

        box_news = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
        box_news.set_hexpand(True)

        box_clasroom = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=20)
        box_clasroom.set_hexpand(True)
        box_clasroom.set_halign(Gtk.Align.END)
        box_clasroom.set_valign(Gtk.Align.CENTER)

        base_dir = os.path.dirname(os.path.abspath(__file__))
        css_path = os.path.join(base_dir, os.pardir, "assets", "style.css")
        css = Gtk.CssProvider()
        css.load_from_path(css_path)


        Gtk.StyleContext.add_provider_for_display(
            Gdk.Display.get_default(), css, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
        )

        for i in range(2):
            cuadro = Gtk.Box()
            cuadro.add_css_class("cuadro")
            box_clasroom.append(cuadro)

        box_news.append(Gtk.Label(label="Aqui van las noticias, posiblemente se presenten imagenes"))


        box.append(box_news)
        box.append(box_clasroom)

        return box
        
if __name__ == "__main__":
    app = Umbral()
    
    exit_status = app.run(None)
    import sys
    sys.exit(exit_status)