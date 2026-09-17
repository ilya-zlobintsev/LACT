info-page = Laitteistotiedot
oc-page = Ylikellotus
thermals-page = Lämmönhallintaan liittyvät näkökohdat
software-page = Ohjelmistotiedot
hardware-info = Laitteistotiedot
system-section = Järjestelmä
lact-daemon = LACT-daemon
lact-gui = LACT -graafinen käyttöliittymä
kernel-version = Kernel-versio
instance = Instanssi
compute-units = Laskentayksiköt
device-name = Laitteen nimi
platform-name = Alustan nimi
api-version = Sovellusliittymän versio
version = Versio
driver-name = Ajurin nimi
driver-version = Ajurin versio
cl-c-version = OpenCL C -versio
workgroup-size = Työryhmän koko
global-memory = Yleisesti pätevä muisti
local-memory = Paikallinen muisti
features = Ominaisuudet
extensions = Laajennukset
show-button = Näytä
device-not-found = Laitetta { $kind } ei löytynyt
cache-info = Välimuistin tiedot
amd-cache-desc =
    { $size } L{ $level } { $types } välimuisti { $shared ->
        [1] paikallinen kullekin CU:lle
       *[other] jaettu { $shared } CU:iden kesken
    }
cache-cpu = Suoritin
monitoring-section = Valvonta
fan-control-section = Tuulettimien hallinta
temperatures = Lämpötilat
oc-missing-fan-control-warning = Varoitus: Ylikellotustuki on poistettu käytöstä, tuulettimien hallintatoiminto ei ole käytettävissä.
fan-speed = Tuulettimien nopeus
auto-page = Automaattinen
curve-page = Käyrä
static-page = Staattinen
target-temp = Tavoitelämpötila (°C)
acoustic-limit = Akustinen raja (kier/min)
acoustic-target = Akustinen tavoite (kier/min)
min-fan-speed = Tuulettimien vähimmäisnopeus (%)
zero-rpm = Nolla kier/min
zero-rpm-stop-temp = Nolla kier/min pysäytyslämpötila (°C)
static-speed = Staattinen nopeus (%)
reset-button = Nollaa
pmfw-reset-warning = Varoitus: tämä nollaa tuulettimien laiteohjelmiston asetukset!
temperature-sensor = Lämpötila-anturi
spindown-delay = Alaspyörimisviive (ms)
spindown-delay-tooltip = Kuinka kauan GPU:n lämpötilan on pysyttävä alhaisemmassa arvossa ennen tuulettimien alasajoa
speed-change-threshold = Nopeudenmuutoskynnys (°C)
automatic-mode-threshold = Automaattitilan kynnys (°C)
automatic-mode-threshold-tooltip =
    Vaihda tuulettimien hallinta automaattitilaan, kun lämpötila on tämän arvon alapuolella.

    Monet Nvidian GPU:t tukevat tuulettimien pysäyttämistä vain automaattisessa tuulettimien hallintatilassa, kun taas mukautetulla käyrällä on rajoitettu nopeusalue, kuten 30–100 %.

    Tämän asetuksen avulla voidaan kiertää tämä rajoitus käyttämällä mukautettua käyrää vain tietyn lämpötilan yläpuolella, kortin sisäänrakennetulla automaattitilalla, joka tukee nollan kierroksen minuutissa käyttöä sen alapuolella.
amd-oc = AMD-ylikellotus
amd-oc-disabled = AMD:n ylikellotus ei ole käytössä! <a href="https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)">Jotkin toiminnot eivät ole käytettävissä.</a>
amd-oc-status =
    AMD-ylikellotus on tällä hetkellä: <b>{ $status ->
        [true] otettu käyttöön
        [false] Poistettu käytöstä
       *[other] Tuntematon
    }</b>
apply-button = Toteuta
revert-button = Palauta
mebibyte = Mit
unknown-throttling = Tuntematon
missing-stat = Ei saatavilla
performance-level-auto = Automaattinen
edit-graphs = Muokkaa
name = Nimi
create = Luo
cancel = Peruuta
save = Tallenna
default-profile = Oletus
edit-rule = Muokkaa sääntöä
remove-rule = Poista sääntö
profile-hook-activated = Käytössä:
profile-hook-deactivated = Ei käytössä:
performance-level-manual = Käsin
amd-oc-detected-system-config =
    Havaittu järjestelmäkokoonpano: <b>{ $config ->
        [unsupported] Ei tuettu
       *[other] { $config }
    }</b>
