compute-units = Számítási egységek (CU)
info-page = Információ
oc-page = Túlhajtás
thermals-page = Hőfokok
software-page = Szoftver
hardware-info = Hardverinformációk
system-section = Rendszer
lact-daemon = LACT démon
lact-gui = LACT grafikus felület
kernel-version = Kernelverzió
instance = Példány
device-name = Eszköznév
platform-name = Platformnév
api-version = API-verzió
version = Verzió
driver-name = Illesztőprogram neve
driver-version = Illesztőprogram verziója
cl-c-version = OpenCL C verzió
workgroup-size = Munkacsoportméret
global-memory = Egyetemes memória
local-memory = Helyi memória
features = Jellemzők
extensions = Kiterjesztések
show-button = Megjelenítés
device-not-found = Nem található { $kind } eszköz
cache-info = Gyorsítótár-információk
amd-cache-desc =
    { $size } L{ $level } { $types }gyorsítótár { $shared ->
        [1] helyileg, minden egyes CU-hoz
       *[other] { $shared } CU-val megosztva
    }
nvidia-cache-desc = { $size } L{ $level }
cache-data = adat
cache-instruction = adat
cache-cpu = CPU-
monitoring-section = Megfigyelés
fan-control-section = Ventilátorszabályozás
temperatures = Hőfokok
oc-missing-fan-control-warning = Figyelem: A túlhajtási támogatás ki van kapcsolva, a ventilátorszabályozás nem érhető el.
fan-speed = Ventilátorsebesség
throttling = Fojtás
auto-page = Automatikus
curve-page = Görbe
static-page = Statikus
target-temp = Hőmérsékletcél (°C)
acoustic-limit = Akusztikus korlát (RPM)
acoustic-target = Akusztikus cél (RPM)
min-fan-speed = Leglassabb ventilátorsebesség (%)
zero-rpm = Nulla RPM
zero-rpm-stop-temp = Nulla RPM megállítási hőmérséklete (°C)
static-speed = Állandó sebesség (%)
reset-button = Visszaállítás
pmfw-reset-warning = Figyelem: ez alaphelyzetbe állítja a ventilátor firmwarebeállításait!
temperature-sensor = Hőmérséklet-érzékelő
spindown-delay = Lepörgési késleltetés (ms)
spindown-delay-tooltip = Mennyi ideig kell a GPU-nak alacsony hőmérsékletűnek lennie mielőtt lelassítja a ventilátort
speed-change-threshold = Sebességváltoztatási küszöb (°C)
automatic-mode-threshold = Automatikus mód küszöbe (°C)
automatic-mode-threshold-tooltip =
    A ventilátorszabályozás automatikusra állítása, amikor a hőmérséklet ennél alacsonyabb.

    Sok Nvidia GPU csak az automatikus módban tudja megállítani a ventilátort, míg az egyéni görbe sebességkerete korlátozott, például 30-100%.

    Ez a beállítás lehetővé teszi, hogy a megkerülje ezt a korlátozást azáltal, hogy egy adott hőmérséklet felett egyéni görbét használ, míg az alatt a kártyába épített automatikus módot használja, amely támogatja a nulla RPM használatát.
amd-oc = AMD Túlhajtás
amd-oc-disabled =
    Az AMD Túlhajtás támogatása nincs bekapcsolva!
    Az alapbeállításokat megváltoztathatja, de a fejlettebb órajel- és feszültségszabályozás nem érhető el.
amd-oc-status =
    Az AMD Túlhajtás jelenleg: <b>{ $status ->
        [true] Be van kapcsolva
        [false] Ki van kapcsolva
       *[other] Ismeretlen
    }</b>
amd-oc-detected-system-config =
    Érzékelt rendszerbeállítások: <b>{ $config ->
        [unsupported] Nem támogatott
       *[other] { $config }
    }</b>
amd-oc-description =
    { $config ->
        [rpm-ostree] Ez a lehetőség be- és kikapcsolja az AMD Túlhajtás támogatását az indítási jelzők <b>rpm-ostree</b> szolgáltatáson keresztüli beállításával.
        [unsupported]
            A jelenlegi rendszer automatikus túlhajtási beállításának támogatása nem ismerhető fel.
            Megpróbálhatja bekapcsolni a túlhajtást a LACT-on keresztül, de hogy ez érvénybe lépjen, lehet, hogy az initramfs újbóli kézi előállítása szükséges.
            Ha ez nem működik, tartalékként állítsa be az <b>amdgpu.ppfeaturemask=0xffffffff</b> rendszerindítási paramétert.
       *[other] Ez a lehetőség be- és kikapcsolja az AMD túlhajtás támogatását az <b>{ $path }</b> fájl elkészítésével, és az initramfs frissítésével.
    }

    További információkért lásd a <a href="https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)">wikit</a>.
