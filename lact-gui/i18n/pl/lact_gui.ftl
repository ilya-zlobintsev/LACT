compute-units = Jednostki Obliczeniowe
kernel-version = Wersja jądra
gpu-usage = Użycie GPU
workgroup-size = Rozmiar Grupy
hardware-info = Informacje o sprzęcie
thermals-page = Zarządzanie temperaturą
software-page = Informacje o oprogramowaniu
system-section = System
lact-daemon = LACT Demon
lact-gui = Środowisko Graficzne LACT
instance = Środowisko
driver-name = Nazwa Sterownika
device-name = Nazwa urządzenia
platform-name = Nazwa Platformy
api-version = Wersja API
version = Wersja
cl-c-version = Wersja OpenCL C
local-memory = Pamięć Lokalna
features = Funkcje
extensions = Dodatki
show-button = Wyświetl
cache-info = Informacje o Pamięci
global-memory = Dostępna Pamięć Całkowita
monitoring-section = Zarządzanie
temperatures = Temperatury
throttling = Ograniczenie
auto-page = Automatycznie
curve-page = Krzywa
static-page = Statyczne
acoustic-limit = Limit Hałasu (RPM)
zero-rpm = Tryb Passywny
zero-rpm-stop-temp = Temperatura wył. temperatury pasywnej
static-speed = Stała Prędkość (%)
reset-button = Przywróć
watt = w
ghz = GHz
mhz = MHz
gpu-clock = Częstotliwość rdzenia GPU
gpu-clock-avg = Średnia Częstotliwość Rdzenia GPU
gpu-clock-target = Wyznaczona Częstotliwość Rdzenia GPU
gpu-voltage = Napięcie GPU
gpu-temp = Temperatura
vram-clock = Częstotliwość VRAM
power-usage = Zużycie Energi
driver-version = Wersja Sterownika
device-not-found = { $kind } nie odnaleziono urządzenia
target-temp = Wyznaczona temperatura (°C)
stats-section = Statystyki
power-cap = Limit Energi
fan-speed = Prędkość Wentylatora
min-fan-speed = Minimalna Prędkość Wentylatora (%)
vram-pstate-clock-offset = VRAM P-State { $pstate } Zegar Offset (MHz)
auto-switch-profiles = Zmień automatycznie
performance-level-auto = Automatycznie
min-vram-clock = Minimalne Zegary VRAM (MHz)
performance-level-auto-description = Automatycznie dopasuj zegary GPU I VRAM. (Podstawowe)
reset-oc-tooltip = Ostrzeżenie: to spowoduje zresetowanie wszystkich zegarów do domyślnych!
max-gpu-clock = Maksymalne Zegary Offset GPU (MHz)
all-rules-matched = Jeśli spełnione są wszystkie z poniższych warunków:
pstates-manual-needed = Poziom wydajności musi być ustawiony na „ręczny”, aby można było przełączać stany zasilania
settings-profile = Profile Ustawień
save = Zapisz
rename-profile-from = Zmień nazwę profilu <b>{ $old_name }</b> na:
nvidia-oc-description =
    Zmiany ustawień obejmują offsety zegarów GPU i VRAM, a także ograniczenie maksymalnych wartości zegarów przy użyciu zablokowanych „funkcji”

    Na wielu kartach graficznych offset dla taktowania VRAM wpływa na rzeczywiste taktowanie pamięci tylko w połowie wartości offsetu.
    Przykład: Offset +1000 MHz dla VRAM może zwiększyć rzeczywistą częstotliwość pamięci tylko o 500 MHz..
    To jest normalne — tak właśnie Nvidia obsługuje prędkości przesyłu danych w pamięci GDDR. Dostosuj swoje ustawienia podkręcania odpowiednio do tego zachowania.

    Bezpośrednie sterowanie napięciem nie jest obsługiwane, ponieważ taka funkcja nie istnieje w sterowniku Nvidia dla systemu Linux.

    Możliwe jest jednak osiągnięcie pseudo-undervoltu, łącząc zablokowane zegary z dodatnim offsetem
    Wymusza to pracę GPU przy napięciu ograniczonym przez ustawione zegary, ale jednocześnie umożliwia wyższą częstotliwość dzięki offsetowi.
    Zbyt duża wartość może prowadzić do niestabilności systemu.
profile-hook-deactivated = Dezaktywowana:
info-page = Informacje o sprzęcie
oc-page = Tryb OC
fan-control-section = Sterowanie chłodzenia
nvidia-cache-desc = { $size } L{ $level }
cache-instruction = Dane
cache-cpu = Procesor
cache-data = Dane
amd-cache-desc =
    { $size } L{ $level } { $types } pamięć { $shared ->
        [1] lokalna dla każdej JO
       *[other] współdzielone z { $shared } JO
    }
oc-missing-fan-control-warning = Uwaga: Modyfikacja jest zablokowana, ustawienia wentylatora nie są dostępne.
acoustic-target = Wyznaczony poziomu hałasu (RPM)
amd-oc = Zarządzanie AMD
amd-oc-status =
    Zarządzanie AMD OC jest obecnie: <b>{ $status ->
        [true] Dostępne
        [false] Zablokowane
       *[other] Nieznane
    }</b>