amd-oc-description =
    { $config ->
        [rpm-ostree] Tämä vaihtoehto ottaa käyttöön AMD-yliohjaamistuen asettamalla käynnistysliput <b>rpm-ostree</b>-parametrin kautta.
        [unsupported]
            Nykyistä järjestelmää ei tunnisteta tuetuksi automaattiselle ylikierron määritykselle.
            Voit yrittää ottaa ylikellotuksen käyttöön LACT:sta, mutta sen voimaantulo saattaa vaatia manuaalisen initramfs-uudelleenluonnin.
            Jos tämä epäonnistuu, varavaihtoehtona on lisätä <b>amdgpu.ppfeaturemask=0xffffffff</b> käynnistysparametriksi käynnistyslataimeen.
       *[other] Tämä vaihtoehto ottaa AMD-yliohjaamistuen käyttöön luomalla tiedoston osoitteeseen <b>{ $path }</b> ja päivittämällä initramfs:n.
    }

    Lisätietoja on <a href="https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)">Wikissä</a>.
enable-amd-oc-description = Tämä ottaa amdgpu-ajurin yliajo-ominaisuuden käyttöön luomalla tiedoston kohteeseen <b>{ $path }</b> ja päivittämällä initramfs:n. Oletko varma, että haluat tehdä tämän?
disable-amd-oc = Poista AMD-ylikellotus käytöstä
enable-amd-oc = Ota AMD-ylikellotus käyttöön
disable-amd-oc-description = Tämä poistaa AMD-ylikellotustuen (yliajo) käytöstä seuraavan uudelleenkäynnistyksen yhteydessä.
amd-oc-updating-configuration = Päivitetään kokoonpanoa (tämä voi kestää jonkin aikaa)
amd-oc-updating-done = Kokoonpano päivitetty. Ota muutokset käyttöön käynnistämällä uudelleen.
reset-config = Nollaa kokoonpano
reset-config-description = Tämä palauttaa kaikki näytönohjaimen asetukset oletusarvoihin ja poistaa kaikki profiilit pysyvästi
power-cap = Virrankulutusrajoitus
stats-section = Tilastot
gpu-clock = GPU-ytimen kellotaajuus
gpu-clock-avg = GPU-ytimen kellotaajuus (keskimääräinen)
gpu-clock-target = GPU-ytimen kellotaajuus (tavoite)
gpu-voltage = GPU-jännite
gpu-temp = Lämpötila
gpu-usage = GPU-käyttö
vram-clock = VRAM:n kellotaajuus
power-usage = Virrankulutus
no-throttling = Ei
vram-usage = VRAM:n käyttö:
performance-level-high = Korkeimmat kellotaajuudet
performance-level-low = Alhaisimmat kellotaajuudet
performance-level-auto-description = Säädä automaattisesti GPU:n ja VRAM:n kellotaajuuksia. (Oletus)
performance-level-high-description = Käytä aina korkeimpia mahdollisia kellotaajuuksia GPU:lle ja VRAM:ille
performance-level-low-description = Käytä aina alhaisimpia kellotaajuuksia GPU:lle ja VRAM:ille
performance-level-manual-description = Manuaalinen suorituskyvyn hallinta
performance-level = Suorituskykytaso
power-profile-mode = Virtaprofiilitila:
manual-level-needed = Suorituskykytaso on asetettava "käsin":ksi, virrankäyttötilojen ja -tilojen käyttämiseksi
overclock-section = Kellotaajuus ja jännite
show-all-pstates = Kaikki P-tilat
pstate-list-description = <b>Seuraavat arvot ovat kellon siirtymiä kullekin P-tilalle korkeimmasta alhaisempaan.</b>
no-clocks-data = Ei kellotietoja saatavilla
reset-oc-tooltip = Varoitus: tämä palauttaa kaikki kellojen asetukset oletusasetuksiin!
pstates = Virtatilat
gpu-pstates = GPU:n virtatilat
vram-pstates = VRAM:n virtatilat
pstates-manual-needed = P-tilan määritysten ottaminen käyttöön edellyttää, että suorituskykytasoksi asetetaan Manuaalinen:ksi.
enable-pstate-config = P-tilan määritys
show-historical-charts = Näytä kaaviot
show-process-monitor = Näytä prosessien valvonta
generate-debug-snapshot = Luo viankorjaustilannevedos
reset-all-config = Nollaa koko kokoonpano
stats-update-interval = Päivitysväli (ms)
historical-data-title = Historiallinen data
graphs-per-row = Kaavioita riviä kohden:
time-period-seconds = Aikajakso (sekuntia):
reset-all-graphs-tooltip = Palauta kaikki kaaviot oletusasetuksiin
add-graph = Lisää kaavio
delete-graph = Poista kaavio
export-csv = Vie CSV:na
edit-graph-sensors = Muokkaa kaavioiden antureita
reconnecting-to-daemon = Yhteys palveluun katkesi; yhdistetään uudelleen...
daemon-connection-lost = Yhteys katkennut
plot-show-detailed-info = Näytä yksityiskohtaiset tiedot
settings-profile = Asetusprofiili
auto-switch-profiles = Vaihda automaattisesti
add-profile = Lisää uusi profiili
import-profile = Tuo profiili tiedostosta
create-profile = Luo profiili
profile-copy-from = Kopioi asetukset kohteesta:
rename-profile = Nimeä profiili uudelleen
rename-profile-from = Nimeä profiili <b>{ $old_name }</b> uudelleen täksi:
delete-profile = Poista profiili
edit-rules = Muokkaa sääntöjä
profile-rules = Profiilin säännöt
export-to-file = Vie tiedostoon
move-up = Siirrä ylöspäin
move-down = Siirry alaspäin
profile-activation = Aktivointi
profile-hooks = Koukut
profile-activation-desc = Aktivoi profiili '{ $name }', kun:
any-rules-matched = Mikä tahansa seuraavista säännöistä täsmää:
all-rules-matched = Kaikki seuraavat säännöt täsmäävät:
dump-vbios = Luo vedos VBIOS:sta
activation-settings-status =
    Valitut aktivointiasetukset ovat tällä hetkellä <b>{ $matched ->
        [true] täsmäävät
       *[false] ei täsmää
    }</b>
