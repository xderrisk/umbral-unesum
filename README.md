## Umbral: monitorizador de aulas
Umbral es un proyecto hecho para la Universidad Estatal del Sur de Manabí que permite monitorear las aulas de la carrera Tecnologias de la Información a travez de sensores.

### Tecnologias usadas
* Python (logica)
* GTK4 y Adwaita (interfaz grafica)
* Flatpak (empaquetado)

### Comandos utiles
Empaquetar y ejecutar con flatpak
```
flatpak-builder --user --install --force-clean build com.xderrisk.umbral-unesum.yaml
```
```
flatpak run com.xderrisk.umbral-unesum
```