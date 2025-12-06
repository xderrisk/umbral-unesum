## Umbral: monitorizador de aulas
Umbral es un proyecto hecho para la Universidad Estatal del Sur de Manabí que permite monitorear las aulas de la carrera Tecnologias de la Información a travez de sensores.

### Tecnologias usadas
* Python (logica)
* GTK4 y Adwaita (interfaz grafica)
* Flatpak (empaquetado)

### Comandos utiles
Instalar Gnome SDK
```
flatpak install flathub org.gnome.Sdk 49
```
Instalar como flatpak
```
flatpak-builder --user --install --force-clean build com.xderrisk.umbral-unesum.yaml
```
Ejecutar
```
flatpak run com.xderrisk.umbral-unesum
```