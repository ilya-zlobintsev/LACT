info-page = ຂໍ້ມູນຮາດແວ
oc-page = ໂອເວີຄລັອກ
thermals-page = ອຸນຫະພູມ
software-page = ຂໍ້ມູນຊອບແວ
hardware-info = ຂໍ້ມູນຮາດແວ
system-section = ລະບົບ
lact-daemon = ເດມອນ LACT
lact-gui = LACT GUI
kernel-version = ເວີຊັນເຄີເນວ
instance = ອິນສະແຕນ
device-name = ຊື່ອຸປະກອນ
platform-name = ຊື່ແພລດຟອມ
api-version = ເວີຊັນ API
version = ເວີຊັນ
driver-name = ຊື່ໄດຣເວີ
driver-version = ເວີຊັນໄດຣເວີ
compute-units = ໜ່ວຍປະມວນຜົນ
cl-c-version = ເວີຊັນ OpenCL C
workgroup-size = ຂະໜາດເວີກກຣຸບ
global-memory = ໜ່ວຍຄວາມຈຳຫຼັກ
local-memory = ໜ່ວຍຄວາມຈຳສະເພາະທີ່
features = ຄຸນສົມບັດ
extensions = ສ່ວນຂະຫຍາຍ
show-button = ສະແດງ
device-not-found = ບໍ່ພົບອຸປະກອນ { $kind }
cache-info = ຂໍ້ມູນແຄຊ
hw-ip-info = ຂໍ້ມູນ IP ຮາດແວ
hw-queues = ຄິວ
amd-cache-desc =
    { $size } L{ $level } { $types } ແຄຊ { $shared ->
        [1] ສະເພາະແຕ່ລະ CU
       *[other] ແບ່ງປັນລະຫວ່າງ { $shared } CUs
    }
nvidia-cache-desc = { $size } L{ $level }
cache-data = ຂໍ້ມູນ
cache-instruction = ຂໍ້ມູນ
cache-cpu = CPU
monitoring-section = ການຕິດຕາມ
fan-control-section = ການຄວບຄຸມພັດລົມ
temperatures = ອຸນຫະພູມ
oc-missing-fan-control-warning = ຄຳເຕືອນ: ການຮອງຮັບໂອເວີຄລັອກຖືກປິດໃຊ້ງານ, ຟັງຊັນການຄວບຄຸມພັດລົມຈຶ່ງບໍ່ສາມາດໃຊ້ງານໄດ້.
fan-speed = ຄວາມໄວພັດລົມ
throttling = ການລົດຄວາມໄວ
auto-page = ອັດຕະໂນມັດ
curve-page = ເສັ້ນໂຄ້ງ
static-page = ຄົງທີ່
target-temp = ອຸນຫະພູມເປົ້າໝາຍ (°C)
acoustic-limit = ຂີດຈຳກັດສຽງລົບກວນ (RPM)
acoustic-target = ເປົ້າໝາຍສຽງລົບກວນ (RPM)
min-fan-speed = ຄວາມໄວພັດລົມຕ່ຳສຸດ (%)
zero-rpm = ສູນ RPM
zero-rpm-stop-temp = ອຸນຫະພູມຢຸດພັດລົມ ສູນ RPM (°C)
static-speed = ຄວາມໄວຄົງທີ່ (%)
reset-button = ຕັ້ງຄ່າໃໝ່
pmfw-reset-warning = ຄຳເຕືອນ: ນີ້ແມ່ນການຕັ້ງຄ່າເຟີມແວພັດລົມຄືນໃໝ່!
temperature-sensor = ເຊັນເຊີອຸນຫະພູມ
spindown-delay = ຄວາມລ່າຊ້າໃນການລົດຄວາມໄວ (ms)
spindown-delay-tooltip = ໄລຍະເວລາທີ່ GPU ຕ້ອງຮັກສາອຸນຫະພູມໃນລະດັບຕ່ຳກ່ອນຈະລົດຄວາມໄວພັດລົມລົງ
speed-change-threshold = ເກນການປ່ຽນແປງຄວາມໄວ (°C)
automatic-mode-threshold = ເກນໂໝດອັດຕະໂນມັດ (°C)
automatic-mode-threshold-tooltip =
    ປ່ຽນການຄວບຄຸມພັດລົມເປັນໂໝດອັດຕະໂນມັດເມື່ອອຸນຫະພູມຕ່ຳກວ່າຈຸດນີ້.

    GPU ຂອງ Nvidia ຫຼາຍລຸ້ນຮອງຮັບພຽງແຕ່ການຢຸດພັດລົມໃນໂໝດຄວບຄຸມພັດລົມອັດຕະໂນມັດເທົ່ານັ້ນ, ໃນຂະນະທີ່ເສັ້ນໂຄ້ງທີ່ກຳນົດເອງມີຂອບເຂດຄວາມໄວຈຳກັດ ເຊັ່ນ 30-100%.

    ຕົວເລືອກນີ້ຊ່ວຍແກ້ໄຂຂໍ້ຈຳກັດນີ້ໂດຍການໃຊ້ເສັ້ນໂຄ້ງທີ່ກຳນົດເອງເມື່ອອຸນຫະພູມສູງກວ່າທີ່ກຳນົດໄວ້, ແລະ ໃຊ້ໂໝດອັດຕະໂນມັດທີ່ຕິດມາກັບກາດຈໍເຊິ່ງຮອງຮັບສູນ RPM ເມື່ອອຸນຫະພູມຕ່ຳກວ່າ.
