info-page = Info Perangkat Keras
oc-page = Overclocking
thermals-page = Termal
software-page = Info Perangkat Lunak
hardware-info = Informasi Perangkat Keras
system-section = Sistem
lact-daemon = Daemon LACT
lact-gui = GUI LACT
kernel-version = Versi Kernel
instance = Instance
device-name = Nama Perangkat
platform-name = Nama Platform
api-version = Versi API
version = Versi
driver-name = Nama Driver
driver-version = Versi Driver
compute-units = Unit Komputasi
cl-c-version = Versi OpenCL C
workgroup-size = Ukuran Workgroup
global-memory = Memori Global
local-memory = Memori Lokal
features = Fitur
extensions = Ekstensi
show-button = Tampilkan
device-not-found = Perangkat { $kind } tidak ditemukan
cache-info = Informasi Cache
amd-cache-desc =
    { $size } L{ $level } { $types } cache { $shared ->
        [1] local to each CU
       *[other] shared between { $shared } CUs
    }
nvidia-cache-desc = { $size } L{ $level }
cache-data = Data
cache-instruction = Data
cache-cpu = CPU
monitoring-section = Pemantauan
fan-control-section = Kontrol Kipas
temperatures = Suhu
oc-missing-fan-control-warning = Peringatan: Dukungan overclocking dinonaktifkan, fungsi kontrol kipas tidak tersedia.
fan-speed = Kecepatan Kipas
throttling = Throttling
auto-page = Otomatis
curve-page = Kurva
static-page = Statis
target-temp = Suhu target (°C)
acoustic-limit = Batas Akustik (RPM)
acoustic-target = Target Akustik (RPM)
min-fan-speed = Kecepatan Kipas Minimum (%)
zero-rpm = Zero RPM
zero-rpm-stop-temp = Suhu berhenti Zero RPM (°C)
static-speed = Kecepatan Statis (%)
reset-button = Reset
pmfw-reset-warning = Peringatan: ini akan mereset pengaturan firmware kipas!
temperature-sensor = Sensor Suhu
spindown-delay = Penundaan Spindown (ms)
spindown-delay-tooltip = Berapa lama GPU perlu bertahan pada nilai suhu yang lebih rendah sebelum menurunkan kecepatan kipas
speed-change-threshold = Ambang Batas Perubahan Kecepatan (°C)
automatic-mode-threshold = Ambang Batas Mode Otomatis (°C)
automatic-mode-threshold-tooltip =
    Alihkan kontrol kipas ke mode otomatis ketika suhu berada di bawah titik ini.

    Banyak GPU Nvidia hanya mendukung penghentian kipas dalam mode kontrol kipas otomatis, sementara kurva kustom memiliki rentang kecepatan terbatas seperti 30-100%.

    Opsi ini memungkinkan untuk mengatasi keterbatasan ini dengan hanya menggunakan kurva kustom ketika suhu di atas nilai tertentu, dengan mode otomatis bawaan kartu yang mendukung Zero RPM digunakan di bawahnya.
amd-oc = Overclocking AMD
amd-oc-disabled = Overclocking AMD tidak diaktifkan! <a href="https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)">Beberapa fungsionalitas tidak akan tersedia.</a>
amd-oc-status =
    Overclocking AMD saat ini: <b>{ $status ->
        [true] Diaktifkan
        [false] Dinonaktifkan
       *[other] Tidak Diketahui
    }</b>
amd-oc-detected-system-config =
    Konfigurasi sistem yang terdeteksi: <b>{ $config ->
        [unsupported] Tidak Didukung
       *[other] { $config }
    }</b>
amd-oc-description =
    { $config ->
        [rpm-ostree] Opsi ini akan mengaktifkan/menonaktifkan dukungan AMD overdrive dengan mengatur boot flag melalui <b>rpm-ostree</b>.
        [unsupported]
            Sistem saat ini tidak dikenali sebagai yang didukung untuk konfigurasi overdrive otomatis.
            Anda dapat mencoba mengaktifkan overclocking dari LACT, tetapi regenerasi initramfs manual mungkin diperlukan agar dapat berlaku.
            Jika gagal, opsi alternatif adalah menambahkan <b>amdgpu.ppfeaturemask=0xffffffff</b> sebagai parameter boot di bootloader Anda.
       *[other] Opsi ini akan mengaktifkan/menonaktifkan dukungan AMD overdrive dengan membuat file di <b>{ $path }</b> dan memperbarui initramfs.
    }

    Lihat <a href="https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)">wiki</a> untuk informasi lebih lanjut.
