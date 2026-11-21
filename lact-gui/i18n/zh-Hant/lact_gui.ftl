compute-units = 運算單元
language-system-default = 系統預設值
language-restart-notice = 重新啟動 LACT GUI 以套用語言變更。
experimental-features = 實驗性功能
nvidia-pstate-offsets = NVIDIA 各 P 狀態的偏移量
nvidia-pstate-offsets-description = 揭露非 P0 狀態的偏移量。預設為停用，因為目前尚無已知的裝置／驅動程式版本支援此功能。
info-page = 硬體資訊
oc-page = 超頻
thermals-page = 臨界
software-page = 軟體資訊
displays-page = 顯示器資訊
detach-page = 分離 { $page }
page-detached = 此頁面已在另一個視窗中開啟。
show-page-window = 焦點視窗
reattach-page = 返回主視窗
system-section = 系統
lact-daemon = LACT 常駐程式
lact-gui = LACT 圖形使用者介面
kernel-version = 內核版本
instance = 實例
device-name = 裝置名稱
platform-name = 平台名稱
api-version = API 版本
driver-name = 驅動程式名稱
driver-version = 驅動程式版本
cl-c-version = OpenCL C 版本
workgroup-size = 工作小組容量
global-memory = 全域記憶體
local-memory = 本地記憶體
extensions = 擴充功能
show-button = 顯示
device-not-found = 找不到 { $kind } 裝置
cache-info = 快取資訊
hw-ip-info = 硬體 IP 資訊
hw-queues = 佇列
amd-cache-desc =
    { $size } L{ $level } { $types } 快取 { $shared ->
        [1] 僅限於各 CU 內部
       *[other] 在 { $shared } 個 CU 之間共用
    }
nvidia-cache-desc = { $size } L{ $level }
cache-cpu = CPU
monitoring-section = 監測
thresholds-section = 閾值 &amp; 極限
fan-control-section = 風扇控制
oc-missing-fan-control-warning = 警告：超頻功能已停用，風扇控制功能無法使用。
fan-speed = 風扇轉速
no-fan-detected = 未偵測到風扇
no-sensors-found = 未找到任何感測器
throttling = 流量限制
auto-page = 自動化
curve-page = 曲線
static-page = 靜態
target-temp = 目標溫度（°C）
acoustic-limit = 噪音表現極限（RPM）
acoustic-target = 噪音表現目標（RPM）
min-fan-speed = 風扇最低轉速 (%)
zero-rpm = 零轉速
zero-rpm-stop-temp = 零轉速停止溫度（°C）
static-speed = 靜態速度（%）
reset-now-button = 立即重設
default-button = 預設
pmfw-reset-warning = 警告：此操作將重置風扇韌體設定！
temperature-sensor = 溫度感測器
spindown-delay = 停轉延遲 (毫秒)
spindown-delay-tooltip = GPU 必須維持在較低的溫度值多久，才會開始降低風扇轉速
speed-change-threshold = 速度變化閾值（°C）
automatic-mode-threshold = 自動模式閾值（°C）
automatic-mode-threshold-tooltip =
    當溫度低於此閾值時，將風扇控制切換至自動模式。

    許多 Nvidia 顯示卡僅在自動風扇控制模式下支援停止風扇運轉，而自訂曲線的轉速範圍則有限制，例如 30-100%。

    此選項可透過以下方式繞過此限制：僅在溫度高於特定閾值時使用自訂曲線，而在低於該閾值時則使用顯示卡內建且支援零轉速的自動模式。
amd-oc = AMD 超頻
amd-oc-disabled = AMD 超頻功能尚未啟用！<a href="https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)">部分功能將無法使用。</a>
amd-oc-status =
    AMD 超頻目前狀態為：<b>{ $status ->
        [true] 已啟用
        [false] 已停用
       *[other] 未知
    }</b>