amd-oc = ໂອເວີຄລັອກ AMD
amd-oc-disabled = ການໂອເວີຄລັອກ AMD ຍັງບໍ່ໄດ້ເປີດໃຊ້ງານ! <a href="https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)">ບາງຟັງຊັນຈະບໍ່ສາມາດໃຊ້ງານໄດ້.</a>
amd-oc-detected-system-config =
    ກວດພົບການຕັ້ງຄ່າລະບົບ: <b>{ $config ->
        [unsupported] ບໍ່ຮອງຮັບ
       *[other] { $config }
    }</b>
amd-oc-description =
    { $config ->
        [rpm-ostree] ຕົວເລືອກນີ້ຈະເປີດ/ປິດການຮອງຮັບ AMD overdrive ໂດຍການຕັ້ງຄ່າ boot flags ຜ່ານ <b>rpm-ostree</b>.
        [unsupported]
            ລະບົບປະຈຸບັນບໍ່ຖືກຮັບຮູ້ວ່າຮອງຮັບການຕັ້ງຄ່າ overdrive ອັດຕະໂນມັດ.
            ທ່ານອາດຈະພະຍາຍາມເປີດໃຊ້ການໂອເວີຄລັອກຈາກ LACT, ແຕ່ອາດຈະຕ້ອງໄດ້ສ້າງ initramfs ດ້ວຍຕົນເອງໃໝ່ເພື່ອໃຫ້ມີຜົນ.
            ຖ້າບໍ່ສຳເລັດ, ທາງເລືອກສຳຮອງແມ່ນການເພີ່ມ <b>amdgpu.ppfeaturemask=0xffffffff</b> ເປັນພາຣາມິເຕີການບູດໃນ bootloader ຂອງທ່ານ.
       *[other] ຕົວເລືອກນີ້ຈະເປີດ/ປິດການຮອງຮັບ AMD overdrive ໂດຍການສ້າງໄຟລ໌ທີ່ <b>{ $path }</b> ແລະ ອັບເດດ initramfs.
    }

    ເບິ່ງຂໍ້ມູນເພີ່ມເຕີມທີ່ <a href="https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)">wiki</a>.