activation-auto-switching-disabled = Automaattinen profiilinvaihto on tällä hetkellä pois käytöstä
profile-hook-command = Suorita komento, kun profiili '{ $cmd }' on:
profile-hook-note = Huomautus: LACT-daemon suorittaa nämä komennot rootina, eikä niillä ole pääsyä työpöytäympäristöön. Sellaisenaan niitä ei voida käyttää suoraan graafisten sovellusten käynnistämiseen.
profile-rule-process-tab = Prosessi on käynnissä
profile-rule-gamemode-tab = Pelitila on päällä
profile-rule-process-name = Prosessin nimi:
profile-rule-args-contain = Argumentit sisältävät:
profile-rule-specific-process = Tietyn prosessin kanssa:
throttling = Ylikuumenemisen estotoimi
max-gpu-voltage = Enimmäisjännite (mV)
min-gpu-voltage = Vähimmäisjännite (mV)
gpu-clock-offset = Kellotaajuuspoikkeama (MHz)
gpu-voltage-offset = Jännitepoikkeama (mV)
cache-data = Data
cache-instruction = Data
watt = W
ghz = GHz
mhz = MHz
nvidia-cache-desc = { $size } L{ $level }
gibibyte = GiB
crash-page-title = Ohjelma kaatui
exit = Poistu
bytes = tavua
kibibyte = Kit
theme-auto = Automaattinen
hw-ip-info = Laitteiston IP-tiedot
hw-queues = Jonot
theme = Teema
vf-curve-editor = VF-käyrän muokkain
nvidia-vf-curve-warning =
    Jännite-taajuus-käyrän muokkain hyödyntää ajurin dokumentoimatonta toiminnallisuutta.
    <span weight = "heavy" underline = "single">Käytä omalla vastuulla</span>.
voltage = Jännite
frequency = Taajuus
vf-active-curve = Aktiivinen käyrä
vf-base-curve = Peruskäyrä
vf-curve-visible-range = Näkyvä alue (%):
vf-curve-visible-range-to = asti
vf-curve-flatten-right = Litistä käyrää oikealle
preferences = Asetukset
about = Tietoja
confirm = Vahvista
confirm-settings = Vahvista asetukset
settings-confirmation = Haluatko säilyttää uudet asetukset? (Palautetaan { $seconds_left } sekunnin kuluttua)
menu = Valikko
error-heading = Virhe
daemon-info-heading = Daemonin tiedot
version-mismatch-description =
    Käyttöliittymän ja daemonin välinen versioristiriita ({ $gui_version }-{ $gui_commit } vastaan { $daemon_version }-{ $daemon_commit })!
    Jos olet päivittänyt LACT:n, sinun on käynnistettävä palvelu uudelleen.