amd-oc-detected-system-config =
    偵測到的系統配置：<b>{ $config ->
        [unsupported] 不支援
       *[other] { $config }
    }</b>
amd-oc-description =
    { $config ->
        [rpm-ostree] 此選項將透過 <b>rpm-ostree</b> 設定開機參數，以啟用或停用 AMD Overdrive 支援。
        [unsupported]
            系統目前未被識別為支援自動 Overdrive 配置。
            您可以嘗試透過 LACT 啟用超頻，但可能需要手動重新生成 initramfs 才能使設定生效。
            若此方法失敗，備用方案是在您的啟動載入器中將 <b>amdgpu.ppfeaturemask=0xffffffff</b> 作為啟動參數加入。
       *[other] 此選項將透過在 <b>{ $path }</b> 建立檔案並更新 initramfs，來切換 AMD Overdrive 的支援狀態。
    }

    請參閱 <a href="https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)">維基</a> 以獲取更多資訊。
enable-amd-oc-description = 這將透過在 <b>{ $path }</b> 建立檔案並更新 initramfs，來啟用 amdgpu 驅動程式的超頻功能。您確定要執行此操作嗎？
disable-amd-oc = 停用 AMD 超頻功能
enable-amd-oc = 啟用 AMD 超頻功能
disable-amd-oc-description = 這將在下次重新啟動時停用 AMD 超頻支援（Overdrive）。
amd-oc-updating-configuration = 正在更新設定（此過程可能需要一些時間）
amd-oc-updating-done = 設定已更新，請重新啟動以套用變更。
reset-config = 重設設定
reset-config-description = 此操作將將所有 GPU 設定重設為預設值，並永久刪除所有設定檔
apply-button = 套用
confirm = 確認
confirm-settings = 確認設定
revert-button = 還原
settings-confirmation = 您要保留新設定嗎？（將於 { $seconds_left } 秒後還原）
power-cap = 功率限制
power-section = 電源
watt = 瓦(W)
ghz = GHz
mhz = MHz
mv = mV
bytes = 位元組
kibibyte = KiB
mebibyte = MiB
gibibyte = GiB
stats-section = 統計資料
gpu-clock = GPU 核心時脈
core-section = 核心
gpu-clock-avg = GPU 核心時脈（平均值）
gpu-clock-target = GPU 核心時脈（目標值）
gpu-voltage = GPU 電壓
gpu-temp = 溫度
gpu-usage = GPU 使用率
vram-clock = VRAM 時脈
vram-section = VRAM
power-usage = 電源使用率
extra-clocks = 額外時鐘
no-throttling = 不
missing-stat = N/A
vram-usage = VRAM 使用率：
gtt-usage = GTT 使用率：
igpu-section = 整合式顯示卡
uma-carveout = UMA 分拆規模
uma-carveout-caption = 作為專用 VRAM 預留的記憶體容量。需重新啟動電腦才能生效。
performance-level-auto = 自動化
performance-level-high = 最高時脈
performance-level-low = 最低時脈
performance-level-manual = 手冊
performance-level-profile-standard = 分析標準
performance-level-profile-min-sclk = 分析最低 GPU 時脈
performance-level-profile-min-mclk = 分析最低 VRAM 時脈
performance-level-profile-peak = 峰值分析
performance-level-auto-description = 自動化調整 GPU 和 VRAM 時脈。（預設）
performance-level-high-description = 請始終將 GPU 和 VRAM 設定為最高時脈
performance-level-low-description = 請始終將 GPU 和 VRAM 設定為最低時脈
performance-level-manual-description = 手動效能控制
performance-level-profile-standard-description = 固定分析模式
performance-level-profile-min-sclk-description = 將 GPU 時脈強制設定為最低水準
performance-level-profile-min-mclk-description = 將 VRAM 時脈強制設定為最低水準
performance-level-profile-peak-description = 將 GPU 和 VRAM 時脈強制設定為最高水準
performance-level = 效能等級
power-profile-mode = 電源配置模式
manual-level-needed = 若要使用電源狀態和模式，必須將效能等級設定為「手動」
power-mizer-mode = PowerMizer 模式
power-mizer-mode-auto = 自動
power-mizer-mode-adaptive = 適應性
power-mizer-mode-prefer-maximum-performance = 追求極致效能
power-mizer-mode-prefer-consistent-performance = 偏好穩定表現
power-mizer-mode-auto-description = 讓驅動程式選擇效能策略。
power-mizer-mode-adaptive-description = 根據 GPU 使用率調整 GPU 時脈。
power-mizer-mode-prefer-maximum-performance-description = 在駕駛能力範圍內，應盡可能追求最佳表現。
power-mizer-mode-prefer-consistent-performance-description = 鎖定至 GPU 基準時脈。
overclock-section = 時脈速度與電壓
advanced-features = 進階功能
show-all-pstates = 所有 P 狀態
enable-locked-clocks = 鎖定的時脈
enable-vf-curve = 自訂 VF 曲線
pstate-list-description = <b>以下數值為各 P 狀態的時脈偏移量，由高至低排列。</b>
no-clocks-data = 沒有時鐘資料
reset-oc-tooltip = 警告：此操作會將所有時鐘設定重設為預設值！
vf-curve-editor = VF 曲線編輯器
nvidia-vf-curve-warning =
    電壓-頻率曲線編輯器仰賴未記載於文件中的驅動程式功能。
    <span weight = "heavy" underline = "single">使用時請自行承擔風險</span>。
