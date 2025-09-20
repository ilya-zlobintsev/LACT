gpu-clock-target = Frequenza Core GPU (Target)
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
gpu-clock-avg = Frequenza Core GPU (Media)
gpu-voltage = Voltaggio GPU
gpu-temp = Temperatura
gpu-usage = Utilizzo GPU
vram-clock = Frequenza VRAM
no-throttling = No
performance-level-high = Frequenze Più Alte
performance-level-low = Frequenze Più Basse
performance-level-high-description = Utilizza sempre le frequenze più elevate per GPU e VRAM.
performance-level-manual-description = Controllo manuale delle prestazioni.
overclock-section = Frequenze e Voltaggio
nvidia-oc-info = Informazioni Overclocking Nvidia
driver-version = Versione Driver
amd-cache-desc =
    { $size } L{ $level } { $types } cache { $shared ->
        [1] locale per ciascun CU
       *[other] condivisa tra { $shared } CU
    }
zero-rpm-stop-temp = Temperatura di arresto a zero RPM (°C)
performance-level-auto-description = Regola automaticamente le frequenze della GPU e della VRAM. (Predefinito)
performance-level-low-description = Utilizza sempre le frequenze più basse per GPU e VRAM.
target-temp = Temperatura target (°C)
enable-amd-oc-description = Questo abiliterà la funzione di overdrive del driver amdgpu creando un file in <b>{ $path }</b> e aggiornando l'initramfs. Sei sicuro di voler procedere?
thermals-page = Temperature
global-memory = Memoria Globale
reset-config-description = Sei sicuro di voler ripristinare tutte le configurazioni della GPU?
mebibyte = MiB
gpu-clock = Frequenza Core GPU
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
    La funzionalità di overclocking su Nvidia include l'impostazione di offset per le frequenze del clock della GPU/VRAM e la limitazione dell'intervallo potenziale delle frequenze utilizzando la funzione “clock bloccati”.

    Su molte schede, l'offset della frequenza del clock della VRAM influirà sulla frequenza effettiva della memoria solo per metà del valore dell'offset.
    Ad esempio, un offset VRAM di +1000 MHz può aumentare la velocità VRAM misurata solo di 500 MHz.
    Questo è normale ed è il modo in cui Nvidia gestisce le velocità di trasmissione dati GDDR. Regola l'overclock di conseguenza.

    Il controllo diretto del voltaggio non è supportato, poiché non esiste nel driver Linux di Nvidia.

    È possibile ottenere una specie di undervolt combinando l'opzione clock bloccati con un offset positivo della frequenza del clock.
    Ciò costringerà la GPU a funzionare ad un voltaggio limitato dai clock bloccati, ottenendo al contempo una frequenza del clock più elevata grazie all'offset.
    Se spinto a livelli troppo elevati potrebbe causare instabilità del sistema.
missing-stat = N/D
performance-level-auto = Automatico
performance-level-manual = Manuale
info-page = Informazioni
amd-oc = AMD Overclocking
enable-vram-locked-clocks = Abilita Frequenze Bloccate VRAM
enable-gpu-locked-clocks = Abilita Frequenze Bloccate GPU
amd-oc-detected-system-config =
    Configurazione di sistema rilevata: <b>{ $config ->
        [unsupported] Non supportato
       *[other] { $config }
    }</b>
no-clocks-data = Nessun dato sulle frequenze disponibile
reset-oc-tooltip = Avvertenza: questa operazione riprtistina tutte le impostazioni delle frequenze ai valori predefiniti!
max-gpu-clock = Frequenza Massima GPU (MHz)
max-vram-clock = Frequenza Massima VRAM (MHz)
min-vram-clock = Frequenza Minima VRAM (MHz)
min-gpu-clock = Frequenza Minima GPU (MHz)
pstate-list-description = <b>I seguenti valori sono gli offset di frequenza per ciascun P-State, dal più alto al più basso.</b>
gpu-clock-offset = Offset Frequenza GPU (MHz)
amd-oc-status =
    L'Overclocking AMD è attualmente: <b>{ $status ->
        [true] Abilitato
        [false] Disabilitato
       *[other] Sconosciuto
    }</b>
