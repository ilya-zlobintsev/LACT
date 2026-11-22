language = Valoda
language-system-default = Sistēmas noklusējums
language-restart-notice = Lai lietotu valodas izmaiņas, restartējiet LACT saskarni.
experimental-features = Eksperimentālās funkcijas
nvidia-pstate-offsets = NVIDIA nobīdes katram P stāvoklim
nvidia-pstate-offsets-description = Atklāt nobīdes stāvokļiem, kas nav P0. Pēc noklusējuma atspējots, jo neviena zināma ierīce/draivera versija to neatbalsta.
info-page = Aparatūras informācija
oc-page = Pārtaktēšana
thermals-page = Temperatūras
software-page = Programmatūras informācija
displays-page = Displeja informācija
detach-page = Atdalīt { $page }
page-detached = Šī lapa ir atvērta citā logā.
show-page-window = Fokusēt logu
reattach-page = Atgriezt galvenajā logā
hardware-info = Aparatūras informācija
system-section = Sistēma
lact-daemon = LACT dēmons
lact-gui = LACT saskarne
kernel-version = Kodola versija
instance = Instance
device-name = Ierīces nosaukums
platform-name = Platformas nosaukums
api-version = API versija
version = Versija
driver-name = Draivera nosaukums
driver-version = Draivera versija
compute-units = Skaitļošanas vienības
cl-c-version = OpenCL C versija
workgroup-size = Darba grupas izmērs
global-memory = Globālā atmiņa
local-memory = Lokālā atmiņa
features = Funkcijas
extensions = Paplašinājumi
show-button = Rādīt
device-not-found = { $kind } ierīce nav atrasta
cache-info = Kešatmiņas informācija
hw-ip-info = Aparatūras IP informācija
hw-queues = Rindas
amd-cache-desc =
    { $size } L{ $level } { $types } kešatmiņa { $shared ->
        [1] katram CU atsevišķa
       *[other] koplietota starp { $shared } CU
    }
nvidia-cache-desc = { $size } L{ $level }
cache-data = Dati
cache-instruction = Dati
cache-cpu = CPU
monitoring-section = Uzraudzība
thresholds-section = Sliekšņi un ierobežojumi
fan-control-section = Ventilatora vadība
temperatures = Temperatūras
oc-missing-fan-control-warning = Brīdinājums: pārtaktēšanas atbalsts ir atspējots, ventilatora vadības funkcionalitāte nav pieejama.
fan-speed = Ventilatora ātrums
no-fan-detected = Ventilators nav atrasts
no-sensors-found = Sensori nav atrasti
throttling = Droselēšana
auto-page = Automātiski
curve-page = Līkne
static-page = Statisks
target-temp = Mērķa temperatūra (°C)
acoustic-limit = Akustiskais ierobežojums (RPM)
acoustic-target = Akustiskais mērķis (RPM)
min-fan-speed = Minimālais ventilatora ātrums (%)
zero-rpm = Nulles RPM
zero-rpm-stop-temp = Nulles RPM apstāšanās temperatūra (°C)
static-speed = Statisks ātrums (%)
reset-button = Atiestatīt
reset-now-button = Atiestatīt tagad
default-button = Noklusējums
pmfw-reset-warning = Brīdinājums: tas atiestata ventilatora aparātprogrammatūras iestatījumus!
temperature-sensor = Temperatūras sensors
spindown-delay = Palēnināšanas aizture (ms)
spindown-delay-tooltip = Cik ilgi GPU jāpaliek zemākā temperatūrā, pirms ventilators palēnina darbību
speed-change-threshold = Ātruma maiņas slieksnis (°C)
automatic-mode-threshold = Automātiskā režīma slieksnis (°C)
automatic-mode-threshold-tooltip =
    Pārslēgt ventilatora vadību uz automātisko režīmu, kad temperatūra ir zem šī punkta.

    Daudzi Nvidia GPU atbalsta ventilatora apturēšanu tikai automātiskajā ventilatora vadības režīmā, savukārt pielāgotai līknei ir ierobežots ātruma diapazons, piemēram, 30–100%.

    Šī opcija ļauj apiet šo ierobežojumu, izmantojot pielāgoto līkni tikai virs noteiktas temperatūras, bet zem tās — kartes iebūvēto automātisko režīmu, kas atbalsta nulles RPM.
amd-oc = AMD pārtaktēšana
amd-oc-disabled = AMD pārtaktēšana nav iespējota! <a href="https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)">Daļa funkcionalitātes nebūs pieejama.</a>
amd-oc-status =
    AMD pārtaktēšana pašlaik ir: <b>{ $status ->
        [true] iespējota
        [false] atspējota
       *[other] nezināma
    }</b>