enable-amd-oc-description = ນີ້ຈະເປີດໃຊ້ຄຸນສົມບັດ overdrive ຂອງໄດຣເວີ amdgpu ໂດຍການສ້າງໄຟລ໌ທີ່ <b>{ $path }</b> ແລະ ອັບເດດ initramfs. ທ່ານແນ່ໃຈບໍ່ວ່າຕ້ອງການເຮັດສິ່ງນີ້?
disable-amd-oc = ປິດການໂອເວີຄລັອກ AMD
enable-amd-oc = ເປີດການໂອເວີຄລັອກ AMD
disable-amd-oc-description = ນີ້ຈະປິດການຮອງຮັບໂອເວີຄລັອກ AMD (overdrive) ໃນການເປີດເຄື່ອງຄັ້ງຕໍ່ໄປ.
amd-oc-updating-configuration = ກຳລັງອັບເດດການຕັ້ງຄ່າ (ອາດຈະໃຊ້ເວລາຈັກໜ່ອຍ)
amd-oc-updating-done = ອັບເດດການຕັ້ງຄ່າແລ້ວ, ກະລຸນາເປີດເຄື່ອງໃໝ່ເພື່ອໃຫ້ການປ່ຽນແປງມີຜົນ.
reset-config = ຣີເຊັດການຕັ້ງຄ່າ
reset-config-description = ທ່ານແນ່ໃຈບໍ່ວ່າຕ້ອງການຣີເຊັດການຕັ້ງຄ່າ GPU ທັງໝົດ?
apply-button = ນຳໄປໃຊ້
revert-button = ກັບຄືນ
power-cap = ຂີດຈຳກັດການໃຊ້ພະລັງງານ
watt = W
ghz = GHz
mhz = MHz
bytes = ໄບຕ໌
kibibyte = KiB
mebibyte = MiB
gibibyte = GiB
stats-section = ສະຖິຕິ
gpu-clock = ຄວາມໄວ Core GPU
gpu-clock-avg = ຄວາມໄວ Core GPU (ສະເລ່ຍ)
gpu-clock-target = ຄວາມໄວ Core GPU (ເປົ້າໝາຍ)
gpu-voltage = ແຮງດັນໄຟ GPU
gpu-temp = ອຸນຫະພູມ
gpu-usage = ການໃຊ້ງານ GPU
vram-clock = ຄວາມໄວ VRAM
power-usage = ການໃຊ້ພະລັງງານ
no-throttling = ບໍ່ມີ
unknown-throttling = ບໍ່ຮູ້ຈັກ
missing-stat = N/A
vram-usage = ການໃຊ້ງານ VRAM:
performance-level-auto = ອັດຕະໂນມັດ
performance-level-high = ຄວາມໄວໂມງສູງສຸດ
performance-level-low = ຄວາມໄວໂມງຕ່ຳສຸດ
performance-level-manual = ກຳນົດເອງ
performance-level-auto-description = ປັບຄວາມໄວໂມງ GPU ແລະ VRAM ອັດຕະໂນມັດ. (ຄ່າເລີ່ມຕົ້ນ)
performance-level-high-description = ໃຊ້ຄວາມໄວໂມງສູງສຸດສຳລັບ GPU ແລະ VRAM ສະເໝີ.
performance-level-low-description = ໃຊ້ຄວາມໄວໂມງຕ່ຳສຸດສຳລັບ GPU ແລະ VRAM ສະເໝີ.
performance-level-manual-description = ຄວບຄຸມປະສິດທິພາບດ້ວຍຕົນເອງ.
performance-level = ລະດັບປະສິດທິພາບ
power-profile-mode = ໂໝດໂປຣໄຟລ໌ພະລັງງານ:
manual-level-needed = ຕ້ອງຕັ້ງຄ່າລະດັບປະສິດທິພາບເປັນ "ກຳນົດເອງ" ເພື່ອໃຊ້ສະຖານະພະລັງງານ ແລະ ໂໝດຕ່າງໆ
overclock-section = ຄວາມໄວໂມງ ແລະ ແຮງດັນໄຟ
nvidia-oc-info = ຂໍ້ມູນການໂອເວີຄລັອກ
nvidia-oc-description =
    ຟັງຊັນການໂອເວີຄລັອກໃນ Nvidia ລວມເຖິງການຕັ້ງຄ່າ offset ສຳລັບຄວາມໄວໂມງ GPU/VRAM ແລະ ການຈຳກັດຂອບເຂດຄວາມໄວໂມງທີ່ເປັນໄປໄດ້ ໂດຍໃຊ້ຄຸນສົມບັດ "ລັອກຄວາມໄວໂມງ".

    ສຳລັບກາດຈໍຫຼາຍລຸ້ນ, ການຕັ້ງຄ່າ offset ຄວາມໄວ VRAM ຈະສົ່ງຜົນຕໍ່ຄວາມໄວໜ່ວຍຄວາມຈຳຕົວຈິງພຽງເຄິ່ງໜຶ່ງຂອງຄ່າ offset ເທົ່ານັ້ນ.
    ຕົວຢ່າງ, ການຕັ້ງ offset VRAM +1000MHz ອາດຈະເພີ່ມຄວາມໄວ VRAM ທີ່ວັດແທກໄດ້ພຽງ 500MHz ເທົ່ານັ້ນ.
    ນີ້ແມ່ນເລື່ອງປົກກະຕິ, ແລະ ເປັນວິທີທີ່ Nvidia ຈັດການອັດຕາການສົ່ງຂໍ້ມູນຂອງ GDDR. ປັບການໂອເວີຄລັອກຂອງທ່ານຕາມຄວາມເໝາະສົມ.

    ສາມາດເຮັດການຫຼຸດແຮງດັນໄຟແບບຈຳລອງ (pseudo-undervolt) ໄດ້ໂດຍການປະສົມປະສານຕົວເລືອກລັອກຄວາມໄວໂມງກັບຄ່າ offset ຄວາມໄວໂມງທີ່ເປັນບວກ.
    ສິ່ງນີ້ຈະບັງຄັບໃຫ້ GPU ເຮັດວຽກໃນລະດັບແຮງດັນໄຟທີ່ຖືກຈຳກັດໂດຍຄວາມໄວໂມງທີ່ຖືກລັອກໄວ້, ໃນຂະນະດຽວກັນກໍໄດ້ຄວາມໄວໂມງທີ່ສູງຂຶ້ນເນື່ອງຈາກຄ່າ offset.
    ອັນນີ້ອາດຈະເຮັດໃຫ້ລະບົບບໍ່ສະຖຽນ ຖ້າຫາກດັນຂຶ້ນສູງເກີນໄປ.
