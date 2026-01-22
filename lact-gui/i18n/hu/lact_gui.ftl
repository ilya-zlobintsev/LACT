compute-units = Számítási Egységek (CU)
info-page = Információ
oc-page = Túlhajtás
thermals-page = Termálok
software-page = Szoftver
hardware-info = Hardver Információ
system-section = Rendszer
lact-daemon = LACT Daemon
lact-gui = LACT Felület
kernel-version = Kernel Változat
instance = Példány
device-name = Eszköz Név
platform-name = Felület Név
api-version = API Verzió
version = Változat
driver-name = Illesztőprogram Név
driver-version = Illesztőprogram Változat
cl-c-version = OpenCL C Változat
workgroup-size = Munkacsoport Méret
global-memory = Egyetemes Memória
local-memory = Helyi Memória
features = Jellemzők
extensions = Kiterjesztések
show-button = Mutasd
device-not-found = { $kind } eszköz megtalálása sikertelen volt
cache-info = Gyorsítótár Információ
amd-cache-desc =
    { $size } L{ $level } { $types } gyorsítótár { $shared ->
        [1] minden CU-hoz helyi
       *[other] megosztva { $shared } CU-kkal
    }
nvidia-cache-desc = { $size } L{ $level }
cache-data = Adat
cache-instruction = Adat
cache-cpu = CPU
monitoring-section = Megfigyelés
fan-control-section = Ventilátor Szabályozás
temperatures = Hőfokok
oc-missing-fan-control-warning = Figyelem: A túlhajtási támogatás ki van kapcsolva, a ventilátor szabályozás nem elérhető.
fan-speed = Ventilátor Sebesség
throttling = Fojtás
auto-page = Autómatikus
curve-page = Görbe
static-page = Statikus
target-temp = Hőmérséklet Célpont (°C)
acoustic-limit = Akusztikus Limit (RPM)
acoustic-target = Akusztikus Célpont (RPM)
min-fan-speed = Leglassabb Ventilátor Sebesség (%)
zero-rpm = Nulla RPM
zero-rpm-stop-temp = Nulla RPM Megállási Hőmérséklet (°C)
static-speed = Állandó Sebesség (%)
reset-button = Visszaállítás
pmfw-reset-warning = Figyelem: Ez eredeti állapotba helyezi a ventilátor firmverbeállításait!
temperature-sensor = Hőmérséklet Szenzor
spindown-delay = Lepörgés Késleltetés (ms)
spindown-delay-tooltip = Mennyi ideig kell a GPU-nak alacsony hőmérsékleten lennie mielőtt lelassítja a ventilátort
speed-change-threshold = Sebességváltoztatási Küszöb (°C)
automatic-mode-threshold = Autómatikus Mód Küszöb (°C)
automatic-mode-threshold-tooltip =
    Állítsd a ventilátor szabályozást autómatikusra amikor a hőmérséklet kevesebb ennél a pontnál.

    Sok Nvidia GPU csak az autómatikus módban tudja megállítani a ventilátort, és az egyedi görbének limitált sebességi kerete van, mint például 30-100%.

    Ez az opció megengedi hogy a limitációt átugord az eredeti görbe használatával amikor a hőmérséklet egy bizonyos mennyiség felett van, a kártyába épített autómatikus móddal ami támogatja a Nulla RPM mód használatát alatta.
amd-oc = AMD Túlhajtás
amd-oc-disabled =
    Az AMD Túlhajtási támogatás nincs bekapcsolva!
    Tudsz alap beállításokat változtatni, de a fejlettebb órajel és feszültség szabályozás nem elérhető.
amd-oc-status =
    Az AMD Túlhajtás jelenleg: <b>{ $status ->
        [true] Be van kapcsolva
        [false] Ki van kapcsolva
       *[other] Nem tudni
    }</b>
amd-oc-detected-system-config =
    Érzékelt rendszer beállítások: <b>{ $config ->
        [unsupported] Nem támogatott
       *[other] { $config }
    }</b>
amd-oc-description =
    { $config ->
        [rpm-ostree] Ez az opció ki és bekapcsolja az AMD túlhajtás támogatását a boot zászlók beállításával <b>rpm-ostree</b>.
        [unsupported]
            A jelenlegi rendszer autómatikus túlhajtási beállítás támogatása nincs felismerve.
            Megpróbálhatod bekapcsolni a túlhajtást a LACT-on keresztül, de manuális kezdeti RAM-alapú fájlrendszer regeneráció jelenlétére szükség lehet hogy működjön.
            Ha ez nem működik, tartalékként tedd a(z) <b>amdgpu.ppfeaturemask=0xffffffff</b> -t bootparaméterként a bootloader-be.
       *[other] Ez az opció bekapcsolja az AMD túlhajtás támogatását egy fájl készítésével a(z) <b>{ $path }</b> helyen és naprakészíti a kezdeti RAM-alapú fájlrendszert.
    }

    Lásd a <a href="https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)">wiki-t</a> több információért.
