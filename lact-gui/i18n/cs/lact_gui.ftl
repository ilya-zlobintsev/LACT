hardware-info = Informace o hardware
system-section = Systém
kernel-version = Verze jádra systému
device-name = Název zařízení
version = Verze
features = Funkce
show-button = Zobrazit
cache-data = Data
cache-instruction = Data
cache-cpu = Procesor
auto-page = Automatické
static-page = Statické
reset-button = Vrátit na výchozí
apply-button = Uplatnit
revert-button = Vrátit zpět
mebibyte = MiB
unknown-throttling = Neznámé
missing-stat = Neaplikovatelné
performance-level-auto = Automatické
performance-level-manual = Ručně
edit-graphs = Upravit
name = Název
create = Vytvořit
cancel = Zrušit
save = Uložit
default-profile = Výchozí
edit-rule = Upravit pravidlo
remove-rule = Odebrat pravidlo
profile-hook-activated = Aktivováno:
profile-hook-deactivated = Deaktivováno:
info-page = Informace
oc-page = Přetakt.
thermals-page = Teplotní
software-page = Software
lact-daemon = Proces služby LACT
lact-gui = Grafické uživ. rozhraní LACT
instance = Instance
compute-units = Výpočetních jednotek
platform-name = Název platformy
api-version = Verze API
driver-name = Název ovladače
driver-version = Verze ovladače
cl-c-version = Verze OpenCL C
workgroup-size = Velikost pracovní skupiny
global-memory = Globální paměť
local-memory = Lokální paměť
extensions = Rozšíření
device-not-found = { $kind } zařízení nenalezeno
cache-info = Informace o mezipaměti
amd-cache-desc =
    { $size } L{ $level } { $types } mezipaměť { $shared ->
        [1] lokální pro každou výpočetní jednotku
       *[other] sdílené mezi { $shared } výpočetními jednotkami
    }
