oc-page = OC
thermals-page = Thermik
software-page = Software
lact-gui = LACT GUI
kernel-version = Kernel Version
instance = Instanz
platform-name = Plattform Name
api-version = API Version
driver-name = Treiber Name
driver-version = Treiber Version
compute-units = Compute Units
cl-c-version = Wersja OpenCL C
local-memory = Lokaler Arbeitsspeicher
features = Funktionalitäten
extensions = Erweiterungen
cache-info = Cache Informationen
nvidia-cache-desc = { $size } L{ $level }
monitoring-section = Überwachung
fan-control-section = Lüfter Kontrolle
temperatures = Temperaturen
fan-speed = Lüfter Geschwindigkeit
throttling = Drosselung
auto-page = Automatik
curve-page = Kurve
acoustic-limit = Akustik Limit (RPM)
acoustic-target = Akustik Ziel (RPM)
min-fan-speed = Minimale Lüftergeschwindigkeit (%)
zero-rpm = Null RPM
device-not-found = { $kind } Gerät nicht gefunden
system-section = System
oc-missing-fan-control-warning = Warnung: Unterstützung für Übertacktung wurde deaktiviert, Lüftersteuerung ist nicht verfügbar.
amd-cache-desc =
    { $size } L{ $level } { $types } Cache { $shared ->
        [1] lokal pro CU
       *[other] mit { $shared } CUs geteilt
    }
hardware-info = Hardware Informationen
lact-daemon = LACT Daemon
target-temp = Zieltemperatur (°C)
global-memory = Globaler Arbeitsspeicher
device-name = Gerätename
show-button = Anzeigen
cache-data = Daten
cache-instruction = Daten
cache-cpu = CPU
static-page = Statisch
version = Version
info-page = Informationen
zero-rpm-stop-temp = Zero RPM Stop Temperatur (°C)
static-speed = Konstante Geschwindigkeit (%)
reset-button = Zurücksetzen
pmfw-reset-warning = Warnung: Dies setzt die Lüfter-Firmware zurück!
reset-oc-tooltip = Warnung: Die wird alle Takteinstellungen auf Standardwerte zurücksetzen!
amd-oc = AMD Overclocking
amd-oc-status =
    AMD Overclocking ist aktuell: <b>{ $status ->
        [true] Aktiviert
        [false] Deaktiviert
       *[other] Unbekannt
    }</b>
amd-oc-detected-system-config =
    Ermittelte Systemkonfiguration: <b>{ $config ->
        [unsupported] Nicht unterstützt
       *[other] { $config }
    }</b>