enable-amd-oc-description = Ez be fogja kapcsolni az amdgpu meghajtó túlhajtási jellemzőjét egy fájl készítésével a(z) <b>{ $path }</b> helyen és naprakészíti a kezdeti RAM-alapú fájlrendszert. Ezt biztos hogy akarod?
disable-amd-oc = AMD Túlhajtás Kikapcsolása
enable-amd-oc = AMD Túlhajtás Bekapcsolása
disable-amd-oc-description = Ez ki fogja kapcsolni az AMD túlhajtás támogatását a következő újraindításnál.
amd-oc-updating-configuration = Beállítás naprakészítése (ez eltarthat egy darabig)
amd-oc-updating-done = Beállítás naprakészítve, kérlek indítsd újra a számítógépet hogy a változások érvénybe lépjenek.
reset-config = Beállítás Alaphelyzetbe Állítása
reset-config-description = Biztos hogy alaphelyzetbe szeretnéd állítani az összes GPU beállítást?
apply-button = Alkalmaz
revert-button = Vissza
power-cap = Fogyasztási Korlát
watt = W
ghz = GHz
mhz = MHz
mebibyte = MiB
gibibyte = GiB
stats-section = Statisztikák
gpu-clock = GPU Mag Órajel
gpu-clock-avg = GPU Mag Órajel (Átlag)
gpu-clock-target = GPU Mag Órajel (Cél)
gpu-voltage = GPU Feszültség
gpu-temp = Hőfok
gpu-usage = GPU Kihasználtság
vram-clock = VRAM Órajel
power-usage = Fogyasztás
no-throttling = Nem
unknown-throttling = Nem tudni
missing-stat = Nincs adat
vram-usage = VRAM Kihasználtság:
performance-level-auto = Automatikus
performance-level-high = Legmagasabb Órajelek
performance-level-low = Legalacsonyabb Órajelek
performance-level-manual = Kézi
performance-level-auto-description = Automatikusan állított GPU és VRAM órajelek. (Alapértelmezett)
performance-level-high-description = Mindig a legmagasabb órajel gyorsaságot használja a GPU és VRAM-hoz.
performance-level-low-description = Mindig a legalacsonyabb órajel gyorsaságot használja a GPU és VRAM-hoz.
performance-level-manual-description = Kézi teljesítmény szabályozás.
performance-level = Teljesítmény Szint
power-profile-mode = Fogyasztási Profil Mód:
manual-level-needed = A teljesítmény szintjének muszáj "kézi" módon lennie hogy a teljesítmény állapot és módok használhatók legyenek
overclock-section = Órajel Gyorsaság és Feszültség
nvidia-oc-info = Nvidia Túlhajtás Információ
oc-warning = Figyelem: Az alábbi értékek megváltoztatása rendszer instabilitást okozhatnak és kárt tudnak okozni a hardverednek!
show-all-pstates = Mutasd az összes Teljesítmény Állapotot
enable-gpu-locked-clocks = GPU Zárt Órajelek Bekapcsolása
enable-vram-locked-clocks = VRAM Zárt Órajelek Bekapcsolása
pstate-list-description = <b>A következő értékek órajel-ellensúlyozások minden Teljesítmény Állapothoz, a legmagasabbtól a legkisebbig.</b>
no-clocks-data = Órajel adatok nem elérhetők
reset-oc-tooltip = Figyelem: Ez alaphelyzetre állítja az összes órajel-beállítást!
gpu-clock-offset = GPU Órajel Ellensúly (MHz)
max-gpu-clock = Legmagasabb GPU Órajel (MHz)
max-vram-clock = Legmagasabb VRAM Órajel (MHz)
max-gpu-voltage = Legmagasabb GPU Feszültség (mV)
min-gpu-clock = Legkisebb GPU Órajel (MHz)
min-vram-clock = Legkisebb VRAM Órajel (MHz)
min-gpu-voltage = Legkisebb GPU Feszültség (mV)
gpu-voltage-offset = GPU Feszültség Ellensúly (mV)
gpu-pstate-clock-offset = GPU Teljesítmény Állapot { $pstate } Órajel Ellensúly (MHz)
vram-pstate-clock-offset = VRAM Teljesítmény Állapot { $pstate } Órajel Ellensúly (MHz)
gpu-pstate-clock = GPU Teljesítmény Állapot { $pstate } Órajel (MHz)
mem-pstate-clock = VRAM Teljesítmény Állapot { $pstate } Órajel (MHz)
gpu-pstate-clock-voltage = GPU Teljesítmény Állapot { $pstate } Feszültség (mV)
mem-pstate-clock-voltage = VRAM Teljesítmény Állapot { $pstate } Feszültség (mV)
pstates = Teljesítmény Állapotok
gpu-pstates = GPU Teljesítmény Állapot
vram-pstates = VRAM Teljesítmény Állapot
pstates-manual-needed = Megjegyzés: A teljesítmény szintjének kötelezően "kézi" módon kell lennie hogy a teljesítmény állapot változtatható legyen
enable-pstate-config = Teljesítmény állapot beállítás bekapcsolása
show-historical-charts = Mutasd a Történelmi Diagramot
show-process-monitor = Mutasd a Folyamat Megfigyelőt
generate-debug-snapshot = Generálj Hibakereső Pillanatfelvételt
dump-vbios = Mentsd Ki a VBIOS-t
reset-all-config = Állítsd Alapértelmezettre az Összes Beállítást
stats-update-interval = Frissítési Intervallum (ms)
historical-data-title = Történelmi Adat
graphs-per-row = Soronkénti Grafikonok:
time-period-seconds = Idő Periódus (Másodpercek):
reset-all-graphs-tooltip = Állítsd Alapértelmezettre az Összes Grafikont
add-graph = Grafikon Hozzáadása
delete-graph = Grafikon Törlése
edit-graphs = Szerkesztés
export-csv = Exportáld CSV-ként
edit-graph-sensors = Grafikon Szenzor Szerkesztése
reconnecting-to-daemon = Daemon csatlakozás megszakadt, újracsatlakozás...
daemon-connection-lost = Csatlakozás Megszakadt
plot-show-detailed-info = Mutasd a részletes információt
settings-profile = Beállítás Profil
auto-switch-profiles = Válts autómatikusan
add-profile = Új profil létrehozása
import-profile = Profil importálása fájlból
create-profile = Profil Létrehozása
name = Név
profile-copy-from = Beállítások másolása innen:
create = Létrehozás
cancel = Megszakítás
save = Mentés
default-profile = Alapértelmezett
rename-profile = Profil Átnevezése
rename-profile-from = Profil <b>{ $old_name }</b> átnevezése erre:
delete-profile = Profil Törlése
edit-rules = Szabályok Szerkesztése
edit-rule = Szabály Szerkesztése
remove-rule = Szabály Törlése
profile-rules = Profil Szabályok
export-to-file = Exportálás Fájlba
move-up = Mozgás Fel
move-down = Mozgás Le
profile-activation = Aktiváció
profile-hooks = Kampók
profile-activation-desc = Profil '{ $name }' aktiválása amikor:
any-rules-matched = Amikor a következő szabályok akármelyike érvénybe lép:
all-rules-matched = Amikor a következő szabályok összese érvénybe lép:
activation-settings-status =
    A kiválasztott aktiváció beállítások éppen <b>{ $matched ->
        [true] érvényes
       *[false] nem érvényes
    }</b>
