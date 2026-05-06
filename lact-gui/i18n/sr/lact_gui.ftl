info-page = Подаци о хардверу
oc-page = Оверклоковање
thermals-page = Температуре
software-page = Подаци о софтверу
hardware-info = Подаци о хардверу
system-section = Систем
lact-daemon = ЛАКТ позадинац
lact-gui = ЛАКТ прочеље
kernel-version = Издање језгра
instance = Примерак
device-name = Назив уређаја
platform-name = Назив платформе
api-version = Издање АПИ-ја
version = Издање
driver-name = Назив гонича
driver-version = Издање гонича
compute-units = Јединице прорачуна
cl-c-version = ОпенЦЛ Це издање
workgroup-size = Величина радне групе
global-memory = Општа меморија
local-memory = Локална меморија
features = Могућности
extensions = Проширења
show-button = Прикажи
device-not-found = { $kind } уређај није пронађен
cache-info = Подаци о кешу
hw-ip-info = Подаци хардверског ИП-ја
hw-queues = Редови
cache-data = Подаци
cache-instruction = Подаци
cache-cpu = Процесор
monitoring-section = Надгледање
fan-control-section = Управљање вентилатором
temperatures = Температуре
fan-speed = Брзина вентилатора
throttling = Успоравање
auto-page = Самостално
curve-page = Крива
static-page = Статичка
target-temp = Жељена температура (°C)
acoustic-limit = Акустичко ограничење (РПМ)
oc-missing-fan-control-warning = Упозорење: подршка за оверклоковање је искључена, управљање вентилатором није доступно.
acoustic-target = Акустички циљ (РПМ)
min-fan-speed = Најмања брзина вентилатора (%)
zero-rpm = Нулти РПМ
zero-rpm-stop-temp = Температура заустављања нултог РПМ-а (°C)
static-speed = Статичка брзина (%)
reset-button = Врати
pmfw-reset-warning = Упозорење: ово враћа фирмверска подешавања вентилатора!
temperature-sensor = Сензор температуре
gpu-clock = ГПЈ такт језгра
nvidia-cache-desc = { $size } Л{ $level }
spindown-delay = Померај заустављања (мс)
spindown-delay-tooltip = Колико дуго ГПЈ треба бити на нижој температури пре успоравања вентилатора
amd-oc = АМД оверклоковање
amd-oc-disabled = АМД оверклоковање није омогућено! <a href="https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)">Неке могућности неће бити доступне.</a>
amd-oc-status =
    АМД оверклоковање је тренутно: <b>{ $status ->
        [true] Омогућено
        [false] Онемогућено
       *[other] Непознато
    }</b>
amd-cache-desc =
    { $size } L{ $level } { $types } кеш { $shared ->
        [1] локалан за сваку CU
       *[other] дељен између { $shared } CU-ова
    }
speed-change-threshold = Праг промене брзине (°C)
automatic-mode-threshold = Праг самосталног режима (°C)
automatic-mode-threshold-tooltip =
    Пребаците управљање вентилатором у самостални режим када је температура испод ове тачке.

    Многе Енвидија графичке карте подржавају заустављање вентилатора само у режиму самосталног управљања, док прилагођена крива има ограничен опсег брзине, као што је 30-100%.

    Ова могућност омогућава заобилажење овог ограничења тако што се прилагођена крива користи само изнад одређене температуре, док се испод ње користи уграђени самостални режим карте који подржава 0 РПМ.
amd-oc-detected-system-config =
    Детектована поставка система: <b>{ $config ->
        [unsupported] Неподржано
       *[other] { $config }
    }</b>
amd-oc-description =
    { $config ->
        [rpm-ostree] Ова могућност ће окинути подршку за АМД овердрајв постављањем подизних заставица преко <b>rpm-ostree</b>.
        [unsupported]
            Тренутни систем није препознат као подржан за самостално подешавање овердрајва.
            Можете покушати да омогућите оверклоковање из ЛАКТ-а, али може бити потребно ручно регенерисање initramfs-а да би оно ступило на снагу.
            Ако то не успе, резервна могућност је да додате <b>amdgpu.ppfeaturemask=0xffffffff</b> као подизни параметар у вашем подизачу система.
       *[other] Ова могућност ће окинути подршку за АМД овердрајв прављењем датотеке на <b>{ $path }</b> и ажурирањем initramfs-а.
    }

    Погледајте <a href="https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)">вики</a> за више података.
