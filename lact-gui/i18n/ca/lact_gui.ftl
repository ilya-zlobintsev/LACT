compute-units = Unitats de càlcul
info-page = Informació
oc-page = OC
thermals-page = Tèrmics
software-page = Programari
hardware-info = Informació del maquinari
system-section = Sistema
lact-daemon = Dimoni LACT
lact-gui = LACT GUI
kernel-version = Versió del nucli
instance = Instància
device-name = Nom del dispositiu
platform-name = Nom de la plataforma
api-version = Versió de l'API
version = Versió
driver-name = Nom del controlador
driver-version = Versió del controlador
cl-c-version = Versió d'OpenCL C
workgroup-size = Mida del grup de treball
global-memory = Memòria global
local-memory = Memòria local
features = Característiques
extensions = Extensions
show-button = Mostra
device-not-found = No s'ha trobat el dispositiu { $kind }
cache-info = Informació de la memòria cau
amd-cache-desc =
    { $size } L{ $level } { $types } de cau { $shared ->
        [1] local per a cada CU
       *[other] compartida entre { $shared } CUs
    }
nvidia-cache-desc = { $size } L{ $level }
cache-data = Dades
cache-instruction = Dades
cache-cpu = CPU
monitoring-section = Monitoratge
fan-control-section = Control del ventilador
temperatures = Temperatures
oc-missing-fan-control-warning = Avís: el suport d'overclocking està desactivat, la funcionalitat de control del ventilador no està disponible.
fan-speed = Velocitat del ventilador
throttling = Limitació
auto-page = Automàtic
curve-page = Corba
static-page = Estàtic
target-temp = Temperatura objectiu (°C)
acoustic-limit = Límit acústic (RPM)
acoustic-target = Objectiu acústic (RPM)
min-fan-speed = Velocitat mínima del ventilador (%)
zero-rpm = Zero RPM
zero-rpm-stop-temp = Temperatura d'aturada Zero RPM (°C)
static-speed = Velocitat estàtica (%)
reset-button = Restableix
pmfw-reset-warning = Avís: això restableix la configuració del firmware del ventilador!
temperature-sensor = Sensor de temperatura
spindown-delay = Retard de desacceleració (ms)
spindown-delay-tooltip = Quant de temps ha de romandre la GPU a una temperatura més baixa abans de reduir la velocitat del ventilador
speed-change-threshold = Llindar de canvi de velocitat (°C)
automatic-mode-threshold = Llindar del mode automàtic (°C)
automatic-mode-threshold-tooltip =
    Canvia el control del ventilador al mode automàtic quan la temperatura estigui per sota d'aquest punt.

    Moltes GPU Nvidia només admeten aturar el ventilador en el mode de control automàtic, mentre que una corba personalitzada té un rang de velocitat limitat, com ara 30-100%.

    Aquesta opció permet eludir aquesta limitació utilitzant només la corba personalitzada quan es supera una temperatura específica, utilitzant el mode automàtic integrat de la targeta que admet zero RPM per sota d'ella.
amd-oc = Overclocking d'AMD
amd-oc-disabled =
    El suport d'Overclocking d'AMD no està habilitat!
    Encara podeu canviar la configuració bàsica, però els rellotges més avançats i el control de voltatge no estaran disponibles.
amd-oc-status =
    L'Overclocking d'AMD està actualment: <b>{ $status ->
        [true] Habilitat
        [false] Inhabilitat
       *[other] Desconegut
    }</b>
amd-oc-detected-system-config =
    Configuració del sistema detectada: <b>{ $config ->
        [unsupported] No suportada
       *[other] { $config }
    }</b>
