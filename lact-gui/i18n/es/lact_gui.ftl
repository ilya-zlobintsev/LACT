hardware-info = Información del hardware
info-page = Información
oc-page = OC
thermals-page = Térmicas
software-page = Software
system-section = Sistema
lact-daemon = Versión del daemon LACT
lact-gui = Versión del GUI LACT
kernel-version = Versión de kernel
instance = Instancia
device-name = Nombre del dispositivo
platform-name = Nombre de la plataforma
api-version = Versión de API
version = Versión
driver-name = Nombre del driver
driver-version = Versión del driver
compute-units = Unidades de cómputo
cl-c-version = Versión de OpenCL C
workgroup-size = Tamaño de grupo de trabajo
global-memory = Memoria global
local-memory = Memoria local
features = Funcionalidades
extensions = Extensiones
show-button = Mostrar
device-not-found = { $kind } dispositivo no encontrado
cache-info = Información de la caché
amd-cache-desc =
    { $size } L{ $level } { $types } cache { $shared ->
        [1] local para cada U. de C.
       *[other] compartido entre { $shared } U. de C.
    }
nvidia-cache-desc = { $size } L{ $level }
cache-data = Data
cache-instruction = Data
cache-cpu = CPU
monitoring-section = Monitoreo
fan-control-section = Control de ventiladores
temperatures = Temperaturas
oc-missing-fan-control-warning = Advertencia: el soporte de overclock está deshabilitado, por lo que la función de control de ventiladores no está disponible.
fan-speed = Velocidad del ventilador
throttling = Estrangulamiento térmico
auto-page = Automático
curve-page = Curva
static-page = Estático
target-temp = Temperatura objetivo (°C)
acoustic-limit = Límite acústico (RPM)
acoustic-target = Acústicas objetivo (RPM)
min-fan-speed = Velocidad mínima (%)
zero-rpm = Cero RPM
zero-rpm-stop-temp = Temperatura para activar ventiladores (°C)
static-speed = Velocidad estática (%)
reset-button = Restablecer
pmfw-reset-warning = Advertencia: esto reiniciará los ajustes de firmware de los ventiladores!
temperature-sensor = Sensor de temperatura
spindown-delay = Delay de desaceleración (ms)
spindown-delay-tooltip = Cuánto tiempo debe mantenerse la GPU en valores bajos de temperatura antes de desacelearar los ventiladores
speed-change-threshold = Umbral de cambio de velocidad (°C)
automatic-mode-threshold = Umbral del modo automático (°C)
automatic-mode-threshold-tooltip =
    Cambia el control de ventiladores al modo automático cuando la temperatura está debajo de este punto.

    Bastantes GPUs de NVidia sólo permiten parar sus ventuladores en el modo de control automático, mientras que una curva personalizada sólo da un rango limitado, como por ejemplo 30-100%.

    Esta opción permite saltarse esa limitación usando sólo la curva personalizada cuando se está sobre una temperatura específica, mientras que se usa el modo que soporte los cero RPM cuando se está debajo de ella.
amd-oc = Overclocking de AMD
amd-oc-disabled =
    El soporte de overclocking de AMD no está activado!
    Puedes cambiar ajustes básicos, pero no estarán disponibles los ajustes más avanzados de reloj y voltaje.
amd-oc-status =
    El overclocking de AMD está: <b>{ $status ->
        [true] Activado
        [false] Desactivado
       *[other] Desconocido
    }</b>
amd-oc-detected-system-config =
    Configuración del sistema detectada: <b>{ $config ->
        [unsupported] No soportado
       *[other] { $config }
    }</b>
amd-oc-description =
    { $config ->
        [rpm-ostree] Esta opción activa soporte overdrive de AMD activando opciones al inicio del sistema desde <b>rpm-ostree</b>.
        [unsupported]
            Este sistema no soporta configuración automática de overdrive.
            Puedes intentar activar overclocking desde LACT, pero una regeneración manual de initramfs puede ser necesaria para que surta efecto.
            Si eso falla, otra opción es agregar <b>amdgpu.ppfeaturemask=0xffffffff</b> como un parámetro de inicio en tu bootloader.
       *[other] Esta opción va a activar overdrive de AMD creando un archivo en  <b>{ $path }</b> y actualizando initramfs.
    }

    Revisa <a href="https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)">la wiki</a> para más información.