nvidia-cache-desc = { $size } L{ $level }
monitoring-section = Dohledování
fan-control-section = Ovládání ventilátorů
temperatures = Teploty
oc-missing-fan-control-warning = Varování: podpora pro přetaktování je vypnutá – funkce řízení ventilátorů proto není k dispozici.
fan-speed = Otáčky ventilátoru
throttling = Přiškrcování
curve-page = Křivka
target-temp = Cílová teplota (°C)
acoustic-limit = Akustický limit (ot/min)
acoustic-target = Akustický cíl (ot/min)
min-fan-speed = Nejnižší přijatelné otáčky ventilátoru (%)
zero-rpm = Nula otáček
zero-rpm-stop-temp = Teplota (°C) pro zastavení na nula otáček
static-speed = Staticky nastavené otáčky (%)
pmfw-reset-warning = Varování: toto vrátí nastavení ventilátorů ve firmware na výchozí hodnoty!
temperature-sensor = Teplotní senzor
spindown-delay = Prodleva (ms) snížení otáček
spindown-delay-tooltip = Jak dlouho je třeba, aby GPU vydrželo na nižší teplotě než budou sníženy otáčky ventilátorů
speed-change-threshold = Práh (°C) změny otáček
automatic-mode-threshold = Práh (°C) pro automatický režim
amd-oc = Přetaktování AMD
disable-amd-oc = Vypnout přetaktování AMD
enable-amd-oc = Zapnout přetaktování AMD
disable-amd-oc-description = Toto při příštím startu systému vypne podporu pro přetaktování AMD (overdrive).
amd-oc-updating-configuration = Nastavení se aktualizuje (toto může chvíli trvat)
amd-oc-updating-done = Nastavení zaktualizováno – restartujte, aby se změny projevily.
reset-config = Vrátit nastavení na výchozí hodnoty
reset-config-description = Opravdu chcete vrátit veškerá nastavení GPU na výchozí hodnoty?
power-cap = Limit elektrického příkonu
watt = W
ghz = GHz
mhz = MHz
stats-section = Statistiky
gpu-clock = Takt jádra GPU
gpu-clock-avg = Takt jádra GPU (průměr)
gpu-clock-target = Takt jádra GPU (cíl)
gpu-voltage = Napětí GPU
gpu-temp = Teplota
gpu-usage = Vytížení GPU
vram-clock = Takt videopaměti
power-usage = Elektrický příkon
no-throttling = Ne
vram-usage = Využití videopaměti:
performance-level-high = Nejvyšší takty
performance-level-low = Nejnižší takty
performance-level-auto-description = Automaticky přizpůsobit takty GPU a videopaměti (výchozí).
performance-level-high-description = Vždy pro GPU a videopaměť používat nejvyšší takty.
performance-level-low-description = Pro GPU a videopaměť vždy používat nejnižší takty.
performance-level-manual-description = Ruční řízení výkonu.
performance-level = Výkonnostní stupeň
power-profile-mode = Režim profilu napájení:
manual-level-needed = Výkonnostní stupeň byl nastaven na „ručně“, aby bylo možné používat stavy a režimy napájení
overclock-section = Takt a napětí
nvidia-oc-info = Informace o přetaktování Nvidia
oc-warning = Změna těchto hodnot může vést k nestabilitě systému a případně poškodit váš hardware!
show-all-pstates = Zobrazit veškeré P-stavy
enable-gpu-locked-clocks = Povolit GPU uzamčené takty
enable-vram-locked-clocks = Povolit uzamčené takty videopaměti
no-clocks-data = Nejsou k dispozici žádné údaje o taktech
reset-oc-tooltip = Varování: toto vrátí veškerá nastavení taktů na výchozí hodnoty!
gpu-clock-offset = Posun taktu GPU (MHz)
max-gpu-clock = Nejvyšší takt GPU (MHz)
max-vram-clock = Nejvyšší takt videopaměti (MHz)
max-gpu-voltage = Nejvyšší napětí GPU (mV)
min-gpu-clock = Nejnižší takt GPU (MHz)
min-vram-clock = Nejnižší takt videopaměti (MHz)
min-gpu-voltage = Nejnižší napětí GPU (mV)
gpu-voltage-offset = Posun napětí GPU (mV)
gpu-pstate-clock-offset = Posun taktu (MHz) P-stavu GPU { $pstate }
vram-pstate-clock-offset = Posun taktu (MHz) P-stavu videopaměti { $pstate }
gpu-pstate-clock = Takt (MHz) GPU P-stavu { $pstate }
mem-pstate-clock = Takt (MHz) P-stavu videopaměti { $pstate }
gpu-pstate-clock-voltage = Napětí (mV) GPU P-stavu { $pstate }
mem-pstate-clock-voltage = Napětí (mV) P-stavu videopaměti { $pstate }
pstates = Stavy napájení
gpu-pstates = Stavy napájení GPU
vram-pstates = Stavy napájení videopaměti
pstates-manual-needed = Aby bylo možné přepínat stavy napájení, je třeba výkonnostní úroveň nastavit na „ručně“
enable-pstate-config = Povolit nastavování stavu napájení
show-historical-charts = Zobrazit historické grafy
show-process-monitor = Zobrazit monitor procesů
generate-debug-snapshot = Vytvořit ladící zachycený stav
dump-vbios = Pořídit výpis VBIOS firmware
reset-all-config = Vrátit veškerá nastavení na výchozí hodnoty
stats-update-interval = Interval aktualizace (ms)
historical-data-title = Historická data
graphs-per-row = Grafů na řádek:
time-period-seconds = Časové období (sekundy):
reset-all-graphs-tooltip = Vrátit veškeré grafy do výchozího stavu
add-graph = Přidat graf
delete-graph = Smazat graf
export-csv = Exportovat jako CSV
edit-graph-sensors = Upravit senzory grafu
reconnecting-to-daemon = Spojení s procesem služby ztraceno – znovupřipojování…
daemon-connection-lost = Spojení ztraceno
plot-show-detailed-info = Zobrazit podrobné informace
settings-profile = Profil nastavení
auto-switch-profiles = Přepínat automaticky
add-profile = Přidat nový profil
import-profile = Naimportovat profil ze souboru
create-profile = Vytvořit profil
profile-copy-from = Zkopírovat nastavení z:
rename-profile = Přejmenovat profil
rename-profile-from = Přejmenovat profil <b>{ $old_name }</b> na:
delete-profile = Smazat profil
edit-rules = Upravit pravidla
profile-rules = Pravidla profilu
export-to-file = Exportovat do souboru
move-up = Přesunout nahoru
move-down = Přesunout dolů
profile-activation = Aktivace
profile-hooks = Háčky
profile-activation-desc = Aktivovat profil „{ $name }“ když:
any-rules-matched = Jakékoli z následujících pravidel se shodují:
all-rules-matched = Všechny z následujících pravidel se shodují:
activation-auto-switching-disabled = Automatické přepínání profilů je v tuto chvíli vypnuté
profile-hook-command = Když je profil „{ $cmd }“, spustit příkaz:
profile-rule-process-tab = Proces je spuštěný
profile-rule-gamemode-tab = Herní režim je zapnutý
profile-rule-process-name = Název procesu:
profile-rule-args-contain = Argumenty obsahují:
profile-rule-specific-process = S konkrétním procesem:
automatic-mode-threshold-tooltip =
    Přepnout řízení ventilátorů na automatický režim, když teplota poklesne pod tento bod.

    Mnohá GPU od Nvidia podporují zastavení ventilátorů pouze v režimu automatického řízení ventilátorů, zatímco uživatelsky určená křivka má omezený rozsah otáček jako např. 30-100%.

    Tato předvolba umožňuje toto omezení obejít použitím uživatelsky určené křivky pouze při překročení určité teploty s tím, že pod ní je použit automatický režim vestavěný v kartě, který podporuje nulové otáčky.