enable-amd-oc-description = Això habilitarà la característica overdrive del controlador amdgpu creant un fitxer a <b>{ $path }</b> i actualitzant l'initramfs. Segur que voleu fer això?
disable-amd-oc = Inhabilita l'Overclocking d'AMD
enable-amd-oc = Habilita l'Overclocking d'AMD
disable-amd-oc-description = Això inhabilitarà el suport d'overclocking d'AMD (overdrive) en el proper reinici.
amd-oc-updating-configuration = S'està actualitzant la configuració (això pot trigar una estona)
amd-oc-updating-done = Configuració actualitzada, reinicieu per aplicar els canvis.
reset-config = Restableix la configuració
reset-config-description = Segur que voleu restablir tota la configuració de la GPU?
apply-button = Aplica
revert-button = Reverteix
power-cap = Límit d'ús d'energia
watt = W
ghz = GHz
mhz = MHz
mebibyte = MiB
stats-section = Estadístiques
gpu-clock = Rellotge del nucli de la GPU
gpu-clock-avg = Rellotge del nucli de la GPU (Mitjana)
gpu-clock-target = Rellotge del nucli de la GPU (Objectiu)
gpu-voltage = Voltatge de la GPU
gpu-temp = Temperatura
gpu-usage = Ús de la GPU
vram-clock = Rellotge de la VRAM
power-usage = Ús d'energia
no-throttling = No
unknown-throttling = Desconegut
missing-stat = N/A
vram-usage = Ús de la VRAM:
performance-level-auto = Automàtic
performance-level-high = Rellotges més alts
performance-level-low = Rellotges més baixos
performance-level-manual = Manual
performance-level-auto-description = Ajusta automàticament els rellotges de la GPU i la VRAM. (Predeterminat)
performance-level-high-description = Utilitza sempre les velocitats de rellotge més altes per a la GPU i la VRAM.
performance-level-low-description = Utilitza sempre les velocitats de rellotge més baixes per a la GPU i la VRAM.
performance-level-manual-description = Control de rendiment manual.
performance-level = Nivell de rendiment
power-profile-mode = Mode de perfil d'energia:
overclock-section = Velocitat de rellotge i voltatge
nvidia-oc-info = Informació d'Overclocking de Nvidia
oc-warning = Avís: canviar aquests valors pot provocar inestabilitat del sistema i pot danyar potencialment el vostre maquinari!
show-all-pstates = Mostra tots els estats P
enable-gpu-locked-clocks = Habilita els rellotges bloquejats de la GPU
enable-vram-locked-clocks = Habilita els rellotges bloquejats de la VRAM
pstate-list-description = <b>Els valors següents són compensacions de rellotge per a cada estat P, anant de més alt a més baix.</b>
no-clocks-data = No hi ha dades de rellotges disponibles
reset-oc-tooltip = Avís: això restableix tota la configuració del rellotge als valors predeterminats!
gpu-clock-offset = Compensació del rellotge de la GPU (MHz)
max-gpu-clock = Rellotge màxim de la GPU (MHz)
max-vram-clock = Rellotge màxim de la VRAM (MHz)
max-gpu-voltage = Voltatge màxim de la GPU (mV)
min-gpu-clock = Rellotge mínim de la GPU (MHz)
min-vram-clock = Rellotge mínim de la VRAM (MHz)
min-gpu-voltage = Voltatge mínim de la GPU (mV)
gpu-voltage-offset = Compensació del voltatge de la GPU (mV)
gpu-pstate-clock-offset = Compensació rellotge estat P { $pstate } GPU (MHz)
vram-pstate-clock-offset = Compensació rellotge estat P { $pstate } VRAM (MHz)
gpu-pstate-clock = Rellotge estat P { $pstate } GPU (MHz)
mem-pstate-clock = Rellotge estat P { $pstate } VRAM (MHz)
gpu-pstate-clock-voltage = Voltatge estat P { $pstate } GPU (mV)
mem-pstate-clock-voltage = Voltatge estat P { $pstate } VRAM (mV)
pstates = Estats d'energia
gpu-pstates = Estats d'energia de la GPU
vram-pstates = Estats d'energia de la VRAM
pstates-manual-needed = Nota: el nivell de rendiment s'ha d'establir a 'manual' per commutar els estats d'energia
enable-pstate-config = Habilita la configuració de l'estat d'energia
show-historical-charts = Mostra els gràfics històrics
show-process-monitor = Mostra el monitor de processos
generate-debug-snapshot = Genera una captura de depuració
dump-vbios = Bolca la VBIOS
reset-all-config = Restableix tota la configuració
stats-update-interval = Interval d'actualització (ms)
historical-data-title = Dades històriques
graphs-per-row = Gràfics per fila:
time-period-seconds = Període de temps (segons):
reset-all-graphs-tooltip = Restableix tots els gràfics
add-graph = Afegeix un gràfic
delete-graph = Suprimeix el gràfic
edit-graphs = Edita
export-csv = Exporta com a CSV
edit-graph-sensors = Edita els sensors del gràfic
reconnecting-to-daemon = S'ha perdut la connexió amb el dimoni, s'està reconnectant...
daemon-connection-lost = Connexió perduda
plot-show-detailed-info = Mostra informació detallada
settings-profile = Perfil de configuració
auto-switch-profiles = Canvia automàticament
add-profile = Afegeix un perfil nou
import-profile = Importa el perfil des d'un fitxer
create-profile = Crea un perfil
name = Nom
profile-copy-from = Copia la configuració de:
create = Crea
cancel = Cancel·la
save = Desa
default-profile = Predeterminat
rename-profile = Reanomena el perfil
rename-profile-from = Reanomena el perfil <b>{ $old_name }</b> a:
delete-profile = Suprimeix el perfil
edit-rules = Edita les regles
edit-rule = Edita la regla
remove-rule = Suprimeix la regla
profile-rules = Regles del perfil
export-to-file = Exporta a un fitxer
move-up = Mou amunt
move-down = Mou avall
profile-activation = Activació
profile-hooks = Ganxos
profile-activation-desc = Activa el perfil '{ $name }' quan:
any-rules-matched = Qualsevol de les regles següents coincideixi:
all-rules-matched = Totes les regles següents coincideixin:
activation-settings-status =
    La configuració d'activació seleccionada està actualment <b>{ $matched ->
        [true] coincident
       *[false] no coincident
    }</b>
