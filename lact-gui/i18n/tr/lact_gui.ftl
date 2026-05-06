info-page = Donanım Bilgisi
oc-page = Hız Aşırtma
thermals-page = Sıcaklıklar
software-page = Yazılım Bilgisi

hardware-info = Donanım Bilgisi

system-section = Sistem
lact-daemon = LACT Daemon
lact-gui = LACT GUI
kernel-version = Kernel Sürümü

instance = Instance
device-name = Cihaz Adı
platform-name = Platform Adı
api-version = API Sürümü
version = Sürüm
driver-name = Sürücü Adı
driver-version = Sürücü Sürümü
compute-units = Hesaplama Birimleri
cl-c-version = OpenCL C Sürümü
workgroup-size = Çalışma Grubu Boyutu
global-memory = Global Bellek
local-memory = Yerel Bellek
features = Özellikler
extensions = Uzantılar
show-button = Göster
device-not-found = {$kind} cihazı bulunamadı
cache-info = Önbellek Bilgisi
hw-ip-info = Donanım IP Bilgisi
hw-queues = Donanım Kuyrukları
amd-cache-desc = {$size} L{$level} {$types} önbellek { $shared ->
    [1] her CU için yerel
    *[other] {$shared} CU arasında paylaşılan
}
nvidia-cache-desc = {$size} L{$level}
cache-data = Veri
cache-instruction = Komut
cache-cpu = CPU

monitoring-section = İzleme
fan-control-section = Fan Kontrolü
temperatures = Sıcaklıklar
oc-missing-fan-control-warning = Uyarı: Hız aşırtma desteği devre dışı; fan kontrolü işlevi kullanılamıyor.
fan-speed = Fan Hızı
throttling = Performans Kısıtlaması
auto-page = Otomatik
curve-page = Fan Eğrisi
static-page = Sabit Hız
target-temp = Hedef Sıcaklık (°C)
acoustic-limit = Akustik Limit (RPM)
acoustic-target = Akustik Hedef (RPM)
min-fan-speed = Minimum Fan Hızı (%)
zero-rpm = Sıfır RPM
zero-rpm-stop-temp = Sıfır RPM Durma Sıcaklığı (°C)
static-speed = Sabit Hız (%)
reset-button = Sıfırla
pmfw-reset-warning = Uyarı: bu, fan firmware ayarlarını sıfırlar!

temperature-sensor = Sıcaklık Sensörü
spindown-delay = Fan Yavaşlama Gecikmesi (ms)
spindown-delay-tooltip = Fan hızını düşürmeden önce GPU'nun daha düşük bir sıcaklık değerinde ne kadar kalması gerektiği
speed-change-threshold = Hız Değişim Eşiği (°C)
automatic-mode-threshold = Otomatik Mod Eşiği (°C)
automatic-mode-threshold-tooltip = Sıcaklık bu noktanın altına düştüğünde fan kontrolünü otomatik moda geçir.

    Birçok Nvidia GPU, fanı yalnızca otomatik fan kontrol modunda durdurmayı destekler; özel fan eğrisi ise %30-%100 gibi sınırlı bir hız aralığına sahiptir.

    Bu seçenek, belirli bir sıcaklığın üzerinde özel fan eğrisini, altında ise sıfır RPM'i destekleyen kartın dahili otomatik modunu kullanarak bu sınırlamayı aşmayı sağlar.

amd-oc = AMD Hız Aşırtma
amd-oc-disabled = AMD Hız Aşırtma etkin değil! <a href="https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)">Bazı işlevler kullanılamayacak.</a>
amd-oc-status = AMD Hız Aşırtma şu anda: <b>{$status ->
    [true] Etkin
    [false] Devre dışı
    *[other] Bilinmiyor
}</b>
amd-oc-detected-system-config = Algılanan sistem yapılandırması: <b>{$config ->
    [unsupported] Desteklenmiyor
    *[other] {$config}
}</b>
amd-oc-description =
    {$config ->
        [rpm-ostree] Bu seçenek, <b>rpm-ostree</b> üzerinden önyükleme parametreleri ayarlayarak AMD Overdrive desteğini açıp kapatır.
        [unsupported]
            Mevcut sistem, otomatik Overdrive yapılandırması için desteklenen bir yapılandırma olarak tanınmadı.
            LACT üzerinden hız aşırtmayı etkinleştirmeyi deneyebilirsiniz, ancak etkili olması için initramfs'in elle yeniden oluşturulması gerekebilir.
            Bu da işe yaramazsa, önyükleyicinizde <b>amdgpu.ppfeaturemask=0xffffffff</b> değerini önyükleme parametresi olarak eklemek bir geri dönüş seçeneğidir.
        *[other] Bu seçenek, <b>{$path}</b> konumunda bir dosya oluşturarak ve initramfs'i güncelleyerek AMD Overdrive desteğini açıp kapatır.
    }

    Daha fazla bilgi için <a href="https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)">wiki</a> sayfasına bakın.