amd-oc-detected-system-config =
    Wykryto konfiguracje systemową: <b>{ $config ->
        [unsupported] Niewspieraną
       *[other] { $config }
    }</b>
disable-amd-oc = Wyłącz AMD Overlocking
enable-amd-oc = Włącz AMD Overclocking
disable-amd-oc-description = To spowoduje wyłączenie wsparcia AMD Overclocking (zaawansowanego) przy następnym restarcie.
amd-oc-updating-configuration = Aktualizowanie konfiguracji (to może chwile potrwać)
amd-oc-updating-done = Konfiguracja została zaktualizowana, potrzebny restart aby zastosować zmiany.
reset-config = Przywróć Konfiguracje
reset-config-description = Czy na pewno chcesz zresetować ustawienia Karty?
no-throttling = Nie
unknown-throttling = Nieznane
missing-stat = Nie dotyczy
mebibyte = Mebibajt
performance-level-high = Najwyższe Zegary
performance-level-low = Najniższe Zegary
performance-level-manual = Ręczne
performance-level-high-description = Zawsze używaj najwyższych zegarów dla GPU i VRAM.
performance-level-low-description = Zawsze używaj najniższych zegarów dla GPU i VRAM.
performance-level-manual-description = Ręczne sterowanie wydajnością.
power-profile-mode = Profil Trybu Zasilania:
manual-level-needed = Poziom wydajności został ustawiony jako Ręczny aby uaktywnić profile mocy
overclock-section = Częstotliwość Zegarów oraz Napieć
nvidia-oc-info = Zarządzanie informacjami OC Nvidia
show-all-pstates = Pokaż wszystkie P-States
enable-gpu-locked-clocks = Odblokuj Zablokowane Zegary GPU
enable-vram-locked-clocks = Odblokuj Zablokowane Zegary VRAM
no-clocks-data = Brak danych o zegarach
gpu-clock-offset = Zegary Offset GPU (MHz)
max-vram-clock = Maksymalne Zegary Offset VRAM (MHz)
max-gpu-voltage = Maksymalne Napięcie GPU (mV)
min-gpu-clock = Minimalne Zegary GPU (MHz)
min-gpu-voltage = Minimalne Napięcie GPU (mV)
gpu-voltage-offset = Off-set Napięcia GPU (mV)
gpu-pstate-clock-offset = GPU P-State { $pstate } Zegar Offset (MHz)
gpu-pstate-clock = GPU P-State { $pstate } Zegar (MHz)
gpu-pstate-clock-voltage = GPU P-State { $pstate } Napięcie(mV)
mem-pstate-clock-voltage = VRAM P-State { $pstate } Napięcie (mV)
pstates = Stany Zasilania
gpu-pstates = Stany Zasilania GPU
vram-pstates = Stany Zasilania VRAN
enable-pstate-config = Aktywuj konfiguracje stanów zasilania
show-historical-charts = Pokaż wykres
add-profile = Dodaj nowy profil
import-profile = Importuj profil z pliku
create-profile = Utwórz profil
name = Nazwa
profile-copy-from = Skopiuj ustawienia z:
create = Utwórz
cancel = Anuluj
default-profile = Domyślne
rename-profile = Zmień nazwę Profilu
delete-profile = Usuń Profil
edit-rules = Modyfikuj Reguły
edit-rule = Modyfikuj regułę
remove-rule = Usuń regułę
profile-rules = Profil Reguł
export-to-file = Wyeksportuj Do Pliku
move-up = Przesuń w górę
move-down = Przesuń w dół
profile-hooks = Zaczepy
profile-activation-desc = Aktywuj profil '{ $name }' kiedy:
any-rules-matched = Jeśli spełniony jest którykolwiek z poniższych warunków:
activation-settings-status =
    Wybrane aktywatory ustawień sa obecnie<b>{ $matched ->
        [true] zgodne
       *[false] niezgodne
    }</b>
activation-auto-switching-disabled = Automatyczna zmiana profili jest obecnie zablokowana
profile-hook-command = Uruchom komendę gdy profil '{ $cmd }' jest:
profile-hook-activated = Aktywowana:
profile-hook-note = Informacja: komendy te są wykonywane jako root poprzez LACT, i nie ma dostępu do środowiska graficznego. Dlatego nie mogą zostać wywołane bezpośrednio aby aktywować aplikacje graficzne.
profile-rule-process-tab = Proces jest uruchomiony
profile-rule-gamemode-tab = Try Gamemode jest aktywny
profile-rule-process-name = Nazwa Procesu:
profile-rule-args-contain = Argumenty zawierają:
profile-rule-specific-process = Z określonym procesem:
pmfw-reset-warning = UWAGA: To zresetuje ustawienia sterownika wentylatora!
pstate-list-description = <b>Widoczne wartości są zegarami z offsetem dla każdego P-State, pogrupowane od największych do najniższych.</b>
amd-oc-disabled =
    Zarządzanie ustawieniami AMD OC nie dostępne!
    W dalszym ciągu może dokonać zmian podstawowych, lecz zaawansowane ustawienia częstotliwości oraz energii nie będą dostępne.