voltage = 電壓
frequency = 頻率
vf-active-curve = 動態曲線
vf-base-curve = 基曲率
vf-curve-visible-range = 可見範圍（%）：
vf-curve-flatten-right = 將曲線向右平滑
vf-curve-flatten-selection = 將選取範圍扁平化
vf-curve-editing-disabled = 在 OC 頁面中，VF 曲線編輯功能已停用
gpu-clock-offset = 時脈偏移量
max-clock = 最高時脈
max-gpu-voltage = 最大電壓
min-clock = 最低時脈
min-gpu-voltage = 最低電壓
gpu-voltage-offset = 電壓偏移
gpu-voltage-boost = 電壓提升
gpu-voltage-boost-tooltip = 控制由驅動程式定義的額外電壓餘裕中，可用的比例。100% 表示可使用全部此餘裕，而非 GPU 總電壓的 100%。較大的餘裕雖可維持更高的時脈頻率，但會增加功耗與發熱量。
pstate-clock-offset = P 狀態 { $pstate } 時鐘偏移量
pstate-clock = P 狀態 { $pstate } 時脈
pstate-clock-voltage = P 狀態 { $pstate } 電壓
pstate = 電源狀態
pstates = 電源狀態
gpu-pstates = GPU 電源狀態
vram-pstates = VRAM 電源狀態
pstates-manual-needed = 若要啟用 P-State 設定，必須將效能等級設定為「手動」。
enable-pstate-config = P 狀態組態配置
menu = 選單
show-historical-charts = 顯示圖表
show-process-monitor = 顯示行程監視器
generate-debug-snapshot = 產生除錯快照
dump-vbios = 匯出 VBIOS
reset-all-config = 重設所有組態配置
stats-update-interval = 更新間隔 (毫秒)
historical-data-title = 歷史資料
graphs-per-row = 每行圖表數：
time-period-seconds = 時間範圍（秒）：
reset-all-graphs-tooltip = 將所有圖表重設為預設值
add-graph = 新增圖表
delete-graph = 刪除圖表
edit-graphs = 編輯
export-csv = 匯出為 CSV 檔案
edit-graph-sensors = 編輯圖形感測器
daemon-info-heading = 常駐程式資訊
reconnecting-to-daemon = 服務連線中斷，正在重新連線...
daemon-connection-lost = 連線中斷
service-explanation =
    套用 GPU 設定需要 LACT 系統服務。
    若未啟用該服務，LACT 將以獨立模式運作，此時僅提供資訊與監控功能。
