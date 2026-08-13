info-page = Donanım Bilgileri
oc-page = Hız Aşırtma
thermals-page = Sıcaklıklar
software-page = Yazılım Bilgileri
displays-page = Ekran Bilgileri
hardware-info = Donanım Bilgileri
system-section = Sistem
lact-daemon = LACT Daemon
lact-gui = LACT GUI
kernel-version = Çekirdek Sürümü
instance = Örnek
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
device-not-found = { $kind } cihazı bulunamadı
cache-info = Önbellek Bilgisi
hw-ip-info = Donanım IP Bilgisi
hw-queues = Donanım Kuyrukları
amd-cache-desc =
    { $size } L{ $level } { $types } önbellek { $shared ->
        [1] her CU'ya özel
       *[other] { $shared } CU arasında paylaşılan
    }
nvidia-cache-desc = { $size } L{ $level }
cache-data = Veri
cache-instruction = Komut
cache-cpu = CPU
monitoring-section = İzleme
thresholds-section = Eşikler &amp; Sınırlar
fan-control-section = Fan Kontrolü
temperatures = Sıcaklıklar
oc-missing-fan-control-warning = Uyarı: Hız aşırtma desteği devre dışı; fan kontrolü işlevi kullanılamıyor.
fan-speed = Fan Hızı
no-fan-detected = Fan algılanmadı
no-sensors-found = Sensör bulunamadı
throttling = Performans Kısıtlaması
auto-page = Otomatik
curve-page = Fan Eğrisi
static-page = Sabit Hız
target-temp = Hedef Sıcaklık (°C)
acoustic-limit = Akustik Sınır (RPM)
acoustic-target = Akustik Hedef (RPM)
min-fan-speed = Minimum Fan Hızı (%)
zero-rpm = Sıfır RPM
zero-rpm-stop-temp = Sıfır RPM'de Durma Sıcaklığı (°C)
static-speed = Sabit Hız (%)
reset-button = Sıfırla
reset-now-button = Şimdi Sıfırla
default-button = Varsayılan
pmfw-reset-warning = Uyarı: Bu işlem fanın firmware ayarlarını sıfırlar!
temperature-sensor = Sıcaklık Sensörü
spindown-delay = Devir Düşürme Gecikmesi (ms)
spindown-delay-tooltip = Fan hızını düşürmeden önce GPU'nun daha düşük bir sıcaklıkta ne kadar süre kalması gerektiği
speed-change-threshold = Hız Değişim Eşiği (°C)
automatic-mode-threshold = Otomatik Mod Eşiği (°C)
automatic-mode-threshold-tooltip =
    Sıcaklık bu noktanın altına düştüğünde fan kontrolünü otomatik moda geçir.

    Birçok NVIDIA GPU'su, fanı yalnızca otomatik fan kontrol modunda durdurmayı destekler; özel fan eğrisi ise %30-%100 gibi sınırlı bir hız aralığına sahiptir.

    Bu seçenek, belirli bir sıcaklığın üzerinde özel fan eğrisini, altında ise sıfır RPM'i destekleyen kartın dahili otomatik modunu kullanarak bu sınırlamayı aşmayı sağlar.
amd-oc = AMD Hız Aşırtma
amd-oc-disabled = AMD Hız Aşırtma etkin değil! <a href="https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)">Bazı işlevler kullanılamayacak.</a>
amd-oc-status =
    AMD Hız Aşırtma şu anda: <b>{ $status ->
        [true] Etkin
        [false] Devre dışı
       *[other] Bilinmiyor
    }</b>
amd-oc-detected-system-config =
    Algılanan sistem yapılandırması: <b>{ $config ->
        [unsupported] Desteklenmiyor
       *[other] { $config }
    }</b>
amd-oc-description =
    { $config ->
        [rpm-ostree] Bu seçenek, <b>rpm-ostree</b> üzerinden önyükleme parametreleri ayarlayarak AMD Overdrive desteğini açıp kapatır.
        [unsupported]
            Mevcut sistem, otomatik Overdrive yapılandırması için desteklenen bir yapılandırma olarak tanınmadı.
            LACT üzerinden hız aşırtmayı etkinleştirmeyi deneyebilirsiniz, ancak etkili olması için initramfs'in elle yeniden oluşturulması gerekebilir.
            Bu da işe yaramazsa, önyükleyicinizde <b>amdgpu.ppfeaturemask=0xffffffff</b> değerini önyükleme parametresi olarak eklemek bir geri dönüş seçeneğidir.
       *[other] Bu seçenek, <b>{ $path }</b> konumunda bir dosya oluşturarak ve initramfs'i güncelleyerek AMD Overdrive desteğini açıp kapatır.
    }

    Daha fazla bilgi için <a href="https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)">wiki</a> sayfasına bakın.