enable-amd-oc-description = Esto va a activar la función de overdrive del driver amdgpu creando un archivo en <b>{ $path }</b> y actualizando initramfs. ¿Estás seguro de que quieres hacer esto?
disable-amd-oc = Desactivar el overclocking de AMD
enable-amd-oc = Activar el overclocking de AMD
disable-amd-oc-description = Esto va a desactivar el soporte de overclocking de AMD (overdrive) en el próximo reinicio.
amd-oc-updating-configuration = Actualizando la configuración (esto puede llevar un rato)
amd-oc-updating-done = Configuración actualizada, por favor reinicia para aplicar los cambios.
reset-config = Restablecer la Configuración
reset-config-description = ¿Estás seguro de que quieres restablecer toda la configuración de la GPU?
apply-button = Aplicar
revert-button = Deshacer
power-cap = Límite de uso de energía
watt = W
ghz = GHz
mhz = MHz
mebibyte = MiB
stats-section = Estadísticas
gpu-clock = Reloj del núcleo de GPU
gpu-clock-avg = Reloj de núcleo de GPU (Promedio)
gpu-clock-target = Reloj de núcleo de GPU (Objetivo)
gpu-voltage = Voltaje de GPU
gpu-temp = Temperatura
gpu-usage = Utilización de GPU
vram-clock = Reloj de VRAM
power-usage = Utilización de energía
no-throttling = No
unknown-throttling = Desconocido
missing-stat = N/D
vram-usage = Uso de VRAM:
performance-level-auto = Automático
performance-level-high = Relojes más altos
performance-level-low = Relojes más bajos
performance-level-manual = Manual
performance-level-auto-description = Ajustar automáticamente los relojes de la GPU y VRAM. (Por defecto)
performance-level-high-description = Siempre usar las velocidades más altas de reloj para la GPU y VRAM.
performance-level-low-description = Siempre usar las velocidades de reloj más bajas para la GPU y VRAM.
performance-level-manual-description = Control manual del rendimiento.
performance-level = Nivel de rendimiento
power-profile-mode = Modo de perfil de energía:
manual-level-needed = El nivel de rendimiento tiene que estar en "manual" para poder usar los modos y estados de energía
overclock-section = Velocidades de reloj y Voltaje
edit-rule = Editar Regla
move-up = Subir
move-down = Bajar
edit-graphs = Editar
name = Nombre
create = Crear
cancel = Cancelar
save = Guardar
default-profile = Predeterminado
nvidia-oc-info = Información Aceleración Nvidia
nvidia-oc-description =
    La funcionalidad de overclocking en Nvidia incluye la configuración de compensaciones para las velocidades de reloj de la GPU/VRAM y la limitación del rango potencial de velocidades de reloj mediante la función de "relojes bloqueados".

    En muchas tarjetas, la compensación de la velocidad de reloj de la VRAM solo afecta la velocidad de reloj real de la memoria a la mitad del valor de compensación.
    Por ejemplo, una compensación de VRAM de +1000 MHz puede aumentar la velocidad de VRAM medida solo en 500 MHz.
    Esto es normal y es la forma en que Nvidia gestiona las velocidades de datos GDDR. Ajuste su overclock según corresponda.

    El control directo de voltaje no es compatible, ya que no existe en el controlador de Nvidia para Linux.

    Es posible lograr un pseudo-subvoltaje combinando la opción de relojes bloqueados con una compensación positiva de la velocidad de reloj.
    Esto obligará a la GPU a funcionar a un voltaje limitado por los relojes bloqueados, mientras que alcanzará una velocidad de reloj más alta gracias a la compensación.
    Esto puede causar inestabilidad en el sistema si se aumenta demasiado.
