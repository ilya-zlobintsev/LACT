info-page = Tiedot
oc-page = Ylikellotus
thermals-page = Lämmönhallintaan liittyvät näkökohdat
software-page = Ohjelmisto
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
amd-oc-disabled =
    AMD-ylikellotustukea ei ole otettu käyttöön!
    Voit edelleen muuttaa perusasetuksia, mutta edistyneemmät kellotaajuudet ja jännitteen säätö eivät ole käytettävissä.
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
reset-config-description = Oletko varma, että haluat nollata koko GPU:n kokoonpanon?
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
performance-level-high-description = Käytä aina GPU:n ja VRAM:n korkeimpia kellotaajuuksia.
performance-level-low-description = Käytä aina GPU:n ja VRAM:n alhaisimpia kellotaajuuksia.
performance-level-manual-description = Manuaalinen suorituskyvyn hallinta.
performance-level = Suorituskykytaso
power-profile-mode = Virtaprofiilitila:
manual-level-needed = Suorituskykytaso on asetettava "käsin":ksi, virrankäyttötilojen ja -tilojen käyttämiseksi
overclock-section = Kellotaajuus ja jännite
nvidia-oc-info = Nvidia-ylikellotustiedot
nvidia-oc-description =
    Nvidian ylikellotustoimintoihin kuuluu GPU/VRAM-kellotaajuuspoikkeamien asettaminen ja mahdollisen kellotaajuuksien alueen rajoittaminen "lukitut kellotaajuudet" -ominaisuuden avulla.

    Monilla korteilla VRAM-kellotaajuuspoikkeama vaikuttaa muistin todelliseen kellotaajuuteen vain puolella poikkeaman arvosta.
    Esimerkiksi +1000 MHz:n VRAM-poikkeama voi lisätä mitattua VRAM-nopeutta vain 500 MHz:llä.
    Tämä on normaalia, ja näin Nvidia käsittelee GDDR-tiedonsiirtonopeuksia. Säädä ylikellotusta vastaavasti.

    Suoraa jännitteen säätöä ei tueta, koska sitä ei ole Nvidian Linux-ajurissa.
    On mahdollista saavuttaa näennäisalijännite yhdistämällä lukittujen kellojen asetus positiiviseen kellotaajuuspoikkeamaan.
    Tämä pakottaa GPU:n toimimaan lukittujen kellojen rajoittamalla jännitteellä, samalla kun saavutetaan korkeampi kellotaajuus poikkeaman ansiosta.

    Tämä voi aiheuttaa järjestelmän epävakautta, jos sitä painetaan liian korkeaksi.
oc-warning = Varoitus: Näiden arvojen muuttaminen voi johtaa järjestelmän epävakauteen ja mahdollisesti vahingoittaa laitteistoasi!
show-all-pstates = Näytä kaikki P-tilat
enable-gpu-locked-clocks = Ota GPU:n lukitut kellotaajuudet käyttöön
enable-vram-locked-clocks = Ota VRAM:n lukitut kellotaajuudet käyttöön
pstate-list-description = <b>Seuraavat arvot ovat kellon siirtymiä kullekin P-tilalle korkeimmasta alhaisempaan.</b>
no-clocks-data = Ei kellotietoja saatavilla
reset-oc-tooltip = Varoitus: tämä palauttaa kaikki kellojen asetukset oletusasetuksiin!
pstates = Virtatilat
gpu-pstates = GPU:n virtatilat
vram-pstates = VRAM:n virtatilat
pstates-manual-needed = Huomautus: suorituskykytason on oltava asetettu 'käsin':ksi virrankäyttötilojen vaihtamiseksi
enable-pstate-config = Ota virtatilan määritys käyttöön
show-historical-charts = Näytä historialliset kaaviot
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
reconnecting-to-daemon = Yhteys daemoniin katkesi; yhdistetään uudelleen...
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
max-gpu-clock = GPU:n enimmäiskellotaajuus (MHz)
max-vram-clock = VRAM:n enimmäiskellotaajuus (MHz)
max-gpu-voltage = GPU:n enimmäisjännite (mV)
min-gpu-clock = GPU:n vähimmäiskellotaajuus (MHz)
min-vram-clock = VRAM:n vähimmäiskellotaajuus (MHz)
min-gpu-voltage = GPU:n vähimmäisjännite (mV)
gpu-clock-offset = GPU-kellotaajuuspoikkeama (MHz)
gpu-voltage-offset = GPU-jännitepoikkeama (mV)
gpu-pstate-clock-offset = VRAM:n P-tila { $pstate } kellotaajuuspoikkeama (MHz)
vram-pstate-clock-offset = VRAM:n P-tila { $pstate } kellotaajuuspoikkeama (MHz)
gpu-pstate-clock = GPU:n P-tila { $pstate } kellotaajuus (MHz)
mem-pstate-clock = VRAM:n P-tila { $pstate } kellotaajuus (MHz)
gpu-pstate-clock-voltage = GPU:n P-tila { $pstate } jännite (mV)
mem-pstate-clock-voltage = VRAM:n P-tila { $pstate } jännite (mV)
cache-data = Data
cache-instruction = Data
watt = W
ghz = GHz
mhz = MHz
nvidia-cache-desc = { $size } L{ $level }
gibibyte = GiB
crash-page-title = Ohjelma kaatui
exit = Poistu
