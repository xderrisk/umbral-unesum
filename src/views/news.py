import gi
gi.require_version("Gtk", "4.0")
from gi.repository import Gtk

class NewsBox(Gtk.Box):
    def __init__(self, parent=None):
        super().__init__(orientation=Gtk.Orientation.VERTICAL)
        self.set_hexpand(True)
        self.set_margin_start(20)
        
        title_label = Gtk.Label(label="Últimas Noticias Universitarias")
        title_label.add_css_class("title-3")
        title_label.set_halign(Gtk.Align.START)
        self.append(title_label)
        
        content_label = Gtk.Label(
            label="Aquí van tarjetas de información sobre la universidad."
        )
        content_label.set_wrap(True)
        content_label.set_halign(Gtk.Align.START)
        
        self.append(content_label)