amd-oc-detected-system-config =
    Noteiktā sistēmas konfigurācija: <b>{ $config ->
        [unsupported] neatbalstīta
       *[other] { $config }
    }</b>
amd-oc-description =
    { $config ->
        [rpm-ostree] Šī opcija pārslēgs AMD overdrive atbalstu, iestatot sāknēšanas karogus ar <b>rpm-ostree</b>.
        [unsupported]
            Pašreizējā sistēma netiek atpazīta kā atbalstīta automātiskai overdrive konfigurēšanai.
            Varat mēģināt iespējot pārtaktēšanu no LACT, taču, lai tā stātos spēkā, var būt nepieciešama manuāla initramfs izveide no jauna.
            Ja tas neizdodas, rezerves variants ir pievienot <b>amdgpu.ppfeaturemask=0xffffffff</b> kā sāknēšanas parametru sāknētājā.
       *[other] Šī opcija pārslēgs AMD overdrive atbalstu, izveidojot failu <b>{ $path }</b> un atjauninot initramfs.
    }

    Sīkāku informāciju skatiet <a href="https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)">vikivietnē</a>.
enable-amd-oc-description = Tas iespējos amdgpu draivera overdrive funkciju, izveidojot failu <b>{ $path }</b> un atjauninot initramfs. Vai tiešām vēlaties to darīt?
disable-amd-oc = Atspējot AMD pārtaktēšanu
enable-amd-oc = Iespējot AMD pārtaktēšanu
disable-amd-oc-description = Tas nākamajā pārstartēšanā atspējos AMD pārtaktēšanas atbalstu (overdrive).
amd-oc-updating-configuration = Atjaunina konfigurāciju (tas var aizņemt kādu laiku)
amd-oc-updating-done = Konfigurācija atjaunināta; lai lietotu izmaiņas, pārstartējiet.
reset-config = Atiestatīt konfigurāciju
reset-config-description = Tas atiestatīs visus GPU iestatījumus uz noklusējumiem un neatgriezeniski izdzēsīs visus profilus
apply-button = Lietot
confirm = Apstiprināt
confirm-settings = Apstiprināt iestatījumus
revert-button = Atgriezt
settings-confirmation = Vai vēlaties paturēt jaunos iestatījumus? (Atgriešana pēc { $seconds_left } sekundēm)
power-cap = Jaudas ierobežojums
power-section = Jauda
watt = W
ghz = GHz
mhz = MHz
mv = mV
bytes = baiti
kibibyte = KiB
mebibyte = MiB
gibibyte = GiB
stats-section = Statistika
gpu-clock = GPU kodola takts
core-section = Kodols
gpu-clock-avg = GPU kodola takts (vidējā)
gpu-clock-target = GPU kodola takts (mērķa)
gpu-voltage = GPU spriegums
gpu-temp = Temperatūra
gpu-usage = GPU lietojums
vram-clock = VRAM takts
vram-section = VRAM
power-usage = Jaudas patēriņš
extra-clocks = Papildu taktis
no-throttling = Nē
unknown-throttling = Nezināms
missing-stat = Nav
vram-usage = VRAM lietojums:
gtt-usage = GTT lietojums:
uma-carveout = UMA atvēlējuma izmērs
uma-carveout-caption = Atmiņas apjoms, kas rezervēts kā atvēlēta VRAM. Lai stātos spēkā, nepieciešama pārstartēšana.
performance-level-auto = Automātiski
performance-level-high = Augstākās taktis
performance-level-low = Zemākās taktis
performance-level-manual = Manuāli
performance-level-profile-standard = Profilēšanas standarta
performance-level-profile-min-sclk = Profilēšana ar zemāko GPU takti
performance-level-profile-min-mclk = Profilēšana ar zemāko VRAM takti
performance-level-profile-peak = Profilēšanas maksimums
performance-level-auto-description = Automātiski pielāgot GPU un VRAM taktis. (Noklusējums)
performance-level-high-description = Vienmēr izmantot augstākās GPU un VRAM takts frekvences
performance-level-low-description = Vienmēr izmantot zemākās GPU un VRAM takts frekvences
performance-level-manual-description = Manuāla veiktspējas vadība
performance-level-profile-standard-description = Fiksēts profilēšanas režīms
performance-level-profile-min-sclk-description = Piespiež GPU takti zemākajā līmenī
performance-level-profile-min-mclk-description = Piespiež VRAM takti zemākajā līmenī
performance-level-profile-peak-description = Piespiež GPU un VRAM taktis augstākajos līmeņos
performance-level = Veiktspējas līmenis
power-profile-mode = Jaudas profila režīms
manual-level-needed = Lai izmantotu jaudas stāvokļus un režīmus, veiktspējas līmenim jābūt iestatītam uz „manuāli”
power-mizer-mode = PowerMizer režīms
power-mizer-mode-auto = Automātiski
power-mizer-mode-adaptive = Adaptīvs
power-mizer-mode-prefer-maximum-performance = Priekšroka maksimālai veiktspējai
power-mizer-mode-prefer-consistent-performance = Priekšroka nemainīgai veiktspējai
power-mizer-mode-auto-description = Ļaut draiverim izvēlēties veiktspējas politiku.
power-mizer-mode-adaptive-description = Pielāgot GPU taktis atkarībā no GPU lietojuma.
power-mizer-mode-prefer-maximum-performance-description = Dot priekšroku maksimālai veiktspējai draivera robežās.
power-mizer-mode-prefer-consistent-performance-description = Fiksēt GPU bāzes taktis.
overclock-section = Takts frekvence un spriegums
advanced-features = Papildu funkcijas
show-all-pstates = Visi P stāvokļi
enable-locked-clocks = Fiksētas taktis
enable-vf-curve = Pielāgota VF līkne
pstate-list-description = <b>Šīs vērtības ir katra P stāvokļa takts nobīdes, no augstākā līdz zemākajam.</b>
no-clocks-data = Takts dati nav pieejami
reset-oc-tooltip = Brīdinājums: tas atiestata visus takts iestatījumus uz noklusējumiem!
vf-curve-editor = VF līknes redaktors
nvidia-vf-curve-warning =
    Sprieguma-frekvences līknes redaktors balstās uz nedokumentētu draivera funkcionalitāti.
    <span weight = "heavy" underline = "single">Lietojiet uz savu atbildību</span>.