enable-amd-oc-description = Ini akan mengaktifkan fitur overdrive driver amdgpu dengan membuat file di <b>{ $path }</b> dan memperbarui initramfs. Apakah Anda yakin ingin melakukan ini?
disable-amd-oc = Nonaktifkan Overclocking AMD
enable-amd-oc = Aktifkan Overclocking AMD
disable-amd-oc-description = Ini akan menonaktifkan dukungan overclocking AMD (overdrive) pada reboot berikutnya.
amd-oc-updating-configuration = Memperbarui konfigurasi (ini mungkin memakan waktu)
amd-oc-updating-done = Konfigurasi diperbarui, harap reboot untuk menerapkan perubahan.
reset-config = Reset Konfigurasi
reset-config-description = Apakah Anda yakin ingin mereset semua konfigurasi GPU?
apply-button = Terapkan
revert-button = Kembalikan
power-cap = Batas Penggunaan Daya
watt = W
ghz = GHz
mhz = MHz
bytes = byte
kibibyte = KiB
mebibyte = MiB
gibibyte = GiB
stats-section = Statistik
gpu-clock = Clock Inti GPU
gpu-clock-avg = Clock Inti GPU (Rata-rata)
gpu-clock-target = Clock Inti GPU (Target)
gpu-voltage = Tegangan GPU
gpu-temp = Suhu
gpu-usage = Penggunaan GPU
vram-clock = Clock VRAM
power-usage = Penggunaan Daya
no-throttling = Tidak
unknown-throttling = Tidak Diketahui
missing-stat = T/A
vram-usage = Penggunaan VRAM:
performance-level-auto = Otomatis
performance-level-high = Clock Tertinggi
performance-level-low = Clock Terendah
performance-level-manual = Manual
performance-level-auto-description = Sesuaikan clock GPU dan VRAM secara otomatis. (Default)
performance-level-high-description = Selalu gunakan kecepatan clock tertinggi untuk GPU dan VRAM.
performance-level-low-description = Selalu gunakan kecepatan clock terendah untuk GPU dan VRAM.
performance-level-manual-description = Kontrol performa manual.
performance-level = Level Performa
power-profile-mode = Mode Profil Daya:
manual-level-needed = Level performa harus diatur ke "manual" untuk menggunakan status daya dan mode
overclock-section = Kecepatan Clock dan Tegangan
nvidia-oc-info = Informasi Overclocking
nvidia-oc-description =
    Fungsionalitas overclocking pada Nvidia mencakup pengaturan offset untuk kecepatan clock GPU/VRAM dan pembatasan rentang potensial kecepatan clock menggunakan fitur "locked clocks".

    Pada banyak kartu, offset kecepatan clock VRAM hanya akan memengaruhi kecepatan memori aktual sebesar setengah dari nilai offset.
    Misalnya, offset VRAM +1000MHz mungkin hanya meningkatkan kecepatan VRAM yang terukur sebesar 500MHz.
    Ini normal, dan merupakan cara Nvidia menangani kecepatan data GDDR. Sesuaikan overclock Anda.

    Dimungkinkan untuk mencapai pseudo-undervolt dengan menggabungkan opsi locked clocks dengan offset kecepatan clock positif.
    Ini akan memaksa GPU berjalan pada tegangan yang dibatasi oleh locked clocks, sekaligus mencapai kecepatan clock yang lebih tinggi berkat offset tersebut.
    Hal ini dapat menyebabkan ketidakstabilan sistem jika didorong terlalu tinggi.
