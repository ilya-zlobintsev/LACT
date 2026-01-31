info-page = Information 
oc-page = OC 
thermals-page = Thermals 
software-page = Software 

hardware-info = Hardware Information

system-section = System
lact-daemon = LACT Daemon
lact-gui = LACT GUI
kernel-version = Kernel Version

instance = Instance
device-name = Device Name
platform-name = Platform Name
api-version = API Version
version = Version
driver-name = Driver Name
driver-version = Driver Version
compute-units = Compute Units
cl-c-version = OpenCL C Version
workgroup-size = Workgroup Size
global-memory = Global Memory
local-memory = Local Memory
features = Features
extensions = Extensions
show-button = Show
device-not-found = {$kind} device not found
cache-info = Cache Information
amd-cache-desc = {$size} L{$level} {$types} cache { $shared ->
    [1] local to each CU
    *[other] shared between {$shared} CUs
}
nvidia-cache-desc = {$size} L{$level}
cache-data = Data
cache-instruction = Data
cache-cpu = CPU

monitoring-section = Monitoring
fan-control-section = Fan Control
temperatures = Temperatures
oc-missing-fan-control-warning = Warning: Overclocking support is disabled, fan control functionality is not available.
fan-speed = Fan Speed
throttling = Throttling
auto-page = Automatic
curve-page = Curve
static-page = Static
target-temp = Target temperature (°C)
acoustic-limit = Acoustic Limit (RPM)
acoustic-target = Acoustic Target (RPM)
min-fan-speed = Minimum Fan Speed (%)
zero-rpm = Zero RPM
zero-rpm-stop-temp = Zero RPM stop temperature (°C)
static-speed = Static Speed (%)
reset-button = Reset
pmfw-reset-warning = Warning: this resets the fan firmware settings!

temperature-sensor = Temperature Sensor
spindown-delay = Spindown Delay (ms)
spindown-delay-tooltip = How long the GPU needs to remain at a lower temperature value before ramping down the fan
speed-change-threshold = Speed Change Threshold (°C)
automatic-mode-threshold = Automatic Mode Threshold (°C)
automatic-mode-threshold-tooltip = Switch fan control to auto mode when the temperature is below this point.

    Many Nvidia GPUs only support stopping the fan in the automatic fan control mode, while a custom curve has a limited speed range such as 30-100%.

    This option allows to work around this limitation by only using the custom curve when above a specific temperature, with the card's builtin auto mode that supports zero RPM being used below it.

amd-oc = AMD Overclocking
amd-oc-disabled = 
    AMD Overclocking support is not enabled!
    You can still change basic settings, but the more advanced clocks and voltage control will not be available.
amd-oc-status = AMD Overclocking is currently: <b>{$status ->
    [true] Enabled
    [false] Disabled
    *[other] Unknown
}</b>
amd-oc-detected-system-config = Detected system configuration: <b>{$config ->
    [unsupported] Unsupported
    *[other] {$config}
}</b>
amd-oc-description = 
    {$config ->
        [rpm-ostree] This option will toggle AMD overdrive support by setting boot flags through <b>rpm-ostree</b>.
        [unsupported] 
            The current system is not recognized as supported for automatic overdrive configuration.
            You may attempt to enable overclocking from LACT, but a manual initramfs regeneration may be required for it to take effect.
            If that fails, a fallback option is to add <b>amdgpu.ppfeaturemask=0xffffffff</b> as a boot parameter in your bootloader.
        *[other] This option will toggle AMD overdrive support by creating a file at <b>{$path}</b> and updating the initramfs.
    }

    See <a href="https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)">the wiki</a> for more information.
enable-amd-oc-description = This will enable the overdrive feature of the amdgpu driver by creating a file at <b>{$path}</b> and updating the initramfs. Are you sure you want to do this?
disable-amd-oc = Disable AMD Overclocking
enable-amd-oc = Enable AMD Overclocking
disable-amd-oc-description = This will disable AMD overclocking support (overdrive) on next reboot.
amd-oc-updating-configuration = Updating configuration (this may take a while)
amd-oc-updating-done = Configuration updated, please reboot to apply changes.

reset-config = Reset Configuration
reset-config-description = Are you sure you want to reset all GPU configuration?

apply-button = Apply
revert-button = Revert

power-cap = Power Usage Limit

watt = W
ghz = GHz
mhz = MHz
bytes = bytes
kibibyte = KiB
mebibyte = MiB
gibibyte = GiB

stats-section = Statistics
gpu-clock = GPU Core Clock
gpu-clock-avg = GPU Core Clock (Average)
gpu-clock-target = GPU Core Clock (Target)
gpu-voltage = GPU Voltage
gpu-temp = Temperature
gpu-usage = GPU Usage
vram-clock = VRAM Clock
power-usage = Power Usage
no-throttling = No
unknown-throttling = Unknown
missing-stat = N/A
vram-usage = VRAM Usage:

performance-level-auto = Automatic
performance-level-high = Highest Clocks
performance-level-low = Lowest Clocks
performance-level-manual = Manual
performance-level-auto-description = Automatically adjust GPU and VRAM clocks. (Default)
performance-level-high-description = Always use the highest clockspeeds for GPU and VRAM.
performance-level-low-description = Always use the lowest clockspeeds for GPU and VRAM.
performance-level-manual-description = Manual performance control.