voltage = Spriegums
frequency = Frekvence
vf-active-curve = Aktīvā līkne
vf-base-curve = Bāzes līkne
vf-curve-visible-range = Redzamais diapazons (%):
vf-curve-visible-range-to = līdz
vf-curve-flatten-right = Izlīdzināt līkni pa labi
vf-curve-flatten-selection = Izlīdzināt atlasi
vf-curve-editing-disabled = VF līknes rediģēšana pārtaktēšanas lapā ir atspējota
gpu-clock-offset = Takts nobīde
max-clock = Maksimālā takts
max-gpu-voltage = Maksimālais spriegums
min-clock = Minimālā takts
min-gpu-voltage = Minimālais spriegums
gpu-voltage-offset = Sprieguma nobīde
gpu-voltage-boost = Sprieguma palielinājums
gpu-voltage-boost-tooltip = Nosaka, cik liela daļa no draivera definētās papildu sprieguma rezerves ir pieejama. 100% nozīmē visu šo rezervi, nevis 100% no kopējā GPU sprieguma. Lielāka rezerve var uzturēt augstākas takts frekvences, bet palielina jaudas patēriņu un siltumu.
pstate-clock-offset = P stāvokļa { $pstate } takts nobīde
pstate-clock = P stāvokļa { $pstate } takts
pstate-clock-voltage = P stāvokļa { $pstate } spriegums
pstate = Jaudas stāvoklis
pstates = Jaudas stāvokļi
gpu-pstates = GPU jaudas stāvokļi
vram-pstates = VRAM jaudas stāvokļi
pstates-manual-needed = Lai iespējotu P stāvokļu konfigurāciju, veiktspējas līmenim jābūt iestatītam uz „Manuāli”.
enable-pstate-config = P stāvokļu konfigurācija
menu = Izvēlne
show-historical-charts = Rādīt grafikus
show-process-monitor = Rādīt procesu uzraugu
generate-debug-snapshot = Izveidot atkļūdošanas momentuzņēmumu
dump-vbios = Izmest VBIOS
reset-all-config = Atiestatīt visu konfigurāciju
stats-update-interval = Atjaunināšanas intervāls (ms)
historical-data-title = Vēstures dati
graphs-per-row = Grafiki rindā:
time-period-seconds = Laika periods (sekundes):
reset-all-graphs-tooltip = Atiestatīt visus grafikus uz noklusējumu
add-graph = Pievienot grafiku
delete-graph = Dzēst grafiku
edit-graphs = Rediģēt
export-csv = Eksportēt kā CSV
edit-graph-sensors = Rediģēt grafika sensorus
error-heading = Kļūda
daemon-info-heading = Dēmona informācija
reconnecting-to-daemon = Savienojums ar pakalpojumu zaudēts, savienojas atkārtoti...
daemon-connection-lost = Savienojums zaudēts
service-explanation =
    GPU iestatījumu lietošanai nepieciešams LACT sistēmas pakalpojums.
    Bez tā LACT darbojas savrupā režīmā, kurā pieejama tikai informācija un uzraudzība.