oc-warning = ການປ່ຽນແປງຄ່າເຫຼົ່ານີ້ອາດເຮັດໃຫ້ລະບົບບໍ່ສະຖຽນ ແລະ ອາດສ້າງຄວາມເສຍຫາຍຕໍ່ຮາດແວຂອງທ່ານໄດ້!
show-all-pstates = ສະແດງ P-States ທັງໝົດ
enable-gpu-locked-clocks = ເປີດໃຊ້ງານການລັອກຄວາມໄວໂມງ GPU
enable-vram-locked-clocks = ເປີດໃຊ້ງານການລັອກຄວາມໄວໂມງ VRAM
pstate-list-description = <b>ຄ່າຕໍ່ໄປນີ້ແມ່ນຄ່າ offset ຂອງຄວາມໄວໂມງສຳລັບແຕ່ລະ P-State, ໂດຍເລີ່ມຈາກສູງສຸດໄປຫາຕ່ຳສຸດ.</b>
no-clocks-data = ບໍ່ມີຂໍ້ມູນຄວາມໄວໂມງ
reset-oc-tooltip = ຄຳເຕືອນ: ນີ້ແມ່ນການຣີເຊັດການຕັ້ງຄ່າຄວາມໄວໂມງທັງໝົດເປັນຄ່າເລີ່ມຕົ້ນ!
vf-curve-editor = ຕົວແກ້ໄຂເສັ້ນໂຄ້ງ VF
nvidia-vf-curve-warning =
    ຕົວແກ້ໄຂເສັ້ນໂຄ້ງແຮງດັນໄຟ-ຄວາມຖີ່ແມ່ນອາໄສຟັງຊັນຂອງໄດຣເວີທີ່ບໍ່ມີເອກະສານກຳກັບ.
    ບໍ່ມີການຮັບປະກັນໃດໆກ່ຽວກັບພຶດຕິກຳ, ຄວາມປອດໄພ ຫຼື ການໃຊ້ງານຂອງມັນ.
    <span weight = "heavy" underline = "single">ໃຊ້ດ້ວຍຄວາມສ່ຽງຂອງທ່ານເອງ</span>.
