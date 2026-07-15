info-page = 하드웨어 정보
oc-page = 오버클럭
thermals-page = 온도
software-page = 소프트 정보
hardware-info = 하드웨어 정보
system-section = 시스템
lact-daemon = LACT 데몬
lact-gui = LACT GUI
kernel-version = 커널 버전
instance = 인스턴스
device-name = 장치 이름
platform-name = 플랫폼 이름
api-version = API 버전
version = 버전
driver-name = 드라이버 이름
driver-version = 드라이버 버전
compute-units = 연산 단위
cl-c-version = OpenCL C 버전
workgroup-size = 워크그룹 크기
global-memory = 글로벌 메모리
local-memory = 로컬 메모리
features = 기능
extensions = 확장
show-button = 보기
device-not-found = { $kind } 장치를 찾을 수 없습니다
cache-info = 캐쉬 정보
hw-ip-info = 하드웨어 IP 정보
hw-queues = 큐
amd-cache-desc =
    { $size } L{ $level } { $types } 캐시 { $shared ->
        [1] 각 CU에 로컬
       *[other] { $shared }개 CU 간 공유
    }
nvidia-cache-desc = { $size } L{ $level }
cache-data = 데이터
cache-instruction = 명령어
cache-cpu = CPU
monitoring-section = 모니터링
fan-control-section = 팬 제어
temperatures = 온도
oc-missing-fan-control-warning = 경고: 오버클럭 지원이 비활성화되어 있어 팬 제어 기능을 사용할 수 없습니다.
fan-speed = 팬 속도
throttling = 스로틀링
auto-page = 자동
curve-page = 커브
static-page = 고정
target-temp = 목표 온도 (°C)
acoustic-limit = 소음 제한 (RPM)
acoustic-target = 소음 목표 (RPM)
min-fan-speed = 최소 팬 속도 (%)
zero-rpm = 0 RPM
zero-rpm-stop-temp = Zero RPM 정지 온도 (°C)
static-speed = 고정 속도 (%)
reset-button = 초기화
pmfw-reset-warning = 경고: 팬 펌웨어 설정이 초기화됩니다!
temperature-sensor = 온도 센서
spindown-delay = 감속 지연 (ms)
spindown-delay-tooltip = GPU가 팬 속도를 낮추기 전에 온도가 낮은 상태를 유지해야 하는 시간
speed-change-threshold = 속도 변경 임계값 (°C)
automatic-mode-threshold = 자동 모드 임계값 (°C)
automatic-mode-threshold-tooltip =
    온도가 이 값 이하일 때 팬 제어를 자동 모드로 전환합니다.

    많은 Nvidia GPU는 자동 팬 제어 모드에서만 팬 정지를 지원하며, 커스텀 커브는 30-100%와 같은 제한된 속도 범위를 가집니다.

    이 옵션은 특정 온도 이상에서만 커스텀 커브를 사용하고, 그 이하에서는 Zero RPM을 지원하는 카드 내장 자동 모드를 사용하여 이 제한을 우회할 수 있게 합니다.
amd-oc = AMD 오버클럭
amd-oc-disabled = AMD 오버클럭이 활성화되지 않았습니다! <a href="https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)">일부 기능을 사용할 수 없습니다.</a>
amd-oc-status =
    AMD 오버클럭 현재 상태: <b>{ $status ->
        [true] 활성화
        [false] 비활성화
       *[other] 알 수 없음
    }</b>
amd-oc-detected-system-config =
    감지된 시스템 구성: <b>{ $config ->
        [unsupported] 지원되지 않음
       *[other] { $config }
    }</b>
amd-oc-description =
    { $config ->
        [rpm-ostree] 이 옵션은 <b>rpm-ostree</b>를 통해 부팅 플래그를 설정하여 AMD 오버드라이브 지원을 토글합니다.
        [unsupported]
            현재 시스템은 자동 오버드라이브 구성이 지원되지 않는 것으로 인식되었습니다.
            LACT에서 오버클럭을 활성화할 수 있지만, 적용하려면 수동으로 initramfs를 재생성해야 할 수 있습니다.
            실패할 경우, 부트로더에 <b>amdgpu.ppfeaturemask=0xffffffff</b>를 부팅 매개변수로 추가하는 대체 방법을 사용할 수 있습니다.
       *[other] 이 옵션은 <b>{ $path }</b>에 파일을 생성하고 initramfs를 업데이트하여 AMD 오버드라이브 지원을 토글합니다.
    }

    자세한 내용은 <a href="https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)">위키</a>를 참조하십시오.