disable-amd-oc = AMD Overclocking deaktivieren
enable-amd-oc = AMD Overclocking aktivieren
amd-oc-updating-configuration = Aktualisieren der Konfiguration (die kann etwas dauern)
amd-oc-updating-done = Konfiguration aktualisiert, bitte Systemneustart durchführen um die Änderungen anzuwenden.
reset-config = Konfiguration zurücksetzen
reset-config-description = Sind Sie sicher, dass die gesamte GPU Konfiguration zurücksetzen wollen?
power-cap = Leistungsaufnamelimit
watt = W
ghz = GHz
stats-section = Statistiken
gpu-clock = GPU Kern Takt
vram-clock = VRAM Takt
unknown-throttling = Unbekannt
missing-stat = N/A
performance-level-high = Höchster Takt
performance-level-low = Niedrigster Takt
performance-level-manual = Manuell
power-profile-mode = Leistungsprofil Modus:
manual-level-needed = Um Energiezustände und -modi nutzen zu können, muss die Leistungsstufe auf „manuell“ eingestellt werden
overclock-section = Takt und Spannung
nvidia-oc-info = Nvidia Overclocking Informationen
show-all-pstates = Zeige alle P-States
enable-gpu-locked-clocks = Aktiviere GPU Takt Sperre
pstate-list-description = <b>Die folgenden Werte zeigen den Taktraten Versatz für jeden P-State, von Höchstem zu Niedrigstem.</b>
no-clocks-data = Keine Informationen zum Takt verfügbar
gpu-clock-offset = GPU Takt-Offset (MHz)
max-gpu-clock = Maximaler GPU Takt (MHz)
max-gpu-voltage = Maximale GPU Spannung (mV)
min-gpu-clock = Minimaler GPU Takt (MHz)
min-vram-clock = Minimaler VRAM Takt (MHz)
gpu-voltage-offset = GPU Spannungs-Offset (mV)
vram-pstate-clock-offset = VRAM P-State { $pstate } Takt-Offset (MHz)
gpu-pstate-clock = GPU P-State { $pstate } Takt (MHz)
mem-pstate-clock = VRAM P-State { $pstate } Takt (MHz)
gpu-pstate-clock-voltage = GPU P-State { $pstate } Spannung (mV)
mem-pstate-clock-voltage = VRAM P-State { $pstate } Spannung (mV)
pstates = Power States
gpu-pstates = GPU Power States
vram-pstates = VRAM Power States
power-usage = Leistungsaufnahme
performance-level-high-description = Verwende immer den höchsten Takt für GPU und VRAM.
performance-level-low-description = Verwende immer den niedrigsten Takt für GPU und VRAM.
performance-level-manual-description = Manuelle Leistungskontrolle.
gpu-pstate-clock-offset = GPU P-State { $pstate } Takt-Offset (MHz)
gpu-usage = GPU Auslastung
min-gpu-voltage = Minimale GPU Spannung (mV)
performance-level-auto-description = Automatische Anpassung des GPU und VRAM Takts. (Standard)
amd-oc-disabled =
    AMD Overclocking Unterstützung ist nicht aktiv!
    Sie können immer noch Basis-Einstellungen anpassen, aber die erweiterten Takt- und Spannungseinstellungen werden nicht verfügbar sein.
disable-amd-oc-description = Dies wird die AMD Overclocking Unterstützung (Overdrive) für den nächsten System-Neustart deaktivieren.
amd-oc-description =
    { $config ->
        [rpm-ostree] Diese Option aktviert/deaktiviert die AMD Overdrive Unterstützung in dem entsprechende Boot-Flags mittels  <b>rpm-ostree</b> gesetzt werden.
        [unsupported]
            Das System unterstützt die automatische Overdrive Funktion aktuell nicht.
            Sie könnten versuche, Overclocking manuell in LACT zu aktivieren, aber die könnte eine manuelle Neuerstellung des initramfs erfordern. 
            Falls die nicht funktioniert, kann als Fallback Lösung der Bootparameter <b>amdgpu.ppfeaturemask=0xffffffff</b> im Bootloader hinzugefügt werden.
       *[other] Diese Option aktiviert/deaktiviert die AMD Overdrive Unterstützung, in dem die Datei  <b>{ $path }</b> erstellt und das initramfs aktualisiert wird.
    }

    Für weitere Informationen besuchen sie  <a href="https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)">das Wiki</a>.
