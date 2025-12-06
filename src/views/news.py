import gi
import os
gi.require_version("Gtk", "4.0")
from gi.repository import Gtk, GLib

class NewsBox(Gtk.Box):
    def __init__(self, parent=None):
        super().__init__(orientation=Gtk.Orientation.VERTICAL)
        self.set_hexpand(True)
        self.set_margin_end(20)

        self.images = []
        self.current_index = 0
        
        title_label = Gtk.Label(label="Últimas Noticias UNESUM-TI")
        title_label.add_css_class("title-3")
        title_label.set_halign(Gtk.Align.START)
        self.append(title_label)

        controls = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        self.image = Gtk.Picture()
        self.image.add_css_class("rounded-image")
        self.image.set_hexpand(True)
        self.image.set_vexpand(True)
        self.image.set_halign(Gtk.Align.FILL)
        self.image.set_valign(Gtk.Align.FILL)
        controls.append(self.image)
        
        self.append(controls)
        self.load_images()
        if self.images:
            self.show_image(0)
            self._timeout_id = GLib.timeout_add_seconds(10, self._on_timeout)

    def load_images(self):
        src_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        assets_news = os.path.normpath(os.path.join(src_dir, os.pardir, "assets", "news"))

        if not os.path.isdir(assets_news):
            return

        for fn in sorted(os.listdir(assets_news)):
            if fn.lower().endswith((".png", ".jpg", ".jpeg", ".gif", ".svg")):
                self.images.append(os.path.join(assets_news, fn))

    def show_image(self, index):
        if not self.images:
            return
        index = index % len(self.images)
        path = self.images[index]
        try:            
            self.image.set_filename(path)
        except Exception:
            self.append(Gtk.Label(label="No hay imagenes disponibles"))
        self.current_index = index

    def next_image(self):
        if not self.images:
            return
        self.show_image((self.current_index + 1) % len(self.images))

    def _on_timeout(self):
        self.next_image()
        return True