vf-curve-enable-editing = ເປີດໃຊ້ການແກ້ໄຂ
voltage = ແຮງດັນໄຟ
frequency = ຄວາມຖີ່
vf-active-curve = ເສັ້ນໂຄ້ງທີ່ເປີດໃຊ້
vf-base-curve = ເສັ້ນໂຄ້ງພື້ນຖານ
vf-curve-visible-range = ຊ່ວງທີ່ເບິ່ງເຫັນ (%):
vf-curve-visible-range-to = ຫາ
vf-curve-flatten-right = ເຮັດໃຫ້ເສັ້ນໂຄ້ງຮາບພຽງໄປທາງຂວາ
gpu-clock-offset = GPU Clock Offset (MHz)
max-gpu-clock = ຄວາມໄວໂມງ GPU ສູງສຸດ (MHz)
max-vram-clock = ຄວາມໄວໂມງ VRAM ສູງສຸດ (MHz)
max-gpu-voltage = ແຮງດັນໄຟ GPU ສູງສຸດ (mV)
min-gpu-clock = ຄວາມໄວໂມງ GPU ຕ່ຳສຸດ (MHz)
min-vram-clock = ຄວາມໄວໂມງ VRAM ຕ່ຳສຸດ (MHz)
min-gpu-voltage = ແຮງດັນໄຟ GPU ຕ່ຳສຸດ (mV)
gpu-voltage-offset = GPU voltage offset (mV)
gpu-pstate-clock-offset = GPU P-State { $pstate } Clock Offset (MHz)
vram-pstate-clock-offset = VRAM P-State { $pstate } Clock Offset (MHz)
gpu-pstate-clock = GPU P-State { $pstate } Clock (MHz)
mem-pstate-clock = VRAM P-State { $pstate } Clock (MHz)
gpu-pstate-clock-voltage = GPU P-State { $pstate } Voltage (mV)
mem-pstate-clock-voltage = VRAM P-State { $pstate } Voltage (mV)
pstates = ສະຖານະພະລັງງານ
gpu-pstates = ສະຖານະພະລັງງານ GPU
vram-pstates = ສະຖານະພະລັງງານ VRAM
pstates-manual-needed = ຕ້ອງຕັ້ງລະດັບປະສິດທິພາບເປັນ 'ກຳນົດເອງ' ເພື່ອເປີດ-ປິດ ສະຖານະພະລັງງານ
enable-pstate-config = ເປີດໃຊ້ການຕັ້ງຄ່າສະຖານະພະລັງງານ
show-historical-charts = ສະແດງກຣາຟ
show-process-monitor = ສະແດງການຕິດຕາມໂປຣເຊສ
generate-debug-snapshot = ສ້າງຂໍ້ມູນດີບັກ (Debug Snapshot)
dump-vbios = ດຶງຂໍ້ມູນ VBIOS (Dump VBIOS)
reset-all-config = ຣີເຊັດການຕັ້ງຄ່າທັງໝົດ
stats-update-interval = ໄລຍະເວລາອັບເດດ (ms)
historical-data-title = ຂໍ້ມູນປະຫວັດການໃຊ້ງານ
graphs-per-row = ຈຳນວນກຣາຟຕໍ່ແຖວ:
time-period-seconds = ໄລຍະເວລາ (ວິນາທີ):
reset-all-graphs-tooltip = ຣີເຊັດກຣາຟທັງໝົດເປັນຄ່າເລີ່ມຕົ້ນ
add-graph = ເພີ່ມກຣາຟ
delete-graph = ລຶບກຣາຟ
edit-graphs = ແກ້ໄຂ
export-csv = ສົ່ງອອກເປັນ CSV
edit-graph-sensors = ແກ້ໄຂເຊັນເຊີກຣາຟ
reconnecting-to-daemon = ຂາດການເຊື່ອມຕໍ່ກັບເດມອນ, ກຳລັງເຊື່ອມຕໍ່ໃໝ່...
daemon-connection-lost = ຂາດການເຊື່ອມຕໍ່
plot-show-detailed-info = ສະແດງຂໍ້ມູນແບບລະອຽດ
settings-profile = ໂປຣໄຟລ໌ການຕັ້ງຄ່າ
auto-switch-profiles = ປ່ຽນອັດຕະໂນມັດ
add-profile = ເພີ່ມໂປຣໄຟລ໌ໃໝ່
import-profile = ນຳເຂົ້າໂປຣໄຟລ໌ຈາກໄຟລ໌
create-profile = ສ້າງໂປຣໄຟລ໌
name = ຊື່
profile-copy-from = ສຳເນົາການຕັ້ງຄ່າຈາກ:
create = ສ້າງ
cancel = ຍົກເລີກ
save = ບັນທຶກ
default-profile = ຄ່າເລີ່ມຕົ້ນ
rename-profile = ປ່ຽນຊື່ໂປຣໄຟລ໌
rename-profile-from = ປ່ຽນຊື່ໂປຣໄຟລ໌ <b>{ $old_name }</b> ເປັນ:
delete-profile = ລຶບໂປຣໄຟລ໌
edit-rules = ແກ້ໄຂກົດ
edit-rule = ແກ້ໄຂກົດ
remove-rule = ລຶບກົດ
profile-rules = ກົດຂອງໂປຣໄຟລ໌
export-to-file = ສົ່ງອອກເປັນໄຟລ໌
move-up = ຍ້າຍຂຶ້ນ
move-down = ຍ້າຍລົງ
profile-activation = ການເປີດໃຊ້ງານ
profile-hooks = ຮຸກ (Hooks)
profile-activation-desc = ເປີດໃຊ້ໂປຣໄຟລ໌ '{ $name }' ເມື່ອ:
any-rules-matched = ກົງກັບກົດຂໍ້ໃດໜຶ່ງຕໍ່ໄປນີ້:
all-rules-matched = ກົງກັບກົດທັງໝົດຕໍ່ໄປນີ້:
activation-settings-status =
    ການຕັ້ງຄ່າການເປີດໃຊ້ງານທີ່ເລືອກໄວ້ປະຈຸບັນ <b>{ $matched ->
        [true] ກົງກັນ
       *[false] ບໍ່ກົງກັນ
    }</b>