oc-warning = Mengubah nilai-nilai ini dapat menyebabkan ketidakstabilan sistem dan berpotensi merusak perangkat keras Anda!
show-all-pstates = Tampilkan Semua P-State
enable-gpu-locked-clocks = Aktifkan Locked Clocks GPU
enable-vram-locked-clocks = Aktifkan Locked Clocks VRAM
pstate-list-description = <b>Nilai-nilai berikut adalah offset clock untuk setiap P-State, dari tertinggi ke terendah.</b>
no-clocks-data = Tidak ada data clock yang tersedia
reset-oc-tooltip = Peringatan: ini akan mereset semua pengaturan clock ke default!
gpu-clock-offset = Offset Clock GPU (MHz)
max-gpu-clock = Clock GPU Maksimum (MHz)
max-vram-clock = Clock VRAM Maksimum (MHz)
max-gpu-voltage = Tegangan GPU Maksimum (mV)
min-gpu-clock = Clock GPU Minimum (MHz)
min-vram-clock = Clock VRAM Minimum (MHz)
min-gpu-voltage = Tegangan GPU Minimum (mV)
gpu-voltage-offset = Offset tegangan GPU (mV)
gpu-pstate-clock-offset = Offset Clock GPU P-State { $pstate } (MHz)
vram-pstate-clock-offset = Offset Clock VRAM P-State { $pstate } (MHz)
gpu-pstate-clock = Clock GPU P-State { $pstate } (MHz)
mem-pstate-clock = Clock VRAM P-State { $pstate } (MHz)
gpu-pstate-clock-voltage = Tegangan GPU P-State { $pstate } (mV)
mem-pstate-clock-voltage = Tegangan VRAM P-State { $pstate } (mV)
pstates = Status Daya
gpu-pstates = Status Daya GPU
vram-pstates = Status Daya VRAM
pstates-manual-needed = Level performa harus diatur ke 'manual' untuk mengaktifkan/menonaktifkan status daya
enable-pstate-config = Aktifkan konfigurasi status daya
show-historical-charts = Tampilkan Grafik
show-process-monitor = Tampilkan Monitor Proses
generate-debug-snapshot = Buat Snapshot Debug
dump-vbios = Dump VBIOS
reset-all-config = Reset Semua Konfigurasi
stats-update-interval = Interval Pembaruan (ms)
historical-data-title = Data Historis
graphs-per-row = Grafik Per Baris:
time-period-seconds = Periode Waktu (Detik):
reset-all-graphs-tooltip = Reset Semua Grafik ke Default
add-graph = Tambah Grafik
delete-graph = Hapus Grafik
edit-graphs = Edit
export-csv = Ekspor sebagai CSV
edit-graph-sensors = Edit Sensor Grafik
reconnecting-to-daemon = Koneksi daemon terputus, menyambung kembali...
daemon-connection-lost = Koneksi Terputus
plot-show-detailed-info = Tampilkan info terperinci
settings-profile = Profil Pengaturan
auto-switch-profiles = Alihkan secara otomatis
add-profile = Tambah profil baru
import-profile = Impor profil dari file
create-profile = Buat Profil
name = Nama
profile-copy-from = Salin pengaturan dari:
create = Buat
cancel = Batal
save = Simpan
default-profile = Default
rename-profile = Ganti Nama Profil
rename-profile-from = Ganti nama profil <b>{ $old_name }</b> menjadi:
delete-profile = Hapus Profil
edit-rules = Edit Aturan
edit-rule = Edit Aturan
remove-rule = Hapus Aturan
profile-rules = Aturan Profil
export-to-file = Ekspor ke File
move-up = Pindah ke Atas
move-down = Pindah ke Bawah
profile-activation = Aktivasi
profile-hooks = Hooks
profile-activation-desc = Aktifkan profil '{ $name }' ketika:
any-rules-matched = Salah satu aturan berikut terpenuhi:
all-rules-matched = Semua aturan berikut terpenuhi:
activation-settings-status =
    Pengaturan aktivasi yang dipilih saat ini <b>{ $matched ->
        [true] terpenuhi
       *[false] tidak terpenuhi
    }</b>
activation-auto-switching-disabled = Pergantian profil otomatis saat ini dinonaktifkan
profile-hook-command = Jalankan perintah ketika profil '{ $cmd }':
profile-hook-activated = Diaktifkan:
profile-hook-deactivated = Dinonaktifkan:
profile-hook-note = Catatan: perintah-perintah ini dijalankan sebagai root oleh daemon LACT, dan tidak memiliki akses ke lingkungan desktop. Oleh karena itu, perintah ini tidak dapat digunakan secara langsung untuk meluncurkan aplikasi grafis.
profile-rule-process-tab = Sebuah proses sedang berjalan
profile-rule-gamemode-tab = Gamemode aktif
profile-rule-process-name = Nama Proses:
profile-rule-args-contain = Argumen Mengandung:
profile-rule-specific-process = Dengan proses tertentu:
crash-page-title = Aplikasi Mengalami Crash
exit = Keluar
hw-ip-info = Informasi IP Perangkat Keras
hw-queues = Antrean
vf-curve-editor = Editor Kurva VF
nvidia-vf-curve-warning =
    Editor kurva tegangan-frekuensi mengandalkan fungsionalitas driver yang tidak terdokumentasi.
    Tidak ada jaminan mengenai perilaku, keamanan, atau ketersediaannya.
    <span weight = "heavy" underline = "single">Gunakan dengan risiko Anda sendiri</span>.
vf-curve-enable-editing = Aktifkan Pengeditan
voltage = Tegangan
frequency = Frekuensi
vf-active-curve = Kurva Aktif
vf-base-curve = Kurva Dasar
vf-curve-visible-range = Rentang Terlihat (%):
vf-curve-visible-range-to = hingga
vf-curve-flatten-right = Ratakan kurva ke kanan
theme = Tema
theme-auto = Otomatis
preferences = Preferensi
ui = UI
daemon = Daemon
about = Tentang