enable-amd-oc-description = Ez be fogja kapcsolni az amdgpu illesztőprogram túlhajtási funkcióját egy fájl készítésével itt: <b>{ $path }</b>, és frissíti az initramfs-t. Biztos, hogy ezt akarja?
disable-amd-oc = AMD Túlhajtás kikapcsolása
enable-amd-oc = AMD Túlhajtás bekapcsolása
disable-amd-oc-description = Ez ki fogja kapcsolni az AMD túlhajtás támogatását a következő újraindításnál.
amd-oc-updating-configuration = Beállítások frissítése (ez eltarthat egy darabig)
amd-oc-updating-done = Beállítás frissítve, a változtatások érvénybe lépéséhez indítsa újra a számítógépet.
reset-config = Beállítások alaphelyzetbe állítása
reset-config-description = Biztos, hogy alaphelyzetbe állítja az összes GPU-beállítást?
apply-button = Alkalmaz
revert-button = Visszavonás
power-cap = Fogyasztási korlát
watt = W
ghz = GHz
mhz = MHz
mebibyte = MiB
gibibyte = GiB
stats-section = Statisztikák
gpu-clock = GPU-mag órajele
gpu-clock-avg = GPU-mag órajele (átlag)
gpu-clock-target = GPU-mag órajele (cél)
gpu-voltage = GPU feszültsége
gpu-temp = Hőfok
gpu-usage = GPU kihasználtsága
vram-clock = VRAM órajele
power-usage = Fogyasztás
no-throttling = Nem
unknown-throttling = Ismeretlen
missing-stat = Nincs adat
vram-usage = VRAM kihasználtsága:
performance-level-auto = Automatikus
performance-level-high = Legmagasabb órajelek
performance-level-low = Legalacsonyabb órajelek
performance-level-manual = Kézi
performance-level-auto-description = Automatikusan állított GPU és VRAM órajelek. (alapértelmezett)
performance-level-high-description = Mindig a legmagasabb órajel használata a GPU-hoz és a VRAM-hoz.
performance-level-low-description = Mindig a legalacsonyabb órajel használata a GPU-hoz és a VRAM-hoz.
performance-level-manual-description = Kézi teljesítményszabályozás.
performance-level = Teljesítményszint
power-profile-mode = Fogyasztási profilmód:
manual-level-needed = A teljesítményszintnek „kézi” módban kell lennie, hogy a teljesítmény-állapotok és -módok használhatók legyenek
overclock-section = Órajel és feszültség
nvidia-oc-info = Nvidia Túlhajtás információi
oc-warning = Ezen értékek megváltoztatása a rendszer instabilitást okozhatja, és akár kárt is tehet a hardverében!
show-all-pstates = Összes teljesítményállapot megjelenítése
enable-gpu-locked-clocks = Zárolt GPU órajelek bekapcsolása
enable-vram-locked-clocks = Zárolt VRAM órajelek bekapcsolása
pstate-list-description = <b>A következő értékek az egyes teljesítményállapotok órajeleltolásai, a legmagasabbtól a legalacsonyabbig.</b>
no-clocks-data = Az órajeladatok nem érhetőek el
reset-oc-tooltip = Figyelem: ez alaphelyzetbe állítja az összes órajel-beállítást!
gpu-clock-offset = GPU órajelléptetése (MHz)
max-gpu-clock = Legmagasabb GPU órajel (MHz)
max-vram-clock = Legmagasabb VRAM órajel (MHz)
max-gpu-voltage = Legmagasabb GPU feszültség (mV)
min-gpu-clock = Legalacsonyabb GPU órajel (MHz)
min-vram-clock = Legalacsonyabb VRAM órajel (MHz)
min-gpu-voltage = Legalacsonyabb GPU feszültség (mV)
gpu-voltage-offset = GPU feszültségléptetése (mV)
gpu-pstate-clock-offset = „{ $pstate }” GPU teljesítményállapot órajelléptetése (MHz)
vram-pstate-clock-offset = „{ $pstate }” VRAM teljesítményállapot órajeleltolása (MHz)
gpu-pstate-clock = „{ $pstate }” GPU teljesítményállapot órajele (MHz)
mem-pstate-clock = „{ $pstate }” VRAM teljesítményállapot órajele (MHz)
gpu-pstate-clock-voltage = „{ $pstate }” GPU teljesítményállapot feszültsége (mV)
mem-pstate-clock-voltage = „{ $pstate }” VRAM teljesítményállapot feszültsége (mV)
pstates = Teljesítményállapotok
gpu-pstates = GPU teljesítményállapotok
vram-pstates = VRAM teljesítményállapotok
pstates-manual-needed = A teljesítményszintnek „kézi” módban kell lennie, hogy a teljesítmény-állapotok kapcsolhatók legyenek
enable-pstate-config = Teljesítményállapot-beállítás bekapcsolása
show-historical-charts = Előzménydiagramok megjelenítése
show-process-monitor = Folyamatfigyelő megjelenítése
generate-debug-snapshot = Hibakeresési pillanatkép előállítása
dump-vbios = VBIOS kimentése
reset-all-config = Összes beállítás visszaállítása
stats-update-interval = Frissítési időköz (ms)
historical-data-title = Előzményadatok
graphs-per-row = Soronkénti grafikonok:
time-period-seconds = Időszak (másodpercben):
reset-all-graphs-tooltip = Összes grafikon alapértelmezettre állítása
add-graph = Grafikon hozzáadása
delete-graph = Grafikon törlése
edit-graphs = Szerkesztés
export-csv = Exportálás CSV-ként
edit-graph-sensors = Grafikonérzékelők szerkesztése
reconnecting-to-daemon = A démon kapcsolata megszakadt, újracsatlakozás…
daemon-connection-lost = A kapcsolat megszakadt
plot-show-detailed-info = Részletes információk megjelenítése
settings-profile = Beállításprofil
auto-switch-profiles = Váltás automatikusan
add-profile = Új profil hozzáadása
import-profile = Profil importálása fájlból
create-profile = Profil létrehozása
name = Név
profile-copy-from = Beállítások másolása innen:
create = Létrehozás
cancel = Megszakítás
save = Mentés
default-profile = Alapértelmezett
rename-profile = Profil átnevezése
rename-profile-from = A(z) <b>{ $old_name }</b> profil átnevezése erre:
delete-profile = Profil törlése
edit-rules = Szabályok szerkesztése
edit-rule = Szabály szerkesztése
remove-rule = Szabály eltávolítása
profile-rules = Profilszabályok
export-to-file = Exportálás fájlba
move-up = Fentebb helyezés
move-down = Lentebb helyezés
profile-activation = Aktiválás
profile-hooks = Eseménykezelők
profile-activation-desc = A(z) „{ $name }” profil aktiválása, ha:
any-rules-matched = A következő szabályok bármelyike teljesül:
all-rules-matched = A következő szabályok mindegyike teljesül:
activation-settings-status =
    A kiválasztott aktiválási beállítások jelenleg <b>{ $matched ->
        [true] teljesülnek
       *[false] nem teljesülnek
    }</b>
