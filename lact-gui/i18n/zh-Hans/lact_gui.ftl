info-page = 信息
oc-page = 超频
thermals-page = 散热
software-page = 软件
hardware-info = 硬件信息
system-section = 系统
lact-daemon = LACT 守护进程
lact-gui = LACT图形用户界面
kernel-version = 内核版本
instance = 实例
compute-units = 计算单元数
device-name = 设备名称
platform-name = 平台名称
api-version = API 版本
version = 版本
driver-name = 驱动名称
driver-version = 驱动版本
cl-c-version = OpenCL C 版本
workgroup-size = 工作组大小
global-memory = 全局内存
local-memory = 本地内存
features = 特性
extensions = 扩展
show-button = 显示
device-not-found = 未找到 { $kind } 设备
cache-info = 缓存信息
cache-data = 数据
monitoring-section = 监控
fan-control-section = 风扇控制
temperatures = 温度
oc-missing-fan-control-warning = 警告：超频支持已禁用，风扇控制功能不可用。
fan-speed = 风扇速度
throttling = 降频
auto-page = 自动
curve-page = 曲线
static-page = 固定
target-temp = 目标温度 (°C)
acoustic-limit = 噪音限制 (转/分钟)
acoustic-target = 噪音目标 (转/分钟)
min-fan-speed = 最小风扇转速 (%)
zero-rpm = 零转速
zero-rpm-stop-temp = 零转速停转温度 (°C)
static-speed = 固定转速 (%)
reset-button = 重置
pmfw-reset-warning = 警告：此操作会重置风扇固件设置！
temperature-sensor = 温度传感器
spindown-delay-tooltip = 风扇降速前，GPU 需要在较低温度下保持的时间
spindown-delay = 降速延迟 (ms)
speed-change-threshold = 转速变化阈值 (°C)
automatic-mode-threshold = 自动模式阈值 (°C)
automatic-mode-threshold-tooltip =
    当温度低于此值时，自动切换风扇控制为自动模式。

    许多英伟达 GPU 仅在自动风扇控制模式下支持风扇停转，而自定义曲线通常只支持有限的转速范围 (例如 30%–100%)。

    此选项通过仅在高于该温度时使用自定义曲线来绕过此限制，而在低于该温度时使用显卡内置且支持零转速的自动模式。
amd-oc = AMD Overclocking
amd-oc-disabled =
    未启用 AMD Overclocking 支持！
    您仍可修改基本设置，但高级时钟和电压控制将不可用。
amd-oc-status =
    AMD Overclocking 当前状态 : <b>{ $status ->
        [true] 已启用
        [false] 已禁用
       *[other] 未知
    }</b>
amd-oc-detected-system-config =
    检测到的系统配置： <b>{ $config ->
        [unsupported] 不支持
       *[other] { $config }
    }</b>
disable-amd-oc = 禁用 AMD Overclocking
enable-amd-oc = 启用 AMD Overclocking
edit-graphs = 编辑
apply-button = 应用
cancel = 取消
amd-cache-desc =
    { $size } L{ $level } { $types } 缓存{ $shared ->
        [1] ，每个 CU 独立
       *[other] ，在 { $shared } 个 CU 间共享
    }
nvidia-cache-desc = { $size } L{ $level }
cache-instruction = 指令
cache-cpu = 处理器
amd-oc-description =
    { $config ->
        [rpm-ostree] 此选项通过 <b>rpm-ostree</b>启动标志来切换 AMD Overdrive 支持。
        [unsupported]
            当前系统未被识别为支持自动 Overdrive 配置。
            您可以尝试通过 LACT 启用 Overclocking，但可能需要手动重新生成 initramfs 才能生效。
            如果仍无法生效，备用方案是在引导程序中添加 <b>amdgpu.ppfeaturemask=0xffffffff</b> 启动参数。
       *[other] 此选项通过在 <b>{ $path }</b> 创建文件并更新 initramfs 来切换 AMD Overdrive 支持。
    }

    更多信息详见 <a href="https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)"> Wiki 文档</a> 。
enable-amd-oc-description = 此操作将通过在 <b>{ $path }</b> 创建文件并更新 initramfs 来启用 amdgpu 驱动的 Overdrive 功能。您确定要继续吗？
disable-amd-oc-description = 此操作将在下一次重启时禁用 AMD Overclocking (Overdrive) 支持。
amd-oc-updating-configuration = 正在更新配置 (可能需要一些时间)
amd-oc-updating-done = 配置已更新，请重启以应用更改。
reset-config = 重置配置
reset-config-description = 您确定要重置所有 GPU 配置吗？
revert-button = 还原
power-cap = 功耗限制
watt = W
ghz = GHz
mhz = MHz
stats-section = 状态统计
gpu-clock = GPU 核心时钟
gpu-clock-avg = GPU 核心时钟 (平均)
gpu-clock-target = GPU 核心时钟 (目标)
gpu-voltage = GPU 电压
gpu-temp = 温度
gpu-usage = GPU 利用率
vram-clock = 显存时钟
power-usage = 功耗
no-throttling = 无
missing-stat = N/A
vram-usage = 显存利用率:
performance-level-high = 最高时钟
performance-level-low = 最低时钟
performance-level-manual = 手动
performance-level-auto-description = 自动调节 GPU 和显存时钟 (默认)
performance-level-high-description = 始终使用 GPU 和显存的最高时钟速度。
performance-level-low-description = 始终使用 GPU 和显存的最低时钟速度。
performance-level-manual-description = 手动性能控制。
performance-level = 性能等级
power-profile-mode = 功耗模式:
manual-level-needed = 必须将性能等级设置为“手动”，才能使用功耗状态和模式
overclock-section = 时钟速度和电压
nvidia-oc-info = 英伟达超频信息
oc-warning = 警告：更改这些数值可能导致系统不稳定，并存在损坏硬件的风险！
show-all-pstates = 显示所有性能状态
enable-gpu-locked-clocks = 启用 GPU 时钟锁定
enable-vram-locked-clocks = 启用显存时钟锁定
mebibyte = MiB
unknown-throttling = 未知
performance-level-auto = 自动
name = 名称
save = 保存
default-profile = 默认