activation-auto-switching-disabled = ການປ່ຽນໂປຣໄຟລ໌ອັດຕະໂນມັດຖືກປິດໃຊ້ງານຢູ່
profile-hook-command = ແລ່ນຄຳສັ່ງເມື່ອໂປຣໄຟລ໌ '{ $cmd }' ຖືກ:
profile-hook-activated = ເປີດໃຊ້ງານ:
profile-hook-deactivated = ປິດໃຊ້ງານ:
profile-hook-note = ໝາຍເຫດ: ຄຳສັ່ງເຫຼົ່ານີ້ຖືກແລ່ນໃນຖານະ root ໂດຍເດມອນ LACT, ແລະ ບໍ່ສາມາດເຂົ້າເຖິງສະພາບແວດລ້ອມເດັສທັອບໄດ້. ດັ່ງນັ້ນ, ພວກມັນຈຶ່ງບໍ່ສາມາດໃຊ້ເພື່ອເປີດແອັບພລິເຄຊັນກຣາຟິກໄດ້ໂດຍກົງ.
profile-rule-process-tab = ມີໂປຣເຊສກຳລັງແລ່ນຢູ່
profile-rule-gamemode-tab = Gamemode ກຳລັງເປີດໃຊ້ຢູ່
profile-rule-process-name = ຊື່ໂປຣເຊສ:
profile-rule-args-contain = ອາກິວເມັນ (Arguments) ປະກອບມີ:
profile-rule-specific-process = ດ້ວຍໂປຣເຊສສະເພາະ:
theme = ຮູບແບບ (Theme)
theme-auto = ອັດຕະໂນມັດ
preferences = ການຕັ້ງຄ່າ
ui = UI
daemon = ເດມອນ
about = ກ່ຽວກັບ
crash-page-title = ແອັບພລິເຄຊັນຂັດຂ້ອງ
exit = ອອກ
amd-oc-status =
    AMD Overclocking is currently: <b>{ $status ->
        [true] Enabled
        [false] Disabled
       *[other] Unknown
    }</b>