close = Sulje
ui = Käyttöliittymä
daemon = Daemoni
displays-page = Näytä info
no-fan-detected = Tuuletinta ei havaittu
no-sensors-found = Antureita ei löytynyt
reset-now-button = Nollaa nyt
default-button = Oletus
power-section = Virta
core-section = Ydin
vram-section = VRAM
extra-clocks = Lisäkellotaajuukset
gtt-usage = GTT:n käyttö:
performance-level-profile-standard = Profilointistandardi
performance-level-profile-min-sclk = Näytönohjaimen alimman kellotaajuuden profilointi
thresholds-section = Kynnysarvot ja rajat
performance-level-profile-min-mclk = VRAM:n alimman kellotaajuuden profilointi
performance-level-profile-peak = Profilointihuippu
performance-level-profile-min-sclk-description = Pakottaa GPU:n kellotaajuuden alimmalle tasolle
performance-level-profile-min-mclk-description = Pakottaa VRAM:n kellotaajuuden alimmalle tasolle
performance-level-profile-peak-description = Pakottaa GPU:n ja VRAM:n kellotaajuudet korkeimmille tasoille
power-mizer-mode = PowerMizer-tila
power-mizer-mode-auto = Automaattinen
power-mizer-mode-adaptive = Mukautuva
power-mizer-mode-prefer-maximum-performance = Suosi parasta suorituskykyä
power-mizer-mode-prefer-consistent-performance = Suosi tasaista suorituskykyä
power-mizer-mode-auto-description = Anna ajurin valita suorituskykykäytäntö.
power-mizer-mode-adaptive-description = Säädä GPU:n kellotaajuuksia GPU:n käytön perusteella.
power-mizer-mode-prefer-maximum-performance-description = Suosi maksimisuorituskykyä ajurin rajojen puitteissa.
power-mizer-mode-prefer-consistent-performance-description = Lukitse GPU:n peruskellotaajuuksiin.
advanced-features = Edistyneet ominaisuudet
enable-locked-clocks = Lukitut kellotaajuudet
enable-vf-curve = Mukautettu jännite-taajuuskäyrä
vf-curve-flatten-selection = Litistä valinta
vf-curve-editing-disabled = Jännite-taajuuskäyrän muokkaus on poistettu käytöstä OC-sivulla
max-clock = Enimmäiskellotaajuus (MHz)
min-clock = Vähimmäiskellotaajuus (MHz)
gpu-voltage-boost = Jännitteen nosto (%)
gpu-voltage-boost-tooltip = Määrittää, kuinka suuri osa ajurin määrittelemästä ylimääräisestä jännitevarasta on käytettävissä. 100 % tarkoittaa koko tätä jännitevaraa, ei 100 %:a näytönohjaimen kokonaisjännitteestä. Suurempi jännitevara voi mahdollistaa korkeammat kellotaajuudet, mutta lisää virrankulutusta ja lämmöntuottoa.
pstate-clock-offset = P-tilan { $pstate } kellotaajuuden poikkeama (MHz)
pstate-clock = P-tila { $pstate } – kellotaajuus (MHz)
pstate-clock-voltage = P-tilan { $pstate } jännite (mV)
service-explanation =
    GPU-asetusten käyttäminen edellyttää LACT-järjestelmäpalvelua.
    Ilman sitä, LACT toimii itsenäisessä tilassa, jossa saatavilla ovat vain tiedot ja seuranta.
service-setup-title = Palvelun käyttöönotto
performance-level-profile-standard-description = Kiinteä profilointitila
setup-error = Käyttöönottovirhe: { $error }
service-connection-status = Yhteyden tila
service-status = Palvelun tila
service-permission-denied =
    Käyttö evätty; palvelua ei ole määritetty sallimaan yhteyksiä käyttäjältäsi.
    Lisätietoja <a href="https://github.com/ilya-zlobintsev/lact#configuration">GitHubissa</a>
service-connected = yhdistetty
service-disconnected = ei yhdistetty
service-version = Palveluversio
gui-version = GUI -versio
service-version-mismatch = yhteensopimaton
service-logs = Palvelulokit
service-start = Käynnistä
service-stop = Pysäytä
service-restart = Käynnistä uudelleen
service-autostart = Suorita automaattisesti käynnistyksen yhteydessä
service-autostart-disable = Poista myös automaattinen käynnistys käytöstä
display-title = Näyttö { $identifier }
display-manufacturer = Valmistaja
display-product-code = Tuotekoodi
display-model = Malli
display-physical-size = Fyysinen koko
display-connection = Yhteys
display-manufacture-date = Valmistuspäivämäärä
displays-missing = Näyttöjä ei havaittu
color-scheme = Väriteema
color-scheme-auto = Järjestelmä
color-scheme-light = Vaalea
color-scheme-dark = Tumma