activation-auto-switching-disabled = Az autómatikus profil váltás ki van kapcsolva
profile-hook-command = Futtass egy parancsot amikor a '{ $cmd }' profil:
profile-hook-activated = Aktív:
profile-hook-deactivated = Nem aktív:
profile-hook-note = Megjegyzés: Ezek a parancsok rendszergazdaként vannak végrehajtva a LACT daemon által, és nincs hozzáférésük az asztali környezethet, ezért nem lehetnek használva arra hogy közvetlenül elindítsanak grafikus applikációkat.
profile-rule-process-tab = Egy folyamat fut
profile-rule-gamemode-tab = A játékmód aktív
profile-rule-process-name = Folyamat Név:
profile-rule-args-contain = Az Argumentumok Tartalmaznak:
profile-rule-specific-process = Egy specifikus folyamattal:
crash-page-title = Applikáció Összeomlott
exit = Kilépés
nvidia-oc-description =
    Az Nvidia kártyákon a túlhajtási funkcionalitás része a GPU/VRAM órajel ellensúlyozása és a pontenciális órajel gyorsaság keretének limitálása a "zárt órajelek" funkcióval.

    Sok kártyán a VRAM órajel gyorsaság ellensúlyozása a megadott érték felével lép érvénybe.
    Például a +1000MHz VRAM ellensúly lehet hogy a mért VRAM gyorsaságot csak 500MHz-el növeli.
    Ez normális, Nvidia így kezeli a GDDR adat rátákat, állítsd a túlhajtást ennek megfelelően.

    Közvetlen feszültség szabályozása nem támogatott, mivel az nem létezik az Nvidia Linux illesztőprogramban.

    Lehetséges elérni egy ál-alulfeszültséget zárt órajelek és pozitív órajel gyorsaság ellensúllyal.
    Ez erőlteti a GPU-t hogy olyan feszültséggel fusson ami korlátozott a zárt órajelekkel, míg elér egy magasabb órajel gyorsaságot az ellensúly miatt.
    Ha túl magasra van állítva, instabilitást tud okozni a rendszerben.