service-setup-title = Pakalpojuma iestatīšana
setup-error = Iestatīšanas kļūda: { $error }
service-connection-status = Savienojuma statuss
service-status = Pakalpojuma statuss
service-permission-denied =
    Atļauja liegta: pakalpojums nav konfigurēts tā, lai atļautu jūsu lietotāja savienojumus.
    Sīkāku informāciju skatiet <a href="https://github.com/ilya-zlobintsev/lact#configuration">GitHub</a>
service-connected = savienots
service-disconnected = nav savienots
service-version = Pakalpojuma versija
gui-version = Saskarnes versija
service-version-mismatch = nesakrīt
service-logs = Pakalpojuma žurnāli
service-start = Palaist
service-stop = Apturēt
service-restart = Restartēt
service-autostart = Automātiski palaist sāknēšanas laikā
service-autostart-disable = Atspējot arī automātisko palaišanu
version-mismatch-description =
    Saskarnes un dēmona versijas nesakrīt ({ $gui_version }-{ $gui_commit } pret { $daemon_version }-{ $daemon_commit })!
    Ja LACT ir atjaunināts, pakalpojums jārestartē.
plot-show-detailed-info = Rādīt detalizētu informāciju
display-title = Displejs { $identifier }
display-manufacturer = Ražotājs
display-product-code = Produkta kods
display-model = Modelis
display-physical-size = Fiziskais izmērs
display-connection = Savienojums
display-manufacture-date = Ražošanas datums
displays-missing = Displeji nav atrasti
settings-profile = Iestatījumu profils
auto-switch-profiles = Pārslēgt automātiski
add-profile = Pievienot jaunu profilu
import-profile = Importēt profilu no faila
create-profile = Izveidot profilu
name = Nosaukums
profile-copy-from = Kopēt iestatījumus no:
create = Izveidot
cancel = Atcelt
close = Aizvērt
save = Saglabāt
default-profile = Noklusējums
rename-profile = Pārdēvēt profilu
rename-profile-from = Pārdēvēt profilu <b>{ $old_name }</b> par:
delete-profile = Dzēst profilu
edit-rules = Rediģēt kārtulas
edit-rule = Rediģēt kārtulu
remove-rule = Noņemt kārtulu
profile-rules = Profila kārtulas
export-to-file = Eksportēt failā
profile-activation = Aktivizēšana
profile-hooks = Āķi
profile-activation-desc = Aktivizēt profilu '{ $name }', kad:
any-rules-matched = Izpildās jebkura no šīm kārtulām:
all-rules-matched = Izpildās visas šīs kārtulas:
activation-settings-status =
    Atlasītie aktivizēšanas iestatījumi pašlaik <b>{ $matched ->
        [true] atbilst
       *[false] neatbilst
    }</b>
activation-auto-switching-disabled = Automātiskā profilu pārslēgšana pašlaik ir atspējota
profile-hook-command = Izpildīt komandu, kad profils '{ $cmd }' ir:
profile-hook-activated = Aktivizēts:
profile-hook-deactivated = Deaktivizēts:
profile-hook-note = Piezīme: šīs komandas izpilda LACT dēmons kā root, un tām nav piekļuves darbvirsmas videi. Tāpēc tās nevar tieši izmantot grafisku lietotņu palaišanai.
profile-rule-process-tab = Darbojas process
profile-rule-gamemode-tab = Gamemode ir aktīvs
profile-rule-process-name = Procesa nosaukums:
profile-rule-args-contain = Argumenti satur:
profile-rule-specific-process = Ar konkrētu procesu:
theme = Motīvs
theme-auto = Automātiski
preferences = Iestatījumi
ui = Saskarne
daemon = Dēmons
about = Par
color-scheme = Krāsu shēma
color-scheme-auto = Sistēmas
color-scheme-light = Gaiša
color-scheme-dark = Tumša
crash-page-title = Lietotne avarēja
exit = Iziet
