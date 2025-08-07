gpu-clock-target = Clock Core GPU (Target)
disable-amd-oc-description = Questa operazione disabiliterà il supporto all'Overclocking AMD (overdrive) al prossimo riavvio di sistema.
device-not-found = { $kind } dispositivo non trovato
local-memory = Memoria Locale
cache-info = Informazioni Cache
watt = W
power-usage = Consumo Energetico
power-profile-mode = Modalità Profilo Di Alimentazione:
software-page = Software
hardware-info = Informazioni Hardware
system-section = Sistema
lact-gui = GUI LACT
lact-daemon = Demone LACT
instance = Istanza
device-name = Nome Dispositivo
platform-name = Nome Piattaforma
api-version = Versione API
version = Versione
cl-c-version = Versione OpenCL C
workgroup-size = Dimensione Gruppo Di Lavoro
features = Funzionalità
extensions = Estensioni
nvidia-cache-desc = { $size } L{ $level }
cache-data = Dati
cache-instruction = Dati
cache-cpu = CPU
monitoring-section = Monitoraggio
fan-control-section = Controllo Ventole
fan-speed = Velocità Ventole
throttling = Limitazione
auto-page = Automatico
curve-page = Curva
acoustic-limit = Limite Acustico (RPM)
acoustic-target = Target Acustico (RPM)
zero-rpm = Zero RPM
static-speed = Velocità Statica (%)
reset-button = Ripristina
pmfw-reset-warning = Avvertenza: questa operazione ripristina le impostazioni del firmware delle ventole!
enable-amd-oc = Abilita l'Overclocking AMD
disable-amd-oc = Disabilita l'Overclocking AMD
reset-config = Ripristina Configurazione
power-cap = Limite Consumo Energetico
ghz = GHz
mhz = MHz
stats-section = Statistiche
gpu-clock-avg = Clock Core GPU (Media)
gpu-voltage = Voltaggio GPU
gpu-temp = Temperatura GPU
gpu-usage = Utilizzo GPU
vram-clock = Clock VRAM
no-throttling = No
performance-level-high = Clock Più Alti
performance-level-low = Clock Più Bassi
performance-level-high-description = Utilizza sempre le velocità di clock più elevate per GPU e VRAM.
performance-level-manual-description = Controllo manuale delle prestazioni.
overclock-section = Velocità Clock e Voltaggio
nvidia-oc-info = Informazioni Overclocking Nvidia
driver-version = Versione Driver
amd-cache-desc =
    { $size } L{ $level } { $types } cache { $shared ->
        [1] locale per ciascun CU
       *[other] condivisa tra { $shared } CU
    }
zero-rpm-stop-temp = Temperatura di arresto a zero RPM (°C)
performance-level-auto-description = Regola automaticamente i clock della GPU e della VRAM. (Predefinito)
performance-level-low-description = Utilizza sempre le velocità di clock più basse per GPU e VRAM.
target-temp = Temperatura target (°C)
enable-amd-oc-description = Questo abiliterà la funzione di overdrive del driver amdgpu creando un file in <b>{ $path }</b> e aggiornando l'initramfs. Sei sicuro di voler procedere?
thermals-page = Temperature
global-memory = Memoria Globale
reset-config-description = Sei sicuro di voler ripristinare tutte le configurazioni della GPU?
mebibyte = MiB
gpu-clock = Clock Core GPU
kernel-version = Versione Kernel
driver-name = Nome Driver
oc-page = OC
temperatures = Temperature
oc-missing-fan-control-warning = Avvertenza: Il supporto all'overclocking è disattivato, la funzionalità di controllo delle ventole non è disponibile.
min-fan-speed = Velocità Minima Ventole (%)
amd-oc-disabled =
    Il supporto all'Overclocking AMD non è abilitato!
    Puoi comunque cambiare le impostazioni di base, ma il controllo avanzato delle frequenze e del voltaggio non sarà disponibile.
compute-units = Unità Di Calcolo
show-button = Mostra
unknown-throttling = Sconosciuto
static-page = Statico
manual-level-needed = Il livello delle prestazioni deve essere impostato su “manuale” per utilizzare gli stati e le modalità di alimentazione
nvidia-oc-description =
    La funzionalità di overclocking su Nvidia include l'impostazione di offset per le velocità di clock della GPU/VRAM e la limitazione dell'intervallo potenziale delle velocità di clock utilizzando la funzione “clock bloccati”.

    Su molte schede, l'offset della velocità di clock della VRAM influirà sulla velocità di clock effettiva della memoria solo per metà del valore dell'offset.
    Ad esempio, un offset VRAM di +1000 MHz può aumentare la velocità VRAM misurata solo di 500 MHz.
    Questo è normale ed è il modo in cui Nvidia gestisce le velocità di trasmissione dati GDDR. Regola l'overclock di conseguenza.

    Il controllo diretto del voltaggio non è supportato, poiché non esiste nel driver Linux di Nvidia.

    È possibile ottenere una specie di undervolt combinando l'opzione clock bloccati con un offset positivo della velocità di clock.
    Ciò costringerà la GPU a funzionare ad un voltaggio limitato dai clock bloccati, ottenendo al contempo una velocità di clock più elevata grazie all'offset.
    Se spinto a livelli troppo elevati potrebbe causare instabilità del sistema.
missing-stat = N/D
performance-level-auto = Automatico
performance-level-manual = Manuale
info-page = Informazioni