amd-oc-updating-configuration = Aggiornamento della configurazione (potrebbe richiedere un po' di tempo)
any-rules-matched = Una delle seguenti regole è soddisfatta:
all-rules-matched = Tutte le seguenti regole sono soddisfatte:
activation-settings-status =
    Le impostazioni di attivazione selezionate sono attualmente <b>{ $matched ->
        [true] soddisfatte
       *[false] non soddisfatte
    }</b>
activation-auto-switching-disabled = Il cambio automatico del profilo è attualmente disabilitato
profile-hook-command = Esegui un comando quando il profilo '{ $cmd }' è:
profile-hook-activated = Attivato:
profile-hook-deactivated = Disattivato:
profile-rule-process-tab = Un processo è in esecuzione
oc-warning = Avvertenza: la modifica di questi valori può causare instabilità del sistema e potenzialmente danneggiare l'hardware!
show-all-pstates = Mostra tutti i P-States
max-gpu-voltage = Voltaggio Massimo GPU (mV)
min-gpu-voltage = Voltaggio Minimo GPU (mV)
gpu-voltage-offset = Offset voltaggio GPU (mV)
gpu-pstate-clock-offset = Offset Frequenza P-State { $pstate } GPU (MHz)
gpu-pstate-clock = Frequenza P-State { $pstate } GPU (MHz)
mem-pstate-clock = Frequenza P-State { $pstate } VRAM (MHz)
mem-pstate-clock-voltage = Voltaggio P-State { $pstate } VRAM (mV)
pstates = Stati di Alimentazione (P-States)
vram-pstates = Stati di Alimentazione VRAM
enable-pstate-config = Abilita configurazione degli stati di alimentazione
show-historical-charts = Mostra Grafici Storici
settings-profile = Impostazioni Profilo
add-profile = Aggiungi nuovo profilo
import-profile = Importa profilo da file
profile-copy-from = Copia le impostazioni da:
rename-profile = Rinomina Profilo
rename-profile-from = Rinomina il profilo <b>{ $old_name }</b> in:
delete-profile = Elimina Profilo
edit-rules = Modifica Regole
export-to-file = Esporta Su File
move-up = Sposta in alto
move-down = Sposta in basso
profile-activation = Attivazione
profile-hooks = Funzioni Hook
profile-activation-desc = Attiva il profilo '{ $name }' quando:
profile-rule-gamemode-tab = Gamemode è attivo
profile-rule-process-name = Nome Processo:
profile-rule-args-contain = Argomenti Inclusi:
profile-rule-specific-process = Con un processo specifico:
amd-oc-updating-done = Configurazione aggiornata, riavviare il sistema per applicare le modifiche.
pstates-manual-needed = Nota: il livello di prestazioni deve essere impostato su “manuale” per attivare gli stati di alimentazione
auto-switch-profiles = Cambia automaticamente
profile-rules = Regole Profilo
amd-oc-description =
    { $config ->
        [rpm-ostree] Questa opzione attiverà o disattiverà il supporto a AMD Overdrive impostando i flag di avvio tramite <b>rpm-ostree</b>.
        [unsupported]
            Il sistema attuale non è riconosciuto come supportato per la configurazione automatica dell'overdrive.
            È possibile provare ad abilitare l'overclocking da LACT, ma potrebbe essere necessaria una rigenerazione manuale dell'initramfs affinché abbia effetto.
            Se ciò non funziona, un'opzione alternativa è aggiungere <b>amdgpu.ppfeaturemask=0xffffffff</b> come parametro di avvio nel bootloader.
       *[other] Questa opzione attiverà/disattiverà il supporto a AMD Overdrive creando un file in <b>{ $path }</b> e aggiornando l'initramfs..
    }

    Consulta <a href="https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)">la wiki</a> per ulteriori informazioni.
vram-pstate-clock-offset = Offset Frequenza P-State { $pstate } VRAM (MHz)
gpu-pstate-clock-voltage = Voltaggio P-State { $pstate } GPU (mV)
gpu-pstates = Stati di Alimentazione GPU
create-profile = Crea Profilo
profile-hook-note = Nota: questi comandi vengono eseguiti come root dal demone LACT e non hanno accesso all'ambiente desktop. Pertanto, non possono essere utilizzati direttamente per avviare applicazioni grafiche.
name = Nome
create = Crea
cancel = Annulla
save = Salva
edit-rule = Modifica Regola
remove-rule = Rimuovi Regola
default-profile = Predefinito
reconnecting-to-daemon = Connessione al demone persa, riconnnessione in corso...
daemon-connection-lost = Connessione Persa
plot-show-detailed-info = Mostra informazioni dettagliate
show-process-monitor = Mostra Monitoraggio Processi
generate-debug-snapshot = Genera Snapshot Di Debug
dump-vbios = Esporta VBIOS
reset-all-config = Ripristina Tutte Le Configurazioni
stats-update-interval = Intervallo Di Aggiornamento (ms)