enable-amd-oc-description = Bu işlem, <b>{$path}</b> konumunda bir dosya oluşturarak ve initramfs'i güncelleyerek amdgpu sürücüsünün Overdrive özelliğini etkinleştirir. Bunu yapmak istediğinizden emin misiniz?
disable-amd-oc = AMD Hız Aşırtmayı Devre Dışı Bırak
enable-amd-oc = AMD Hız Aşırtmayı Etkinleştir
disable-amd-oc-description = Bir sonraki yeniden başlatmada AMD hız aşırtma desteğini (Overdrive) devre dışı bırakır.
amd-oc-updating-configuration = Yapılandırma güncelleniyor (bu biraz sürebilir)
amd-oc-updating-done = Yapılandırma güncellendi, değişiklikleri uygulamak için lütfen yeniden başlatın.

reset-config = Yapılandırmayı Sıfırla
reset-config-description = Tüm GPU yapılandırmasını sıfırlamak istediğinizden emin misiniz?

apply-button = Uygula
revert-button = Geri Al

power-cap = Güç Sınırı

watt = W
ghz = GHz
mhz = MHz
bytes = bayt
kibibyte = KiB
mebibyte = MiB
gibibyte = GiB

stats-section = İstatistikler
gpu-clock = GPU Çekirdek Saat Hızı
gpu-clock-avg = GPU Çekirdek Saat Hızı (Ortalama)
gpu-clock-target = GPU Çekirdek Saat Hızı (Hedef)
gpu-voltage = GPU Voltajı
gpu-temp = Sıcaklık
gpu-usage = GPU Kullanımı
vram-clock = VRAM Saat Hızı
power-usage = Güç Kullanımı
no-throttling = Hayır
unknown-throttling = Bilinmiyor
missing-stat = Yok
vram-usage = VRAM Kullanımı:

performance-level-auto = Otomatik
performance-level-high = En Yüksek Saat Hızları
performance-level-low = En Düşük Saat Hızları
performance-level-manual = Manuel
performance-level-auto-description = GPU ve VRAM saat hızlarını otomatik ayarla. (Varsayılan)
performance-level-high-description = GPU ve VRAM için her zaman en yüksek saat hızlarını kullan.
performance-level-low-description = GPU ve VRAM için her zaman en düşük saat hızlarını kullan.
performance-level-manual-description = Manuel performans kontrolü.

performance-level = Performans Seviyesi
power-profile-mode = Güç Profili Modu:
manual-level-needed = Güç durumlarını ve modlarını kullanmak için performans seviyesi "manuel" olarak ayarlanmalıdır

overclock-section = Saat Hızı ve Voltaj
nvidia-oc-info = Hız Aşırtma Bilgisi
nvidia-oc-description =
    Nvidia'da hız aşırtma işlevi, GPU/VRAM saat hızları için offsetler ayarlamayı ve "kilitli saat hızları" özelliğiyle saat hızlarının olası aralığını sınırlamayı içerir.

    Birçok kartta VRAM saat hızı offseti, gerçek bellek saat hızını offset değerinin yalnızca yarısı kadar etkiler.
    Örneğin, +1000 MHz VRAM offseti ölçülen VRAM hızını yalnızca 500 MHz artırabilir.
    Bu normaldir; Nvidia'nın GDDR veri hızlarını ele alış biçimi böyledir. Hız aşırtmanızı buna göre ayarlayın.

    Kilitli saat hızları seçeneğini pozitif bir saat hızı offsetiyle birleştirerek sözde undervolt elde etmek mümkündür.
    Bu, offset sayesinde daha yüksek bir saat hızı elde edilirken GPU'nun kilitli saat hızlarıyla sınırlandırılmış bir voltajda çalışmasını zorlar.
    Çok yüksek değerlerde sistem kararsızlığına neden olabilir.
oc-warning = Bu değerleri değiştirmek sistem kararsızlığına yol açabilir ve donanımınıza zarar verebilir!
show-all-pstates = Tüm P-State'leri Göster
enable-gpu-locked-clocks = GPU Kilitli Saat Hızlarını Etkinleştir
enable-vram-locked-clocks = VRAM Kilitli Saat Hızlarını Etkinleştir
pstate-list-description = <b>Aşağıdaki değerler, en yüksekten en düşüğe doğru her P-State için saat hızı offsetleridir.</b>
no-clocks-data = Saat verisi yok
reset-oc-tooltip = Uyarı: bu, tüm saat hızı ayarlarını varsayılanlara sıfırlar!
vf-curve-editor = VF Eğrisi Düzenleyici
nvidia-vf-curve-warning = Voltaj-frekans eğrisi düzenleyicisi belgelenmemiş sürücü işlevlerine dayanır.
    Davranışı, güvenliği veya kullanılabilirliği konusunda hiçbir garanti yoktur.
    <span weight = "heavy" underline = "single">Kendi riskinizle kullanın</span>.
vf-curve-enable-editing = Düzenlemeyi Etkinleştir
voltage = Voltaj
frequency = Frekans
vf-active-curve = Etkin Eğri
vf-base-curve = Temel Eğri
vf-curve-visible-range = Görünür Aralık (%):
vf-curve-visible-range-to = -
vf-curve-flatten-right = Eğriyi sağ tarafta düzleştir