activation-auto-switching-disabled = El canvi automàtic de perfil està actualment desactivat
profile-hook-command = Executa una ordre quan el perfil '{ $cmd }' estigui:
profile-hook-activated = Activat:
profile-hook-deactivated = Desactivat:
profile-hook-note = Nota: aquestes ordres s'executen com a root pel dimoni LACT i no tenen accés a l'entorn d'escriptori. Com a tal, no es poden utilitzar directament per llançar aplicacions gràfiques.
profile-rule-process-tab = S'està executant un procés
profile-rule-gamemode-tab = El mode de joc està actiu
profile-rule-process-name = Nom del procés:
profile-rule-args-contain = Els arguments contenen:
profile-rule-specific-process = Amb un procés específic:
amd-oc-description =
    { $config ->
        [rpm-ostree] Aquesta opció activarà o desactivarà el suport per a AMD overdrive establint paràmetres de boot mitjançant <b>rpm-ostree</b>.
        [unsupported]
            El sistema actual no es reconeix com a compatible per a la configuració automàtica d'overdrive.
            Pots intentar activar l'overclocking des de LACT, però podria ser necessari regenerar manualment l'initramfs perquè tingui efecte.
            Si això falla, una opció alternativa és afegir <b>amdgpu.ppfeaturemask=0xffffffff</b> com a paràmetre de boot al bootloader.
       *[other] Aquesta opció activarà o desactivarà el suport per a AMD overdrive creant un fitxer a <b>{ $path }</b> i actualitzant l'initramfs.
    }

    Consulta <a href="https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)">la wiki</a> per a més informació.
manual-level-needed = El nivell de rendiment ha d'estar configurat a "manual" per utilitzar els estats i modes d'energia
nvidia-oc-description =
    La funcionalitat d'overclocking a Nvidia inclou la configuració d'increments per a les velocitats de rellotge de la GPU/VRAM i la limitació de l'interval potencial de velocitats de rellotge mitjançant la funció "locked clocks".

    A moltes targetes, l'increment de la velocitat de rellotge de la VRAM només afectarà la velocitat real de la memòria per la meitat del valor de l'increment.
    Per exemple, un increment de +1000MHz a la VRAM podria augmentar la velocitat mesurada de la VRAM només 500MHz.
    Això és normal i és com Nvidia gestiona les velocitats de dades GDDR. Ajusta el teu overclocking en conseqüència.

    El control directe de voltatge no està suportat, ja que no existeix en el controlador Nvidia per a Linux.

    És possible aconseguir un pseudo-undervolt combinant l'opció de rellotges bloquejats amb un increment de la velocitat de rellotge.
    Això forçarà la GPU a funcionar a un voltatge que està limitat pels rellotges bloquejats, mentre s'aconsegueix una velocitat de rellotge més alta a causa de l'increment.
    Això pot causar inestabilitat del sistema si s'augmenta massa.