enable-amd-oc-description = Ово ће омогућити функцију овердрајва гонича amdgpu прављењем датотеке на <b>{ $path }</b> и ажурирањем initramfs-а. Да ли сте сигурни да желите ово да урадите?
disable-amd-oc = Онемогући АМД оверклоковање
enable-amd-oc = Омогући АМД оверклоковање
disable-amd-oc-description = Ово ће онемогућити подршку за АМД оверклоковање (overdrive) при наредном поновном покретању.
amd-oc-updating-configuration = Ажурирање поставки (ово може потрајати)
amd-oc-updating-done = Поставке су ажуриране, поново покрените рачунар да бисте применили изменења.
reset-config = Врати поставке
reset-config-description = Да ли сте сигурни да желите да вратите све поставке графичке картице?
apply-button = Примени
revert-button = Врати
power-cap = Ограничење употребе напајања
watt = W
ghz = GHz
mhz = MHz
bytes = бајтова
kibibyte = KiB
mebibyte = MiB
gibibyte = GiB
stats-section = Статистика
gpu-clock-avg = ГПЈ такт језгра (просек)
gpu-clock-target = ГПЈ такт језгра (циљ)
gpu-voltage = ГПЈ напон
gpu-temp = Температура
gpu-usage = Употреба ГПЈ
vram-clock = ВРАМ такт
power-usage = Потрошња енергије
no-throttling = Не
unknown-throttling = Непознато
missing-stat = Недоступно
vram-usage = Употреба ВРАМ-а:
performance-level-auto = Самостално
performance-level-high = Највиши тактови
performance-level-low = Најнижи тактови
performance-level-manual = Ручно
performance-level-auto-description = Самостално прилагођава тактове ГПЈ-а и ВРАМ-а. (Подразумевано)
performance-level-high-description = Увек користи највише брзине тактова за ГПЈ и ВРАМ.
performance-level-low-description = Увек користи најниже брзине тактова за ГПЈ и ВРАМ.
performance-level-manual-description = Ручно управљање учинком.
performance-level = Ниво учинка
power-profile-mode = Режим профила напајања:
manual-level-needed = Ниво учинка мора бити постављен на „ручно“ да би се користили стања и режими напајања
overclock-section = Брзина такта и напона
nvidia-oc-info = Подаци о оверклоковању
nvidia-oc-description =
    Функционалност оверклоковања на Енвидији укључује постављање помераја за тактовне брзине ГПЈ/ВРАМ-а и ограничавање потенцијалног опсега тактовних брзина помоћу функције „закључаних тактова“.

    На многим картицама, померај тактовне брзине ВРАМ-а ће утицати на стварну тактовну брзину меморије само за пола вредности помераја.
    На пример, померај ВРАМ-а од +1000MHz може повећати измерену брзину ВРАМ-а за само 500MHz.
    Ово је нормално и тако Енвидија рукује брзинама података GDDR-а. Прилагодите свој оверклок у складу са тим.

    Могуће је постићи псеудо-поднапон комбиновањем могућности закључаних тактова са позитивним померајем тактовне брзине.
    Ово ће приморати ГПЈ да ради на напону који је ограничен закључаним тактовима, док истовремено постиже вишу тактовну брзину захваљујући померају.
    Ово може изазвати нестабилност система ако се превише подигне.
oc-warning = Промена ових вредности може довести до нестабилности система и потенцијално оштетити ваш хардвер!
show-all-pstates = Прикажи сва П-стања
enable-gpu-locked-clocks = Омогући закључане тактове ГПЈ-а
enable-vram-locked-clocks = Омогући закључане тактове ВРАМ-а
pstate-list-description = <b>Следеће вредности су одступања тактова за свако П-стање, од највишег до најнижег.</b>
no-clocks-data = Нема доступних података о тактовима
reset-oc-tooltip = Упозорење: ово враћа сва подешавања тактова на подразумеване вредности!
vf-curve-editor = Уређивач VF криве
nvidia-vf-curve-warning =
    Уређивач криве напона-учесталости ослања се на недокументовану функционалност гонича .
    Нема гаранција у вези са његовим понашањем, безбедношћу или доступношћу.
    <span weight = "heavy" underline = "single">Користите на сопствени ризик</span>.