performance-level = Performance Level
power-profile-mode = Power Profile Mode:
manual-level-needed = Performance level has to be set to "manual" to use power states and modes

overclock-section = Clockspeed and Voltage
nvidia-oc-info = Nvidia Overclocking Information
nvidia-oc-description = 
    Overclocking functionality on Nvidia includes setting offsets for GPU/VRAM clockspeeds and limiting the potential range of clockspeeds using the "locked clocks" feature.

    On many cards, the VRAM clockpeed offset will only affect the actual memory clockspeed by half of the offset value.
    For example, a +1000MHz VRAM offset may only increase the measured VRAM speed by 500MHz.
    This is normal, and is how Nvidia handles GDDR data rates. Adjust your overclock accordingly.

    Direct voltage control is not supported, as it does not exist in the Nvidia Linux driver.

    It is possible to achieve a pseudo-undervolt by combining the locked clocks option with a positive clockspeed offset.
    This will force the GPU to run at a voltage that's constrained by the locked clocks, while achieving a higher clockspeed due to the offset.
    This can cause system instability if pushed too high.
oc-warning = Warning: changing these values may lead to system instability and can potentially damage your hardware!
show-all-pstates = Show all P-States
enable-gpu-locked-clocks = Enable GPU Locked Clocks
enable-vram-locked-clocks = Enable VRAM Locked Clocks
pstate-list-description = <b>The following values are clock offsets for each P-State, going from highest to lowest.</b>
no-clocks-data = No clocks data available
reset-oc-tooltip = Warning: this resets all clock settings to defaults!

gpu-clock-offset = GPU Clock Offset (MHz)
max-gpu-clock = Maximum GPU Clock (MHz)
max-vram-clock = Maximum VRAM Clock (MHz) 
max-gpu-voltage = Maximum GPU Voltage (mV) 
min-gpu-clock = Minimum GPU Clock (MHz)
min-vram-clock = Minimum VRAM Clock (MHz) 
min-gpu-voltage = Minimum GPU Voltage (mV) 
gpu-voltage-offset = GPU voltage offset (mV)
gpu-pstate-clock-offset = GPU P-State {$pstate} Clock Offset (MHz)
vram-pstate-clock-offset = VRAM P-State {$pstate} Clock Offset (MHz)
gpu-pstate-clock = GPU P-State {$pstate} Clock (MHz)
mem-pstate-clock = VRAM P-State {$pstate} Clock (MHz)
gpu-pstate-clock-voltage = GPU P-State {$pstate} Voltage (mV)
mem-pstate-clock-voltage = VRAM P-State {$pstate} Voltage (mV)

pstates = Power States
gpu-pstates = GPU Power States
vram-pstates = VRAM Power States
pstates-manual-needed = Note: performance level must be set to 'manual' to toggle power states
enable-pstate-config = Enable power state configuration

show-historical-charts = Show Historical Charts
show-process-monitor = Show Process Monitor
generate-debug-snapshot = Generate Debug Snapshot
dump-vbios = Dump VBIOS
reset-all-config = Reset All Configuration
stats-update-interval = Update Interval (ms)

historical-data-title = Historical Data
graphs-per-row = Graphs Per Row:
time-period-seconds = Time Period (Seconds):
reset-all-graphs-tooltip = Reset All Graphs To Default
add-graph = Add Graph
delete-graph = Delete Graph
edit-graphs = Edit
export-csv = Export as CSV
edit-graph-sensors = Edit Graph Sensors

reconnecting-to-daemon = Daemon connection lost, reconnecting...
daemon-connection-lost = Connection Lost

plot-show-detailed-info = Show detailed info

settings-profile = Settings Profile
auto-switch-profiles = Switch automatically
add-profile = Add new profile
import-profile = Import profile from file

create-profile = Create Profile
name = Name
profile-copy-from = Copy settings from:
create = Create
cancel = Cancel
save = Save
default-profile = Default
rename-profile = Rename Profile
rename-profile-from = Rename profile <b>{$old_name}</b> to:
delete-profile = Delete Profile
edit-rules = Edit Rules
edit-rule = Edit Rule
remove-rule = Remove Rule
profile-rules = Profile Rules
export-to-file = Export To File
move-up = Move Up
move-down = Move Down
profile-activation = Activation
profile-hooks = Hooks
profile-activation-desc = Activate profile '{$name}' when:
any-rules-matched = Any of the following rules are matched:
all-rules-matched = All of the following rules are matched:
activation-settings-status = Selected activation settings are currently <b>{ $matched ->
    [true] matched
    *[false] not matched
}</b>
activation-auto-switching-disabled = Automatic profile switching is currently disabled
profile-hook-command = Run a command when the profile '{$cmd}' is:
profile-hook-activated = Activated:
profile-hook-deactivated = Deactivated:
profile-hook-note = Note: these commands are executed as root by the LACT daemon, and do not have access to the desktop environment. As such, they cannot be used directly to launch graphical applications.

profile-rule-process-tab = A process is running
profile-rule-gamemode-tab = Gamemode is active
profile-rule-process-name = Process Name:
profile-rule-args-contain = Arguments Contain:
profile-rule-specific-process = With a specific process:

# Crash page
crash-page-title = Application Crashed
exit = Exit