enable-amd-oc-description = Bu işlem, <b>{ $path }</b> konumunda bir dosya oluşturarak ve initramfs'i güncelleyerek amdgpu sürücüsünün Overdrive özelliğini etkinleştirir. Bunu yapmak istediğinizden emin misiniz?
disable-amd-oc = AMD Hız Aşırtmayı Devre Dışı Bırak
enable-amd-oc = AMD Hız Aşırtmayı Etkinleştir
disable-amd-oc-description = Bir sonraki yeniden başlatmada AMD hız aşırtma desteğini (Overdrive) devre dışı bırakır.
amd-oc-updating-configuration = Yapılandırma güncelleniyor (bu biraz sürebilir)
amd-oc-updating-done = Yapılandırma güncellendi, değişiklikleri uygulamak için lütfen yeniden başlatın.
reset-config = Ayarları Sıfırla
reset-config-description = Tüm GPU ayarları varsayılanlarına döndürülecek ve profiller kalıcı olarak silinecek. Devam etmek istediğinizden emin misiniz?
apply-button = Uygula
confirm = Onayla
confirm-settings = Ayarları Onayla
revert-button = Geri Al
settings-confirmation = Yeni ayarları korumak istiyor musunuz? ({ $seconds_left } saniye içinde geri alınacak)
power-cap = Güç Tüketimi Sınırı
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
no-throttling = Yok
unknown-throttling = Bilinmiyor
missing-stat = Yok
vram-usage = VRAM Kullanımı:
performance-level-auto = Otomatik
performance-level-high = En Yüksek Saat Hızları
performance-level-low = En Düşük Saat Hızları
performance-level-manual = Manuel
performance-level-auto-description = GPU ve VRAM saat hızlarını otomatik olarak ayarla. (Varsayılan)
performance-level-high-description = GPU ve VRAM için her zaman en yüksek saat hızlarını kullan.
performance-level-low-description = GPU ve VRAM için her zaman en düşük saat hızlarını kullan.
performance-level-manual-description = Manuel performans kontrolü.
performance-level = Performans Seviyesi
power-profile-mode = Güç Profili Modu:
manual-level-needed = Güç durumlarını ve modlarını kullanmak için performans seviyesi "manuel" olarak ayarlanmalıdır.
power-mizer-mode = PowerMizer Modu
power-mizer-mode-auto = Otomatik
power-mizer-mode-adaptive = Uyarlanabilir
power-mizer-mode-prefer-maximum-performance = En Yüksek Performansı Tercih Et
power-mizer-mode-prefer-consistent-performance = Tutarlı Performansı Tercih Et
power-mizer-mode-auto-description = Performans politikasını sürücünün seçmesine izin ver.
power-mizer-mode-adaptive-description = GPU kullanımına göre GPU saat hızlarını ayarla.
power-mizer-mode-prefer-maximum-performance-description = Sürücü sınırları içinde maksimum performansı tercih et.
power-mizer-mode-prefer-consistent-performance-description = GPU'nun temel saat hızlarını sabitle.
overclock-section = Saat Hızı ve Voltaj
nvidia-oc-info = Hız Aşırtma Bilgisi
nvidia-oc-description =
    NVIDIA'da hız aşırtma işlevi, GPU/VRAM saat hızları için ofsetler ayarlamayı ve "kilitli saat hızları" özelliğiyle saat hızlarının olası aralığını sınırlamayı içerir.

    Birçok kartta VRAM saat hızı ofseti, gerçek bellek saat hızını ofset değerinin yalnızca yarısı kadar etkiler.
    Örneğin, +1000 MHz VRAM ofseti ölçülen VRAM hızını yalnızca 500 MHz artırabilir.
    Bu normaldir; NVIDIA'nın GDDR veri hızlarını ele alış biçimi böyledir. Hız aşırtmanızı buna göre ayarlayın.

    Kilitli saat hızları seçeneğini pozitif bir saat hızı ofsetiyle birleştirerek sözde undervolt elde etmek mümkündür.
    Bu, ofset sayesinde daha yüksek bir saat hızı elde edilirken GPU'nun kilitli saat hızlarıyla sınırlandırılmış bir voltajda çalışmasını zorlar.
    Çok yüksek değerlerde sistem kararsızlığına neden olabilir.
