import gi
gi.require_version("Gtk", "4.0")
from gi.repository import Gtk

class ClassroomsBox(Gtk.Box):
    def __init__(self, parent=None):
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=20)
        self.set_hexpand(True)
        self.set_halign(Gtk.Align.END)
        self.set_valign(Gtk.Align.CENTER)

        self._load_class_cards()

    def _load_class_cards(self):
        
        for i in range(2):
            card = Gtk.Box()
            card.add_css_class("cuadro")
            label = Gtk.Label(label=f"Aula {i+100}")
            label.add_css_class("title-1")
            label.set_hexpand(True)
            card.append(label)
            
            self.append(card)