enable-amd-oc-description = Ta czynność odblokuje zaawansowane ustawienie w sterowniku amdgpu poprzez utworzenie pliku w <b>{ $path }</b> oraz zaktualizowaniu initramfs. Czy jesteś tego pewien?
amd-oc-description =
    { $config ->
        [rpm-ostree] To ustawienie spowoduje dostępność Zarządzaniem AMD OC dzięki ustawieniu flagi startowej poprzez <b>rpm-ostree</b>.
        [unsupported]
            Obecny system nie jest rozpoznawany jako wspierany dla automatycznego wymuszania ustawień.
            Możliwa jest próba odblokowania zarządzania poprzez LACT, lecz może to wymagać manualnego wygenerowania initramfs.
            Jeżeli to zawiedzie, opcją awaryjną jest dodanie  <b>amdgpu.ppfeaturemask=0xffffffff</b> jako parametru startowego do twojego programu rozruchowego.
       *[other] Ta opcja zmieni dostępność Zarządzania poprzez tworzenie pliku w <b>{ $path }</b> oraz aktualizacji initramfs.
    }

    Sprawdź <a href="https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)">wiki</a> po więcej informacji.
oc-warning = Ostrzeżenie: zmieniając te ustawienia , może nastąpić niestabilność systemu które może prowadzić do uszkodzenia sprzętu!
mem-pstate-clock = VRAM P-State { $pstate } Zegar (MHz)
profile-activation = Aktywacja
show-process-monitor = Pokaż monitor procesu
apply-button = Zastosuj
revert-button = Cofnij
vram-usage = Użycie VRAM:
performance-level = Poziom Wydajności
spindown-delay-tooltip = Jak długo GPU musi utrzymać niską temperaturę aby obniżyć prędkość wentylatorów
hw-queues = Kolejki
temperature-sensor = Czujnik temperatury
spindown-delay = Opóźnienie zatrzymania (ms)
speed-change-threshold = Próg zmiany prędkości (°C)
automatic-mode-threshold = Próg trybu automatycznego (°C)
automatic-mode-threshold-tooltip =
    Przełącz sterowanie wentylatorem w tryb automatyczny, gdy temperatura spadnie poniżej tej wartości.

    Wiele kart graficznych Nvidia obsługuje zatrzymanie wentylatora tylko w trybie automatycznego sterowania, podczas gdy własna (niestandardowa) krzywa ma ograniczony zakres prędkości, np. 30–100%.

    Ta opcja pozwala obejść to ograniczenie, używając niestandardowej krzywej tylko powyżej określonej temperatury, a poniżej niej korzystając z wbudowanego trybu automatycznego karty, który obsługuje tryb zero RPM.
bytes = bajty
kibibyte = kibibajty
gibibyte = gibibajty
vf-curve-editor = Edytor krzywej VF
nvidia-vf-curve-warning =
    Edytor krzywej napięcia–częstotliwości opiera się na nieudokumentowanej funkcjonalności sterownika.
    Nie ma żadnych gwarancji dotyczących jego działania, bezpieczeństwa ani dostępności.
    <span weight = "heavy" underline = "single">Używasz na własne ryzyko</span>.
vf-curve-enable-editing = Włącz edycje
voltage = Napięcie
frequency = Częstotliwość
vf-active-curve = aktywna krzywa
vf-base-curve = krzywa bazowa
vf-curve-visible-range = Zakres widoczny (%):
vf-curve-visible-range-to = do
vf-curve-flatten-right = wyrównaj krzywą w prawo
generate-debug-snapshot = wygeneruj zrzut diagnostyczny
dump-vbios = wyeksportuj VBIOS
reset-all-config = Zresetuj całą konfiguracje
stats-update-interval = Interwał aktualizacji (ms)
historical-data-title = Dane historyczne
graphs-per-row = Wykresy w jednym rzędzie:
time-period-seconds = Okres czasu (sekundy):
reset-all-graphs-tooltip = przywróć wszystkie wykresy do ustawień domyślnych
add-graph = dodaj wykres
delete-graph = usuń wykres
edit-graphs = edytuj
export-csv = wyeksportuj jako CSV
edit-graph-sensors = edytuj czujniki wykresu
reconnecting-to-daemon = Utracono połączenie z usługą (daemon), ponowne łączenie...
daemon-connection-lost = Utracono połączenie
plot-show-detailed-info = pokaż szczegółowe informacje
theme = motyw
theme-auto = automatycznie
preferences = preferencje
daemon = Daemon
about = O
crash-page-title = Aplikacja uległa awarii
exit = Wyjście
hw-ip-info = Informacje o sprzętowym adresie IP
ui = Interfejs użytkownika