activation-auto-switching-disabled = Az automatikus profilváltás ki van kapcsolva
profile-hook-command = Parancs futtatása, amikor a(z) „{ $cmd }” profil:
profile-hook-activated = Aktiválódik:
profile-hook-deactivated = Deaktiválódik:
profile-hook-note = Megjegyzés: ezeket a parancsokat rendszergazdaként hajtja végre a LACT démon, és nincs hozzáférése az asztali környezethez, ezért közvetlenül nem használható arra, hogy grafikus alkalmazásokat indítson el.
profile-rule-process-tab = Egy folyamat fut
profile-rule-gamemode-tab = A játékmód aktív
profile-rule-process-name = Folyamat neve:
profile-rule-args-contain = Az argumentumok tartalmazzák:
profile-rule-specific-process = Egy konkrét folyamattal:
crash-page-title = Az alkalmazás összeomlott
exit = Kilépés
nvidia-oc-description =
    Az Nvidia kártyákon a túlhajtási funkcionalitás része a GPU/VRAM órajelének léptetése, és a potenciális órajelkeretének korlátozása a „órajelek zárolása” funkcióval.

    Sok kártyán a VRAM órajel eltolása csak a megadott érték felével lép érvénybe.
    Például a +1000 MHz-es VRAM-eltolás lehet, hogy a VRAM mért sebességét csak 500 MHz-cel növeli.
    Ez normális, az Nvidia így kezeli a GDDR adatsebességeket, ennek megfelelően állítsa be a túlhajtást.

    A közvetlen feszültségszabályozás nem támogatott, mivel az nem létezik az Nvidia linuxos illesztőprogramjában.

    A feszültségcsökkentéshez hasonló eredményt lehet elérni zárolt órajelekkel és pozitív órajelléptetéssel.
    Ez arra kényszeríti a GPU-t, hogy olyan feszültséggel fusson, melyet a zárolt órajelek korlátoznak, miközben a léptetés miatt magasabb órajelet ér el.
    Ha túl magasra van állítva, ez a rendszer instabilitását okozhatja.
hw-ip-info = Hardver IP információi
hw-queues = Várakozási sorok
bytes = bájt
kibibyte = KiB
theme = Téma
theme-auto = Automatikus
