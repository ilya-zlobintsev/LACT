info-page = 信息
oc-page = OC
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
amd-oc = AMD 超频
amd-oc-disabled =
    未启用 AMD 超频支持！
    您仍可修改基本设置，但高级时钟和电压控制将不可用。
amd-oc-status =
    AMD 超频当前状态 : <b>{ $status ->
        [true] 已启用
        [false] 已禁用
       *[other] 未知
    }</b>
amd-oc-detected-system-config =
    检测到的系统配置： <b>{ $config ->
        [unsupported] 不支持
       *[other] { $config }
    }</b>
disable-amd-oc = 禁用 AMD 超频
enable-amd-oc = 启用 AMD 超频
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
            您可以尝试通过 LACT 启用超频，但可能需要手动重新生成 initramfs 才能生效。
            如果仍无法生效，备用方案是在引导程序中添加 <b>amdgpu.ppfeaturemask=0xffffffff</b> 启动参数。
       *[other] 此选项通过在 <b>{ $path }</b> 创建文件并更新 initramfs 来切换 AMD Overdrive 支持。
    }

    更多信息详见 <a href="https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)"> Wiki 文档</a> 。
enable-amd-oc-description = 此操作将通过在 <b>{ $path }</b> 创建文件并更新 initramfs 来启用 amdgpu 驱动的 Overdrive 特性。您确定要继续吗？
disable-amd-oc-description = 此操作将在下一次重启时禁用 AMD 超频 (Overdrive) 支持。
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
gpu-usage = GPU 使用率
vram-clock = 显存时钟
power-usage = 功耗
no-throttling = 无
missing-stat = N/A
vram-usage = 显存使用：
performance-level-high = 最高时钟
performance-level-low = 最低时钟
performance-level-manual = 手动
performance-level-auto-description = 自动调节 GPU 和显存时钟 (默认)
performance-level-high-description = 始终使用 GPU 和显存的最高时钟速度。
performance-level-low-description = 始终使用 GPU 和显存的最低时钟速度。
performance-level-manual-description = 手动性能控制。
performance-level = 性能等级
power-profile-mode = 功耗模式:
manual-level-needed = 必须将性能等级设置为“手动”，才能使用电源状态和模式
overclock-section = 时钟速度和电压
nvidia-oc-info = 英伟达超频信息
oc-warning = 警告：更改这些数值可能导致系统不稳定，并存在损坏硬件的风险！
show-all-pstates = 显示所有 P-State
enable-gpu-locked-clocks = 启用 GPU 时钟锁定
enable-vram-locked-clocks = 启用显存时钟锁定
mebibyte = MiB
unknown-throttling = 未知
performance-level-auto = 自动
name = 名称
save = 保存
default-profile = 默认
pstate-list-description = <b>以下数值为各 P-State 的时钟偏移，从最高性能到最低性能排列。</b>
no-clocks-data = 无可用时钟数据
reset-oc-tooltip = 警告：此操作会重置所有时钟设置为默认值！
gpu-clock-offset = GPU 时钟偏移 (MHz)
max-gpu-clock = 最大 GPU 时钟 (MHz)
max-vram-clock = 最大显存时钟 (MHz)
max-gpu-voltage = 最大 GPU 电压 (mV)
min-gpu-clock = 最小 GPU 时钟 (MHz)
min-vram-clock = 最小显存时钟 (MHz)
min-gpu-voltage = 最小 GPU 电压 (mV)
gpu-voltage-offset = GPU 电压偏移 (mV)
gpu-pstate-clock-offset = GPU P-State { $pstate } 时钟偏移 (MHz)
vram-pstate-clock-offset = 显存 P-State { $pstate } 时钟偏移 (MHz)
gpu-pstate-clock = GPU P-State { $pstate } 时钟 (MHz)
mem-pstate-clock = 显存 P-State { $pstate } 时钟 (MHz)
gpu-pstate-clock-voltage = GPU P-State { $pstate } 电压 (mV)
mem-pstate-clock-voltage = GPU P-State { $pstate } 电压(mV)
pstates = 电源状态
gpu-pstates = GPU 电源状态
vram-pstates = 显存电源状态
pstates-manual-needed = 注意：必须将性能等级设置为“手动”，才能切换电源状态
enable-pstate-config = 启用电源状态配置
show-historical-charts = 显示历史图表
show-process-monitor = 显示进程监视器
generate-debug-snapshot = 生成调试快照
dump-vbios = 转储 VBIOS
reset-all-config = 重置所有配置
stats-update-interval = 更新间隔 (ms)
historical-data-title = 历史数据
graphs-per-row = 每行图表数量：
time-period-seconds = 时间周期 (秒)：
reset-all-graphs-tooltip = 重置所有图表为默认
add-graph = 添加图表
delete-graph = 删除图表
export-csv = 导出为 CSV
edit-graph-sensors = 编辑图表传感器
reconnecting-to-daemon = 守护进程连接丢失，正在重新连接...
daemon-connection-lost = 连接丢失
plot-show-detailed-info = 显示详细信息
settings-profile = 设置配置文件
auto-switch-profiles = 自动切换
add-profile = 添加新配置文件
import-profile = 从文件导入配置文件
create-profile = 创建配置文件
profile-copy-from = 复制设置自：
create = 创建
rename-profile = 重命名配置文件
rename-profile-from = 将配置文件 <b>{ $old_name }</b> 重命名为：
delete-profile = 删除配置文件
edit-rules = 编辑规则
edit-rule = 编辑规则
remove-rule = 移除规则
profile-rules = 配置文件规则
export-to-file = 导出至文件
move-up = 上移
move-down = 下移
profile-activation = 激活
profile-hooks = 挂钩
profile-activation-desc = 满足以下条件时，激活配置文件 '{ $name }'：
any-rules-matched = 匹配以下任意规则时：
all-rules-matched = 匹配以下所有规则时：
activation-settings-status =
    当前选择的激活设置 <b>{ $matched ->
        [true] 已匹配
       *[false] 未匹配
    }</b>
activation-auto-switching-disabled = 当前已禁用自动配置文件切换
profile-hook-command = 运行以下命令，当配置文件 '{ $cmd }' ：
profile-hook-activated = 已激活：
profile-hook-deactivated = 已停用：
profile-hook-note = 注意：这些命令由 LACT 守护进程以 root 身份执行，无法访问桌面环境。因此，不能直接用于启动图形化应用程序。
profile-rule-process-tab = 进程正在运行时
profile-rule-gamemode-tab = 游戏模式激活时
profile-rule-process-name = 进程名称：
profile-rule-args-contain = 参数中包含：
profile-rule-specific-process = 使用特定进程：
nvidia-oc-description =
    英伟达超频功能包括为GPU和显存设置时钟速度偏移，使用“锁定时钟”特性限制时钟速度范围。

    在许多显卡上，显存时钟速度偏移对实际显存时钟速度的影响仅为偏移数值的一半。
    例如， +1000MHz 的显存偏移可能只会使测得的显存速度增加 500MHz.
    这是正常现象，因为英伟达处理 GDDR 数据速率的方式就是如此。请据此调整你的超频设置。

    不支持直接电压控制，因为英伟达 Linux 驱动中不存在此功能。

    可通过将锁定时钟选项与正时钟偏移结合使用，实现类似降压效果。
    这种方式会强制 GPU 在受锁定时钟限制的电压下运行，同时通过偏移实现更高的时钟速度。
    如果偏移过高，可能会导致系统不稳定。