gpu-clock-avg = GPU Kern Takt (Durchschnitt)
max-vram-clock = Maximaler VRAM Takt (MHz)
enable-amd-oc-description = Dies wird die Overdrive Funktionen des amdgpu Treibers aktivieren indem die Datei <b>{ $path }</b> erstellt und das initramfs aktualisiert wird. Sind Sie sicher, dass die dies tun wollen?
gpu-clock-target = GPU Kern Takt (Ziel)
gpu-voltage = GPU Spannung
gpu-temp = GPU Temperatur
oc-warning = Warnung: Änderungen an diesen Werten kann zu Instabilität des Systems führen und möglicherweise die Hardware beschädigen!
enable-vram-locked-clocks = Aktiviere VRAM Takt Sperre
no-throttling = Nein
performance-level-auto = Automatisch
add-profile = Neues Profil hinzufügen
import-profile = Profil aus Datei importieren
auto-switch-profiles = Automatisch umschalten
all-rules-matched = Alle der folgenden Regeln treffen zu:
any-rules-matched = Jeder der folgenden Regeln trifft zu:
profile-activation-desc = Aktiviere Profil '{ $name }' wenn:
enable-pstate-config = Power-State Konfiguration aktivieren
show-historical-charts = Zeige historische Daten
create-profile = Profil erstellen
profile-copy-from = Einstellungen übernehmen von:
create = Erstelle
delete-profile = Profil löschen
edit-rules = Regeln bearbeiten
remove-rule = Regel löschen
profile-rules = Profilregeln
export-to-file = In Datei exportieren
move-up = Nach oben bewegen
profile-activation = Aktivierung
profile-hooks = Hooks
move-down = Nach unten bewegen
rename-profile = Profil umbenennen
pstates-manual-needed = Hinweis: Um Power-States ändern zu können, muss das Performance-Level auf "manuell" gesetzt werden
edit-rule = Regel bearbeiten
rename-profile-from = Umbenennen des Profils <b>{ $old_name }</b> in:
activation-settings-status =
    Die ausgewählten Aktivierungseinstellungen <b>{ $matched ->
        [true] treffen zu
       *[false] treffen nicht zu
    }</b>
name = Name
cancel = Abbrechen
default-profile = Standard
save = Speichern
settings-profile = Einstellungsprofil
profile-rule-process-name = Prozessname:
mhz = MHz
profile-hook-command = Einen Befehl ausführen, wenn das Profil { $cmd } ist:
profile-hook-activated = Aktiviert:
profile-hook-deactivated = Deaktiviert:
profile-rule-process-tab = Ein Prozess läuft gerade
profile-rule-gamemode-tab = Gamemode ist aktiv
profile-rule-args-contain = Argumente enthalten:
profile-rule-specific-process = Mit einem spezifischen Prozess:
activation-auto-switching-disabled = Der automatische Profilwechsel ist momentan deaktiviert
profile-hook-note = Hinweis: Diese Befehle werden als root durch den LACT Daemon ausgeführt und haben keinen Zugriff auf die Desktopumgebung. Somit können sie keine grafischen Anwendungen starten.
nvidia-oc-description =
    Die Übertaktungsfunktionen bei Nvidia beinhalten das Festlegen von Offsets für die GPU- und VRAM-Taktraten sowie das Einschränken des möglichen Taktbereichs durch die Funktion "gesperrte Takte".

    Bei vielen Grafikkarten wirkt sich der VRAM-Takt-Offset nur zur Hälfte auf die tatsächlich gemessenen Speichertakt aus.
    Zum Beispiel kann ein VRAM-Offset von +1000 MHz die gemessene VRAM-Takt nur um 500 MHz erhöhen.
    Das ist normal und entspricht der Art und Weise, wie Nvidia mit GDDR-Datenraten umgeht. Passe dein Overclocking entsprechend an.

    Eine direkte Spannungsregelung wird nicht unterstützt, da sie im Nvidia-Linux-Treiber nicht existiert.

    Ein sogenanntes „Pseudo-Undervolting“ ist möglich, indem man die Option „gesperrten Takte“ mit einem positiven Takt-Offset kombiniert.
    Dies zwingt die GPU dazu, mit einer durch die gesperrten Takte begrenzten Spannung zu arbeiten, während durch den Offset eine höhere Taktrate erreicht wird.
    Wird diese Einstellung zu aggressiv gewählt, kann sie zu Systeminstabilität führen.
mebibyte = MiB
reconnecting-to-daemon = Verbindung zum Daemon verloren, verbinde neu...
daemon-connection-lost = Verbindung verloren
plot-show-detailed-info = Genaue Informationen anzeigen
workgroup-size = Arbeitsgruppengröße
temperature-sensor = Temperatursensor
