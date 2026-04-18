# Guía de Contribución

Bienvenido a la guía de contribución. Este documento, diseñado con apoyo de inteligencia artificial, contiene los lineamientos necesarios para colaborar en el código de manera eficiente y estandarizada.

## 📝 Convenciones de Commits

En este proyecto utilizamos [Conventional Commits](https://www.conventionalcommits.org/). Esto nos ayuda a mantener un historial limpio y facilita la generación automática de changelogs.

### Formato del Mensaje
`tipo(alcance opcional): descripción corta en minúsculas`

### Tipos de Commit
| Tipo | Descripción |
| :--- | :--- |
| **feat** | Una nueva funcionalidad. |
| **fix** | Corrección de un error. |
| **docs** | Solo cambios en la documentación. |
| **style** | Cambios de formato que no afectan el código. |
| **refactor** | Mejora de código que no corrige errores ni añade funciones. |
| **perf** | Cambios para mejorar el rendimiento. |
| **test** | Adición o corrección de pruebas. |
| **build** | Cambios que afectan el sistema de compilación o dependencias. |
| **chore** | Tareas de mantenimiento que no alteran el código fuente. |

### Ejemplos
- `feat(ui): agregar soporte para modo oscuro`
- `fix: corregir desbordamiento en el sensor de temperatura`
- `docs: actualizar guía de instalación en el README`
- `chore: actualizar dependencias en el archivo de configuración`

> **Nota:** Si el cambio rompe la compatibilidad (Breaking Change), añade un `!` después del tipo, por ejemplo: `feat!: cambiar estructura de la base de datos`.