service-setup-title = 服務設定
setup-error = 設定錯誤：{ $error }
service-connection-status = 連線狀態
service-status = 服務狀態
service-permission-denied =
    權限遭拒絕，此服務未設定為允許來自您此使用者的連線。
    請參閱 <a href="https://github.com/ilya-zlobintsev/lact#configuration">GitHub</a> 以取得更多資訊
service-connected = 已連線
service-disconnected = 未連接
service-version = 服務版本
gui-version = 圖形使用者介面 (GUI) 版本
service-version-mismatch = 不匹配
language = 語言
hardware-info = 硬體資訊
version = 版本
features = 功能
cache-data = 資料
cache-instruction = 資料
temperatures = 溫度 (F)
reset-button = 重新設置
unknown-throttling = 未知
vf-curve-visible-range-to = 改為
error-heading = 錯誤
service-start = 開始
service-logs = 服務記錄
service-stop = 停止
service-restart = 重新啟動
service-autostart = 開機時自動啟動
service-autostart-disable = 同時也請停用自動啟動功能
version-mismatch-description =
    GUI 與常駐程式版本不符（{ $gui_version }-{ $gui_commit } 對比 { $daemon_version }-{ $daemon_commit }）！
    若您已更新 LACT，則需重新啟動服務。
plot-show-detailed-info = 顯示詳細資訊
display-title = 顯示器 { $identifier }
display-manufacturer = 製造商
display-product-code = 產品編號
display-model = 型號
display-physical-size = 物理容量
display-manufacture-date = 製造日期
displays-missing = 未偵測到顯示器
settings-profile = 設定配置檔
auto-switch-profiles = 自動切換
add-profile = 新增新配置檔
import-profile = 從檔案匯入配置檔
create-profile = 建立配置檔
name = 名稱
profile-copy-from = 從以下位置複製設定：
create = 建立
default-profile = 預設
rename-profile = 重新命名配置檔
rename-profile-from = 將配置檔 <b>{ $old_name }</b> 重新命名為：
delete-profile = 刪除配置檔
edit-rules = 編輯規則
edit-rule = 編輯規則
remove-rule = 移除規則
profile-rules = 配置檔規則
export-to-file = 匯出至檔案
profile-hooks = 掛鉤
profile-activation-desc = 在以下情況下啟用「{ $name }」配置檔：
any-rules-matched = 若符合以下任一規則，即為匹配：
all-rules-matched = 以下所有規則均符合：
activation-settings-status =
    目前選定的啟用設定為 <b>{ $matched ->
        [true] 符合
       *[false] 不符
    }</b>
activation-auto-switching-disabled = 目前已停用自動切換配置檔功能
profile-hook-command = 當配置檔為「{ $cmd }」時，執行以下指令：
profile-hook-activated = 已啟用：
profile-hook-deactivated = 已停用：
profile-hook-note = 注意：這些指令是由 LACT 常駐程式以 root 身分執行的，且無法存取桌面環境。因此，無法直接使用這些指令來啟動圖形化應用程式。
profile-rule-process-tab = 有個行程正在執行中
profile-rule-gamemode-tab = 遊戲模式已啟用
profile-rule-process-name = 行程名稱：
profile-rule-args-contain = 參數包含：
profile-rule-specific-process = 透過特定的行程：
theme = 主題
theme-auto = 自動化
preferences = 偏好設定
ui = 介面
daemon = 常駐程式
color-scheme = 配色方案
color-scheme-auto = 系統
color-scheme-light = 日間
color-scheme-dark = 夜間
crash-page-title = 應用程式己當機
exit = 離開
display-connection = 連絡
cancel = 取消
close = 關閉
save = 儲存
profile-activation = 啟用
about = 關於