gpu-clock-offset = GPU Saat Hızı Offseti (MHz)
max-gpu-clock = Maksimum GPU Saat Hızı (MHz)
max-vram-clock = Maksimum VRAM Saat Hızı (MHz)
max-gpu-voltage = Maksimum GPU Voltajı (mV)
min-gpu-clock = Minimum GPU Saat Hızı (MHz)
min-vram-clock = Minimum VRAM Saat Hızı (MHz)
min-gpu-voltage = Minimum GPU Voltajı (mV)
gpu-voltage-offset = GPU Voltaj Offseti (mV)
gpu-pstate-clock-offset = GPU P-State {$pstate} Saat Hızı Offseti (MHz)
vram-pstate-clock-offset = VRAM P-State {$pstate} Saat Hızı Offseti (MHz)
gpu-pstate-clock = GPU P-State {$pstate} Saat Hızı (MHz)
mem-pstate-clock = VRAM P-State {$pstate} Saat Hızı (MHz)
gpu-pstate-clock-voltage = GPU P-State {$pstate} Voltajı (mV)
mem-pstate-clock-voltage = VRAM P-State {$pstate} Voltajı (mV)

pstates = Güç Durumları
gpu-pstates = GPU Güç Durumları
vram-pstates = VRAM Güç Durumları
pstates-manual-needed = Güç durumlarını değiştirmek için performans seviyesi 'manuel' olmalıdır
enable-pstate-config = Güç durumu yapılandırmasını etkinleştir

show-historical-charts = Geçmiş Grafikleri Göster
show-process-monitor = İşlem İzleyicisini Göster
generate-debug-snapshot = Hata Ayıklama Anlık Görüntüsü Oluştur
dump-vbios = VBIOS Dökümü Al
reset-all-config = Tüm Yapılandırmayı Sıfırla
stats-update-interval = Güncelleme Aralığı (ms)

historical-data-title = Geçmiş Veriler
graphs-per-row = Satır Başına Grafik:
time-period-seconds = Zaman Aralığı (Saniye):
reset-all-graphs-tooltip = Tüm grafikleri varsayılan ayarlara döndür
add-graph = Grafik Ekle
delete-graph = Grafiği Sil
edit-graphs = Düzenle
export-csv = CSV Olarak Dışa Aktar
edit-graph-sensors = Grafikteki Sensörleri Düzenle

reconnecting-to-daemon = Daemon bağlantısı koptu, yeniden bağlanılıyor...
daemon-connection-lost = Daemon Bağlantısı Koptu

plot-show-detailed-info = Ayrıntılı Bilgiyi Göster

settings-profile = Ayar Profili
auto-switch-profiles = Profilleri Otomatik Değiştir
add-profile = Yeni Profil Ekle
import-profile = Profili Dosyadan İçe Aktar

create-profile = Profil Oluştur
name = Ad
profile-copy-from = Ayarları şuradan kopyala:
create = Oluştur
cancel = İptal
save = Kaydet
default-profile = Varsayılan
rename-profile = Profili Yeniden Adlandır
rename-profile-from = <b>{$old_name}</b> profilini şu adla yeniden adlandır:
delete-profile = Profili Sil
edit-rules = Kuralları Düzenle
edit-rule = Kuralı Düzenle
remove-rule = Kuralı Kaldır
profile-rules = Profil Kuralları
export-to-file = Dosyaya Dışa Aktar
move-up = Yukarı Taşı
move-down = Aşağı Taşı
profile-activation = Etkinleştirme
profile-hooks = Profil Hook'ları
profile-activation-desc = Profil '{$name}' şu durumda etkinleştir:
any-rules-matched = Aşağıdaki kurallardan herhangi biri eşleşirse:
all-rules-matched = Aşağıdaki kuralların tümü eşleşirse:
activation-settings-status = Seçili etkinleştirme ayarları şu anda <b>{ $matched ->
    [true] eşleşti
    *[false] eşleşmedi
}</b>
activation-auto-switching-disabled = Otomatik profil değiştirme şu anda devre dışı
profile-hook-command = Profil '{$cmd}' için şu durumda bir komut çalıştır:
profile-hook-activated = Etkinleştirildiğinde:
profile-hook-deactivated = Devre dışı bırakıldığında:
profile-hook-note = Not: bu komutlar LACT daemon'u tarafından root olarak çalıştırılır ve masaüstü ortamına erişimleri yoktur. Bu nedenle grafik arayüzlü uygulamaları doğrudan başlatmak için kullanılamazlar.

profile-rule-process-tab = Bir işlem çalışıyor
profile-rule-gamemode-tab = GameMode etkin
profile-rule-process-name = İşlem Adı:
profile-rule-args-contain = Argümanlar şunları içerir:
profile-rule-specific-process = Belirli bir işlemle:

theme = Tema
theme-auto = Otomatik
preferences = Tercihler
ui = Arayüz
daemon = Daemon
about = Hakkında

# Crash page
crash-page-title = Uygulama Çöktü
exit = Çık