enable-amd-oc-description = <b>{ $path }</b>에 파일을 생성하고 initramfs를 업데이트하여 amdgpu 드라이버의 오버드라이브 기능을 활성화합니다. 정말 실행하시겠습니까?
disable-amd-oc = AMD 오버클럭 비활성화
enable-amd-oc = AMD 오버클럭 활성화
disable-amd-oc-description = 다음 재부팅 시 AMD 오버클럭(오버드라이브) 지원이 비활성화됩니다.
amd-oc-updating-configuration = 구성 업데이트 중 (시간이 소요될 수 있습니다)
amd-oc-updating-done = 구성이 업데이트되었습니다. 변경 사항을 적용하려면 재부팅하십시오.
reset-config = 구성 초기화
reset-config-description = 모든 GPU 구성을 초기화하시겠습니까?
apply-button = 적용
confirm = 확인
confirm-settings = 설정 확인
revert-button = 되돌리기
settings-confirmation = 새 설정을 유지하시겠습니까? ({ $seconds_left }초 후 되돌려집니다)
power-cap = 전력 사용량 제한
watt = W
ghz = GHz
mhz = MHz
bytes = 바이트
kibibyte = KiB
mebibyte = MiB
gibibyte = GiB
stats-section = 통계
gpu-clock = GPU 코어 클럭
gpu-clock-avg = GPU 코어 클럭 (평균)
gpu-clock-target = GPU 코어 클럭 (목표)
gpu-voltage = GPU 전압
gpu-temp = 온도
gpu-usage = GPU 사용량
vram-clock = VRAM 클럭
power-usage = 전력 사용량
no-throttling = 없음
unknown-throttling = 알 수 없음
missing-stat = 해당 없음
vram-usage = VRAM 사용량:
performance-level-auto = 자동
performance-level-high = 최대 클럭
performance-level-low = 최소 클럭
performance-level-manual = 수동
performance-level-auto-description = GPU 및 VRAM 클럭을 자동으로 조절합니다. (기본값)
performance-level-high-description = GPU 및 VRAM에 항상 최대 클럭 속도를 사용합니다.
performance-level-low-description = GPU 및 VRAM에 항상 최소 클럭 속도를 사용합니다.
performance-level-manual-description = 수동 성능 제어.
performance-level = 성능 레벨
power-profile-mode = 전력 프로파일 모드:
manual-level-needed = 전력 상태 및 모드를 사용하려면 성능 레벨을 "수동"으로 설정해야 합니다
overclock-section = 클럭 속도 및 전압
nvidia-oc-info = 오버클럭 정보
nvidia-oc-description =
    Nvidia의 오버클럭 기능에는 GPU/VRAM 클럭 속도 오프셋 설정과 "잠금 클럭" 기능을 사용한 클럭 속도 범위 제한이 포함됩니다.

    많은 카드에서 VRAM 클럭 속도 오프셋은 실제 메모리 클럭 속도에 오프셋 값의 절반만 반영됩니다.
    예를 들어, +1000MHz VRAM 오프셋은 측정된 VRAM 속도를 500MHz만 증가시킬 수 있습니다.
    이는 Nvidia가 GDDR 데이터 전송률을 처리하는 정상적인 방식입니다. 이에 맞게 오버클럭을 조정하십시오.

    잠금 클럭 옵션과 양수 클럭 속도 오프셋을 조합하여 의사 언더볼트를 구현할 수 있습니다.
    이를 통해 잠금 클럭에 의해 제한된 전압에서 GPU를 실행하면서, 오프셋으로 인해 더 높은 클럭 속도를 달성할 수 있습니다.
    지나치게 높이면 시스템 불안정을 유발할 수 있습니다.
oc-warning = 이 값들을 변경하면 시스템 불안정이 발생할 수 있으며 하드웨어가 손상될 가능성이 있습니다!
show-all-pstates = 모든 P-States 보기
enable-gpu-locked-clocks = GPU 잠금 클럭 활성화
enable-vram-locked-clocks = VRAM 잠금 클럭 활성화
pstate-list-description = <b>다음 값은 가장 높은 것부터 가장 낮은 것까지 각 P-State의 클럭 오프셋입니다.</b>
no-clocks-data = 클럭 데이터 없음
reset-oc-tooltip = 경고: 모든 클럭 설정이 기본값으로 초기화됩니다!
vf-curve-editor = VF 커브 편집기
nvidia-vf-curve-warning =
    전압-주파수 커브 편집기는 문서화되지 않은 드라이버 기능에 의존합니다.
    동작, 안정성 또는 사용 가능 여부에 대한 보장이 없습니다.
    <span weight = "heavy" underline = "single">사용에 따른 책임은 사용자에게 있습니다</span>.