oc-warning = Bu değerleri değiştirmek sistem kararsızlığına yol açabilir ve donanımınıza zarar verebilir!
show-all-pstates = Tüm P-State'leri Göster
enable-gpu-locked-clocks = GPU için Kilitli Saat Hızlarını Etkinleştir
enable-vram-locked-clocks = VRAM için Kilitli Saat Hızlarını Etkinleştir
pstate-list-description = <b>Aşağıdaki değerler, en yüksekten en düşüğe doğru her P-State için saat hızı ofsetleridir.</b>
no-clocks-data = Saat hızı verisi yok
reset-oc-tooltip = Uyarı: Bu işlem tüm saat hızı ayarlarını varsayılanlara sıfırlar!
vf-curve-editor = VF Eğrisi Düzenleyicisi
nvidia-vf-curve-warning =
    Voltaj-frekans eğrisi düzenleyicisi belgelenmemiş sürücü işlevlerine dayanır.
    Davranışı, güvenliği veya kullanılabilirliği konusunda hiçbir garanti yoktur.
    <span weight = "heavy" underline = "single">Kendi riskinizle kullanın</span>.
vf-curve-enable-editing = Düzenlemeyi Etkinleştir
voltage = Voltaj
frequency = Frekans
vf-active-curve = Etkin Eğri
vf-base-curve = Temel Eğri
vf-curve-visible-range = Görünür Aralık (%):
vf-curve-visible-range-to = ile
vf-curve-flatten-right = Eğriyi sağa doğru düzleştir
vf-curve-flatten-selection = Seçimi düzleştir
gpu-clock-offset = GPU Saat Hızı Ofseti (MHz)
max-gpu-clock = Maksimum GPU Saat Hızı (MHz)
max-vram-clock = Maksimum VRAM Saat Hızı (MHz)
max-gpu-voltage = Maksimum GPU Voltajı (mV)
min-gpu-clock = Minimum GPU Saat Hızı (MHz)
min-vram-clock = Minimum VRAM Saat Hızı (MHz)
min-gpu-voltage = Minimum GPU Voltajı (mV)
gpu-voltage-offset = GPU Voltaj Ofseti (mV)
gpu-voltage-boost = GPU Voltaj Takviyesi (%)
gpu-voltage-boost-tooltip = Sürücünün tanımladığı ek voltaj payının ne kadarının kullanılacağını belirler. %100, toplam GPU voltajının %100'ünü değil, bu ek payın tamamını ifade eder. Daha fazla voltaj payı daha yüksek saat hızlarının korunmasını sağlayabilir, ancak güç tüketimini ve sıcaklığı artırır.
gpu-pstate-clock-offset = GPU P-State { $pstate } Saat Hızı Ofseti (MHz)
vram-pstate-clock-offset = VRAM P-State { $pstate } Saat Hızı Ofseti (MHz)
gpu-pstate-clock = GPU P-State { $pstate } Saat Hızı (MHz)
mem-pstate-clock = VRAM P-State { $pstate } Saat Hızı (MHz)
gpu-pstate-clock-voltage = GPU P-State { $pstate } Voltajı (mV)
mem-pstate-clock-voltage = VRAM P-State { $pstate } Voltajı (mV)
pstates = Güç Durumları
gpu-pstates = GPU Güç Durumları
vram-pstates = VRAM Güç Durumları
pstates-manual-needed = Güç durumlarını değiştirmek için performans seviyesi 'manuel' olmalıdır.
enable-pstate-config = Güç durumu yapılandırmasını etkinleştir
menu = Menü
show-historical-charts = Geçmiş Grafikleri Göster
show-process-monitor = İşlem İzleyicisini Göster
generate-debug-snapshot = Hata Ayıklama Anlık Görüntüsü Oluştur
dump-vbios = VBIOS Dökümünü Al
reset-all-config = Tüm Ayarları Sıfırla
stats-update-interval = Güncelleme Aralığı (ms)
historical-data-title = Geçmiş Veriler
graphs-per-row = Satır Başına Grafik Sayısı:
time-period-seconds = Zaman Aralığı (saniye):
reset-all-graphs-tooltip = Tüm grafikleri varsayılan ayarlara döndür
add-graph = Grafik Ekle
delete-graph = Grafiği Sil
edit-graphs = Düzenle
export-csv = CSV Olarak Dışa Aktar
edit-graph-sensors = Grafikteki Sensörleri Düzenle
gtt-usage = GTT Kullanımı:
error-heading = Hata
daemon-info-heading = Daemon Bilgileri
reconnecting-to-daemon = Daemon bağlantısı koptu, yeniden bağlanılıyor...
daemon-connection-lost = Daemon Bağlantısı Koptu
service-explanation =
    GPU ayarlarını uygulamak LACT sistem hizmetini gerektirir.
    Bu hizmet olmadan LACT, yalnızca bilgi ve izleme özelliklerinin kullanılabildiği bağımsız modda çalışır.