amd-oc-disabled =
    Podpora pro přetaktování AMD není zapnutá.
    I tak je možné měnit základní nastavení, ale pokročilejší řízení taktů a napětí nebude k dispozici.
amd-oc-status =
    Přetaktování AMD je v tuto chvíli: <b>{ $status ->
        [true] Zapnuto
        [false] Vypnuto
       *[other] Neznámé
    }</b>
amd-oc-detected-system-config =
    Zjištěné nastavení systému: <b>{ $config ->
        [unsupported] Nepodporováno
       *[other] { $config }
    }</b>
amd-oc-description =
    { $config ->
        [rpm-ostree] Tato volba vypne/zapne podporu pro přetaktování AMD nastavení příznaků při zavádění systému prostřednictvím <b>rpm-ostree</b>.
        [unsupported]
            Stávající systém není rozpoznán jako podporovaný pro automatické nastavení možnosti přetaktování.
            Můžete se pokusit ho zapnout z LACT, ale může být zapotřebí ruční vyvolání znovuvytvoření initramfs, aby se změna projevila.
            Pokud se toto nezdaří, náhradní možností je přidat <b>amdgpu.ppfeaturemask=0xffffffff</b> jako parametr zavádění do vámi využívaného zavaděče operačního systému.
       *[other] Tato volba zapne podporu pro přetaktování AMD vytvořením souboru v <b>{ $path }</b> a zaktualizováním initramfs.
    }

    Další informace viz <a href="https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)">wiki</a>.
enable-amd-oc-description = Toto zapne funkci přetaktování ovladače amdgpu a to vytvořením souboru v <b>{ $path }</b> a zaktualizováním initramfs. Opravdu to chcete udělat?
pstate-list-description = <b>Následující hodnoty jsou posuny taktů pro každý P-stav – od nejvyššího po nejnižší.</b>
activation-settings-status =
    Vybrané nastavení aktivace je v tuto chvíli <b>{ $matched ->
        [true] odpovídá
       *[false] neodpovídá
    }</b>
profile-hook-note = Pozn.: tyto příkazy jsou vykonávány procesem služby LACT (jako root) a nemají proto přístup k desktopovému prostředí. Jako takové je tedy není možné použít přímo ke spouštění grafických aplikací.
nvidia-oc-description =
    Funkce přetaktování na Nvidia zahrnuje nastavení posunů pro takty GPU / videopaměti a omezení potenciálního rozsahu taktů pomocí funkce „uzamčené takty“.

    Na mnoha kartách ovlivní posun taktu videopaměti skutečný takt paměti pouze o polovinu hodnoty posunu.
    Například, posun +1000MHz videopaměti může zvýšit měřenou rychlost paměti pouze o 500MHz.
    Toto je normální a plyne z toho, jak Nvidia zachází s takty GDDR. Přizpůsobte tomu příslušně svá nastavení přetaktování.

    Přímé ovládání napětí není podporováno, protože v linuxovém ovladači od Nvidia není přítomno.

    Je možné dosáhnout svého druhu snížení napětí kombinací předvolby uzamčené takt a kladného posunu taktu.
    Toto GPU přinutí běžet na napětí, které je omezeno uzamčenými takty a přitom dosahovat vyšších taktů (díky posunu).
    Pokud nastaveno příliš vysoko, může toto ale způsobovat nestabilitu systému.
gibibyte = GiB
crash-page-title = Aplikace zhavarovala
exit = Ukončit
bytes = bajtů
kibibyte = KiB