vf-curve-enable-editing = Омогући уређивање
voltage = Напон
frequency = Учесталост
vf-active-curve = Активна крива
vf-base-curve = Основна крива
vf-curve-visible-range = Видљиви опсег (%):
vf-curve-visible-range-to = до
vf-curve-flatten-right = Изравнај криву десно
gpu-clock-offset = Померај такта графичке (MHz)
max-gpu-clock = Највећи такт графичке (MHz)
max-vram-clock = Највећи такт ВРАМ-а (MHz)
max-gpu-voltage = Највећи напон графичке (mV)
min-gpu-clock = Најмањи такт графичке (MHz)
min-vram-clock = Најмањи такт ВРАМ-а (MHz)
min-gpu-voltage = Најмањи напон графичке (mV)
gpu-voltage-offset = Померај напона графичке (mV)
gpu-pstate-clock-offset = Померај такта П-стања ГПЈ-а { $pstate } (MHz)
vram-pstate-clock-offset = Померај такта П-стања ВРАМ-а { $pstate } (MHz)
gpu-pstate-clock = Такт П-стања ГПЈ-а { $pstate } (MHz)
mem-pstate-clock = Такт П-стања ВРАМ-а { $pstate } (MHz)
gpu-pstate-clock-voltage = Напон П-стања ГПЈ-а { $pstate } (mV)
mem-pstate-clock-voltage = Напон П-стања ВРАМ-а { $pstate } (mV)
pstates = Стања напајања
gpu-pstates = Стања напајања ГПЈ-а
vram-pstates = Стања напајања ВРАМ
pstates-manual-needed = Ниво учинка мора бити постављен на „ручно“ да би се окинула стања напајања
enable-pstate-config = Омогући поставку стања напајања
show-historical-charts = Прикажи графике
show-process-monitor = Прикажи монитор процеса
generate-debug-snapshot = Направи снимак стања за отклањање грешака
dump-vbios = Избаци ВБИОС
reset-all-config = Врати све поставке
stats-update-interval = Период освежавања (мс)
historical-data-title = Историјски подаци
graphs-per-row = Графици по реду:
time-period-seconds = Временски период (секунде):
reset-all-graphs-tooltip = Врати све графике на подразумевано
add-graph = Додај график
delete-graph = Обриши график
edit-graphs = Уреди
export-csv = Извези као CSV
edit-graph-sensors = Уреди сензоре графика
reconnecting-to-daemon = Веза са позадинацем је изгубљена, поново се повезујем...
daemon-connection-lost = Веза је изгубљена
plot-show-detailed-info = Прикажи детаљне информације
settings-profile = Профил подешавања
auto-switch-profiles = Самостално пребаци
add-profile = Додај нови профил
import-profile = Увези профил из датотеке
create-profile = Направи профил
name = Назив
profile-copy-from = Копирај подешавања из:
create = Направи
cancel = Откажи
save = Сачувај
default-profile = Подразумевано
rename-profile = Преименуј профил
rename-profile-from = Преименуј профил <b>{ $old_name }</b> у:
delete-profile = Обриши профил
edit-rules = Уреди правила
edit-rule = Уреди правило
remove-rule = Уклони правило
profile-rules = Правила профила
export-to-file = Извези у датотеку
move-up = Помери горе
move-down = Помери доле
profile-activation = Покретање
profile-hooks = Куке
profile-activation-desc = Покрени профил „{ $name }“ када:
any-rules-matched = Било које од следећих правила су испуњена:
all-rules-matched = Сва следећа правила су испуњена:
activation-settings-status =
    Изабрана подешавања покретања су тренутно <b>{ $matched ->
        [true] испуњена
       *[false] нису испуњена
    }</b>
activation-auto-switching-disabled = Самостално пребацивање профила је тренутно онемогућено
profile-hook-command = Покрени наредбу када је профил „{ $cmd }“:
profile-hook-activated = Покренута:
profile-hook-deactivated = Заустављена:
profile-hook-note = Напомена: ове наредбе ЛАКТ позадинац извршава као root, и оне немају приступ стоном окружењу. Због тога се не могу директно користити за покретање графичких програма.
profile-rule-process-tab = Процес је покренут
profile-rule-gamemode-tab = Режим игре је покренут
profile-rule-process-name = Назив процеса:
profile-rule-args-contain = Аргументи садрже:
profile-rule-specific-process = Са одређеним процесом:
theme = Тема
theme-auto = Самостална
preferences = Поставке
ui = КП
daemon = Позадинац
about = О програму
crash-page-title = Програм је пао
exit = Изађи