service-connection-status = Bağlantı Durumu
service-status = Hizmet Durumu
service-permission-denied =
    İzin reddedildi; hizmet, kullanıcınızın bağlantı kurmasına izin verecek şekilde yapılandırılmamış.
    Daha fazla bilgi için <a href="https://github.com/ilya-zlobintsev/lact#configuration">GitHub</a> sayfasına bakın.
service-connected = bağlı
service-disconnected = bağlı değil
service-version = Hizmet Sürümü
gui-version = GUI Sürümü
service-version-mismatch = eşleşmiyor
service-logs = Hizmet Günlükleri
service-start = Başlat
service-stop = Durdur
service-restart = Yeniden Başlat
service-autostart = Açılışta otomatik başlat
service-autostart-disable = Otomatik başlatmayı da devre dışı bırak
version-mismatch-description =
    GUI ve daemon sürümleri eşleşmiyor ({$gui_version}-{$gui_commit} ile {$daemon_version}-{$daemon_commit})!
    LACT'yi güncellediyseniz hizmeti yeniden başlatmanız gerekir.
plot-show-detailed-info = Ayrıntılı Bilgiyi Göster
display-title = Ekran {$identifier}
display-manufacturer = Üretici
display-product-code = Ürün Kodu
display-model = Model
display-physical-size = Fiziksel Boyut
display-connection = Bağlantı
display-manufacture-date = Üretim Tarihi
displays-missing = Hiçbir ekran algılanmadı
settings-profile = Ayar Profili
auto-switch-profiles = Profilleri Otomatik Değiştir
add-profile = Yeni Profil Ekle
import-profile = Profili Dosyadan İçe Aktar
create-profile = Profil Oluştur
name = Ad
profile-copy-from = Ayarları şuradan kopyala:
create = Oluştur
cancel = İptal
close = Kapat
save = Kaydet
default-profile = Varsayılan
rename-profile = Profili Yeniden Adlandır
rename-profile-from = <b>{ $old_name }</b> profilini şu adla yeniden adlandır:
delete-profile = Profili Sil
edit-rules = Kuralları Düzenle
edit-rule = Kuralı Düzenle
remove-rule = Kuralı Kaldır
profile-rules = Profil Kuralları
export-to-file = Dosyaya Dışa Aktar
move-up = Yukarı Taşı
move-down = Aşağı Taşı
profile-activation = Etkinleştirme
profile-hooks = Profil Kancaları
profile-activation-desc = Profil '{ $name }' şu durumda etkinleştirilsin:
any-rules-matched = Aşağıdaki kurallardan herhangi biri eşleşirse:
all-rules-matched = Aşağıdaki kuralların tümü eşleşirse:
activation-settings-status =
    Seçili etkinleştirme ayarları şu anda <b>{ $matched ->
        [true] eşleşti
       *[false] eşleşmedi
    }</b>
activation-auto-switching-disabled = Otomatik profil değiştirme şu anda devre dışı
profile-hook-command = Profil '{ $cmd }' şu durumdayken komut çalıştır:
profile-hook-activated = Etkinleştirildiğinde:
profile-hook-deactivated = Devre dışı bırakıldığında:
profile-hook-note = Not: Bu komutlar LACT daemon'ı tarafından root kullanıcısı olarak çalıştırılır ve masaüstü ortamına erişimleri yoktur. Bu nedenle grafik arayüzlü uygulamaları doğrudan başlatmak için kullanılamazlar.
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
color-scheme = Renk Şeması
color-scheme-auto = Sistem
color-scheme-light = Açık
color-scheme-dark = Koyu
# Crash page
crash-page-title = Uygulama Çöktü
exit = Çık