vf-curve-enable-editing = 편집 활성화
voltage = 전압
frequency = 주파수
vf-active-curve = 활성 커브
vf-base-curve = 기본 커브
vf-curve-visible-range = 표시 범위 (%):
vf-curve-visible-range-to = ~
vf-curve-flatten-right = 커브를 오른쪽으로 평탄화
gpu-clock-offset = GPU 클럭 오프셋 (MHz)
max-gpu-clock = 최대 GPU 클럭 (MHz)
max-vram-clock = 최대 VRAM 클럭 (MHz)
max-gpu-voltage = 최대 GPU 전압 (mV)
min-gpu-clock = 최소 GPU 클럭 (MHz)
min-vram-clock = 최소 VRAM 클럭 (MHz)
min-gpu-voltage = 최소 GPU 전압 (mV)
gpu-voltage-offset = GPU 전압 오프셋 (mV)
gpu-pstate-clock-offset = GPU P-State { $pstate } 클럭 오프셋 (MHz)
vram-pstate-clock-offset = VRAM P-State { $pstate } 클럭 오프셋 (MHz)
gpu-pstate-clock = GPU P-State { $pstate } 클럭 (MHz)
mem-pstate-clock = VRAM P-State { $pstate } 클럭 (MHz)
gpu-pstate-clock-voltage = GPU P-State { $pstate } 전압 (mV)
mem-pstate-clock-voltage = VRAM P-State { $pstate } 전압 (mV)
pstates = 전력 상태
gpu-pstates = GPU 전력 상태
vram-pstates = VRAM 전력 상태
pstates-manual-needed = 전력 상태를 전환하려면 성능 레벨을 '수동'으로 설정해야 합니다
enable-pstate-config = 전력 상태 구성 활성화
menu = 메뉴
show-historical-charts = 그래프 표시
show-process-monitor = 프로세스 모니터 표시
generate-debug-snapshot = 디버그 스냅샷 생성
dump-vbios = VBIOS 덤프
reset-all-config = 전체 구성 초기화
stats-update-interval = 업데이트 간격 (ms)
historical-data-title = 기록 데이터
graphs-per-row = 행당 그래프 수:
time-period-seconds = 시간 범위 (초):
reset-all-graphs-tooltip = 모든 그래프를 기본값으로 초기화
add-graph = 그래프 추가
delete-graph = 그래프 삭제
edit-graphs = 편집
export-csv = CSV로 내보내기
edit-graph-sensors = 그래프 센서 편집
error-heading = 오류
daemon-info-heading = 데몬 정보
reconnecting-to-daemon = 데몬 연결이 끊어졌습니다. 재연결 중...
daemon-connection-lost = 연결 끊김
embedded-daemon-info =
    데몬에 연결할 수 없어 임베디드 모드로 실행합니다.
    lactd 서비스가 실행 중인지 확인하십시오.
    임베디드 모드에서는 설정을 변경할 수 없습니다.

    { $error_info }데몬을 활성화하려면 다음 명령을 실행한 후 LACT를 다시 시작하십시오:
version-mismatch = 버전 불일치
version-mismatch-description =
    GUI와 데몬 간 버전 불일치 ({ $gui_version }-{ $gui_commit } vs { $daemon_version }-{ $daemon_commit })!
    LACT를 업데이트한 경우 다음 명령으로 서비스를 다시 시작해야 합니다:
plot-show-detailed-info = 상세 정보 표시
settings-profile = 설정 프로파일
auto-switch-profiles = 자동 전환
add-profile = 새 프로파일 추가
import-profile = 파일에서 프로파일 가져오기
create-profile = 프로파일 생성
name = 이름
profile-copy-from = 설정 복사:
create = 생성
cancel = 취소
close = 닫기
save = 저장
default-profile = 기본값
rename-profile = 프로파일 이름 변경
rename-profile-from = 프로파일 <b>{ $old_name }</b>의 이름 변경:
delete-profile = 프로파일 삭제
edit-rules = 규칙 편집
edit-rule = 규칙 편집
remove-rule = 규칙 제거
profile-rules = 프로파일 규칙
export-to-file = 파일로 내보내기
move-up = 위로 이동
move-down = 아래로 이동
profile-activation = 활성화
profile-hooks = 훅
profile-activation-desc = 다음 조건에서 프로파일 '{ $name }' 활성화:
any-rules-matched = 다음 규칙 중 하나라도 일치할 때:
all-rules-matched = 다음 규칙이 모두 일치할 때:
activation-settings-status =
    선택한 활성화 설정은 현재 <b>{ $matched ->
        [true] 일치함
       *[false] 일치하지 않음
    }</b>
activation-auto-switching-disabled = 자동 프로파일 전환이 현재 비활성화되어 있습니다
profile-hook-command = 프로파일 '{ $cmd }'이(가) 다음 상태일 때 명령 실행:
profile-hook-activated = 활성화됨:
profile-hook-deactivated = 비활성화됨:
profile-hook-note = 참고: 이 명령은 LACT 데몬에 의해 루트 권한으로 실행되며, 데스크톱 환경에 접근할 수 없습니다. 따라서 GUI 응용 프로그램을 직접 실행하는 데 사용할 수 없습니다.
profile-rule-process-tab = 프로세스가 실행 중일 때
profile-rule-gamemode-tab = Gamemode가 활성화되어 있을 때
profile-rule-process-name = 프로세스 이름:
profile-rule-args-contain = 인수에 포함:
profile-rule-specific-process = 특정 프로세스가 있을 때:
theme = 테마
theme-auto = 자동
preferences = 환경 설정
ui = UI
daemon = 데몬
about = 정보
crash-page-title = 응용 프로그램 충돌
exit = 종료
displays-page = 정보 표시
display-title = 디스플레이 { $identifier }
display-manufacturer = 제조사
display-product-code = 제품 코드
display-model = 모델
display-physical-size = 실제 크기
display-connection = 연결
display-manufacture-date = 제조일
displays-missing = 표시장치가 탐색되지 않음
vf-curve-flatten-selection = 선택영역 평탄화
thresholds-section = 임계값; 한계