oc-warning = Advertencia: cambiar estos valores puede provocar inestabilidad en el sistema y potencialmente dañar su hardware.
show-all-pstates = Muestra todos los Estados-P
enable-gpu-locked-clocks = Habilitar Relojes Bloqueados de GPU
enable-vram-locked-clocks = Habilita Relojes VRAM Bloqueados
pstate-list-description = <b>Los siguientes valores son desplazamientos de reloj para cada Estado-P, desde el más alto hasta el más bajo.</b>
no-clocks-data = No hay datos de relojes disponibles
reset-oc-tooltip = Advertencia: ¡esto restablece todas las configuraciones del reloj a los valores predeterminados!
edit-rules = Editar Reglas
remove-rule = Retirar Regla
profile-rules = Perfil de Reglas
export-to-file = Exportar a Archivo
profile-activation = Activación
profile-hooks = Ganchos
profile-activation-desc = Activar perfil ‘{ $name }’ cuando:
any-rules-matched = Cualquiera de las siguientes reglas están marcadas:
all-rules-matched = Todas las siguientes reglas están marcadas:
activation-settings-status =
    Seleccionó ajustes de activación actualmente son <b>{ $matched ->
        [true] coincide
       *[false] no coincide
    }</b>
activation-auto-switching-disabled = El conmutador automático de perfil está actualmente inhabilitado
profile-hook-command = Ejecuta un comando cuando el perfil ‘{ $cmd }’ está:
profile-hook-activated = Activado:
profile-hook-deactivated = Desactivado:
profile-rule-process-tab = Se está ejecutando un proceso
profile-rule-gamemode-tab = Modo juego está activado
profile-rule-process-name = Nombre del proceso:
profile-rule-args-contain = Argumentos contienen:
profile-rule-specific-process = Con un proceso específico:
dump-vbios = Volcado de VBIOS
reset-all-config = Restablecer Toda Configuración
stats-update-interval = Actualizar Intervalo (ms)
historical-data-title = Datos Históricos
graphs-per-row = Gráficos por fila:
time-period-seconds = Periodo Temporal (sg.):
add-graph = Agregar Gráfico
delete-graph = Eliminar Gráfico
export-csv = Exportar como CSV
edit-graph-sensors = Editar Sensores de Gráfico
reconnecting-to-daemon = Conexión de daemon perdida, reconectando…
daemon-connection-lost = Conexión Perdida
plot-show-detailed-info = Mostrar informe detallado
settings-profile = Perfil de Ajustes
auto-switch-profiles = Conmutar automáticamente
add-profile = Agregar perfil
import-profile = Importar perfil desde archivo
create-profile = Crear Perfil
profile-copy-from = Copiar ajustes desde:
rename-profile = Renombrar Perfil
rename-profile-from = Renombrar perfil <b>{ $old_name }</b> a:
delete-profile = Eliminar Perfil
profile-hook-note = Nota: Estos comandos se ejecutan como root por el demonio LACT y no tienen acceso al entorno de escritorio. Por lo tanto, no se pueden usar directamente para lanzar aplicaciones gráficas.
pstates = Estado de Energía
gpu-pstates = Estados de Energía GPU
vram-pstates = Estados de Energía VRAM
pstates-manual-needed = Nota: el nivel de rendimiento debe ajustarse en 'manual' para conmutar los estados de energía
enable-pstate-config = Habilitar configuración de estado de energía
show-historical-charts = Mostrar Cartas Históricas
show-process-monitor = Mostrar Monitor de Proceso
generate-debug-snapshot = Generar Capturas Depuradoras
gpu-clock-offset = Desplazamiento de Reloj GPU (MHz)
max-gpu-clock = Reloj GPU Máximo (MHz)
max-vram-clock = Reloj VRAM Máximo (MHz)
max-gpu-voltage = Voltaje GPU Máximo (mV)
min-gpu-clock = Reloj GPU Mínimo (MHz)
min-vram-clock = Reloj VRAM Mínimo (MHz)
min-gpu-voltage = Voltaje GPU Mínimo (mV)
gpu-voltage-offset = Desplazamiento de voltaje GPU (mV)
gpu-pstate-clock-offset = Estado-P de GPU { $pstate } Desplz. Reloj (MHz)
reset-all-graphs-tooltip = Restablecer Todas las Gráficas a Predeterminado
vram-pstate-clock-offset = VRAM P-State { $pstate } Desplazamiento de Reloj (MHz)
gpu-pstate-clock = GPU P-State { $pstate } Reloj (MHz)
mem-pstate-clock = VRAM P-State { $pstate } Reloj (MHz)
gpu-pstate-clock-voltage = GPU P-State { $pstate } Voltaje (mV)
mem-pstate-clock-voltage = VRAM P-State { $pstate } Voltaje (mV)
