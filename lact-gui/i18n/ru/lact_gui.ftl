thermals-page = Температуры
oc-page = Разгон
platform-name = Имя платформы
instance = Инстанция
api-version = Версия API
info-page = Сведения об оборудовании
lact-daemon = Сервис LACT
software-page = Сведения о ПО
hardware-info = Сведения об оборудовании
lact-gui = Графический интерфейс LACT
compute-units = Вычислительные блоки (Compute Units)
version = Версия
kernel-version = Версия ядра
device-name = Имя устройства
system-section = Система
monitoring-section = Мониторинг
amd-oc-status =
    Статус разгона AMD: <b>{ $status ->
        [true] вкл.
        [false] выкл.
       *[other] неизвестен
    }</b>
enable-amd-oc = Включить разгон AMD
min-fan-speed = Минимальная скорость вентиляторов (%)
amd-oc-disabled = Разгон AMD не задействован! <a href="https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)">Некоторые функции будут недоступны.</a>
gpu-clock-target = Целевая частота ядра ГП
local-memory = Локальная память
driver-name = Название драйвера
cl-c-version = Версия OpenCL C
global-memory = Глобальная память
extensions = Расширения
cache-instruction = Данные
cache-cpu = ЦП
temperatures = Температуры
fan-speed = Скорость вентиляторов
throttling = Троттлинг
static-page = Фиксированно
target-temp = Целевая температура (°C)
acoustic-target = Акустическая цель (об/мин)
zero-rpm = Режим нулевых оборотов (Zero RPM)
static-speed = Фиксированная скорость (%)
reset-button = Сброс
amd-oc = Разгон AMD
amd-oc-detected-system-config =
    Обнаружена конфигурация системы: <b>{ $config ->
        [unsupported] не поддерживается
       *[other] { $config }
    }</b>
enable-amd-oc-description = Функция разгона драйвера amdgpu будет включена путём создания файла по адресу <b>{ $path }</b> и обновления initramfs. Уверены, что хотите продолжить?
disable-amd-oc = Выключить разгон AMD
amd-oc-updating-configuration = Обновление конфигурации (может занять некоторое время)
amd-oc-updating-done = Конфигурация была обновлена, пожалуйста, перезагрузите систему для применения изменений.
watt = Вт
ghz = ГГц
mebibyte = МиБ
stats-section = Статистика
gpu-clock = Частота ядра ГП
gpu-clock-avg = Средняя частота ядра ГП
gpu-voltage = Напряжение ГП
gpu-usage = Использование ГП
power-usage = Потребляемая мощность
no-throttling = Нет
missing-stat = Н/Д
performance-level-high = Максимальные частоты
performance-level-low = Минимальные частоты
performance-level-low-description = Всегда использовать минимальные тактовые частоты для ГП и VRAM.
performance-level-manual-description = Ручное управление производительностью.
workgroup-size = Размер рабочей группы
features = Функции
cache-info = Информация о кэше
nvidia-cache-desc = { $size } L{ $level }
reset-config-description = Уверены, что хотите сбросить все настройки ГП?
zero-rpm-stop-temp = Температура остановки нулевых оборотов (°C)
show-button = Показать
disable-amd-oc-description = Разгон AMD будет выключен при следующей перезагрузке.
fan-control-section = Настройка вентиляторов
driver-version = Версия драйвера
cache-data = Данные
oc-missing-fan-control-warning = Внимание: разгон не включен, настройка вентиляторов недоступна.
mhz = МГц
device-not-found = Устройство { $kind } не было найдено
curve-page = Кривая
acoustic-limit = Акустический порог (об/мин)
pmfw-reset-warning = Внимание: настройки прошивки вентиляторов будут сброшены!
reset-config = Сбросить конфигурацию
amd-oc-description =
    { $config ->
        [rpm-ostree] Эта опция включит поддержку разгона AMD путём установки флагов запуска через  <b>rpm-ostree</b>.
        [unsupported]
            Текущая система не распознана как поддерживаемая для автоматической настройки разгона.
            Вы можете попробовать включить разгон через LACT, но для вступления изменений в силу может потребоваться ручная регенерация initramfs.
            Если это не сработает, альтернативно можно добавить параметр загрузки <b>amdgpu.ppfeaturemask=0xffffffff</b>  в загрузчик ОС.
       *[other] Эта опция включит разгон AMD путём создания файла в <b>{ $path }</b> и обновления initramfs.
    }

    См. <a href="https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)">вики</a> для получения дополнительной информации.
power-cap = Порог энергопотребления
gpu-temp = Температура
unknown-throttling = Неизвестно
vram-clock = Частота VRAM
performance-level-auto-description = Автоматическая регулировка частот ГП и VRAM (по умолчанию).
performance-level-high-description = Всегда использовать максимальные тактовые частоты для ГП и VRAM.
auto-page = Автоматически
performance-level-auto = Автоматически
performance-level-manual = Вручную
vram-pstates = Состояния питания VRAM
profile-hook-deactivated = Деактивирован:
power-profile-mode = Режим профиля питания:
overclock-section = Частота и напряжение
show-all-pstates = Показать все P-States
enable-gpu-locked-clocks = Включить фиксированные частоты ГП
gpu-clock-offset = Смещение частоты ГП (МГц)
max-vram-clock = Максимальная частота VRAM (МГц)
max-gpu-voltage = Максимальное напряжение ГП (мВ)
gpu-voltage-offset = Смещение напряжения ГП (мВ)
gpu-pstate-clock = Частота ГП для P-State { $pstate } (МГц)
mem-pstate-clock = Частота VRAM для P-State { $pstate } (МГц)
gpu-pstate-clock-voltage = Напряжение ГП для P-State { $pstate } (мВ)
mem-pstate-clock-voltage = Напряжение VRAM для P-State { $pstate } (мВ)
pstates = Состояния питания (P-States)
enable-pstate-config = Включить настройку состояний питания
settings-profile = Профиль настроек
create-profile = Создать профиль
profile-copy-from = Скопировать настройки:
rename-profile = Переименовать профиль
delete-profile = Удалить профиль
profile-rules = Правила профиля
move-up = Вверх
any-rules-matched = Соответствие любому из следующих правил:
all-rules-matched = Соответствие всем следующим правилам:
activation-auto-switching-disabled = Автоматическое переключение профилей в данный момент отключено
profile-rule-process-tab = Процесс запущен
profile-rule-gamemode-tab = Игровой режим (gamemode) активен
profile-rule-process-name = Имя процесса:
profile-rule-args-contain = Аргументы содержат:
profile-rule-specific-process = С определенным процессом:
nvidia-oc-info = Информация о разгоне
min-gpu-clock = Минимальная частота ГП (МГц)
min-vram-clock = Минимальная частота VRAM (МГц)
gpu-pstate-clock-offset = Смещение частоты ГП для P-State { $pstate } (МГц)
vram-pstate-clock-offset = Смещение частоты VRAM для P-State { $pstate } (МГц)
gpu-pstates = Состояния питания ГП
rename-profile-from = Переименовать профиль <b>{ $old_name }</b>:
pstate-list-description = <b>Следующие значения являются смещениями частоты для каждого P-State, от самого высокого к самому низкому.</b>
max-gpu-clock = Максимальная частота ГП (МГц)
edit-rules = Изменить правила
export-to-file = Экспорт в файл
no-clocks-data = Данные о частотах недоступны
manual-level-needed = Чтобы использовать режимы питания, уровень производительности должен быть установлен на «вручную»
oc-warning = Изменение этих значений может привести к нестабильной работе системы, а также повредить ваше аппаратное обеспечение!
enable-vram-locked-clocks = Включить фиксированные частоты VRAM
profile-hook-command = Выполнить команду, когда профиль '{ $cmd }':
profile-hook-activated = Активирован:
nvidia-oc-description =
    Разгон на видеокартах Nvidia включает возможность задавать смещения для частот ГП и VRAM, а также ограничивать потенциальный диапазон частот с помощью функции «locked clocks» (фиксированные частоты).

    На многих моделях видеокарт смещение частоты видеопамяти фактически влияет на реальную скорость памяти только наполовину от заданного значения.
    Например, при установке смещения +1000 МГц прирост измеренной частоты VRAM может составить всего +500 МГц.
    Это нормальное поведение, связанное с тем, как Nvidia обрабатывает скорость передачи данных GDDR. Учитывайте это при настройке разгона.

    Можно сделать «псевдо-андервольт» с помощью комбинации «locked clocks» и положительного смещения частоты.
    В этом случае ГП будет работать на напряжении, ограниченном фиксированными частотами, но при этом достигнет более высокой частоты за счёт смещения.
    Чрезмерное увеличение параметров может привести к нестабильности системы.
import-profile = Импорт профиля из файла
reset-oc-tooltip = Внимание: все настройки частот будут сброшены к значениям по умолчанию!
auto-switch-profiles = Автоматическое переключение
add-profile = Добавить новый профиль
profile-activation = Активация
profile-activation-desc = Активировать профиль '{ $name }' при:
show-historical-charts = Показать графики
move-down = Вниз
min-gpu-voltage = Минимальное напряжение ГП (мВ)
pstates-manual-needed = Уровень производительности должен быть установлен на «вручную» для переключения состояний питания
profile-hooks = Хуки
activation-settings-status =
    Выбранные настройки активации в данный момент <b>{ $matched ->
        [true] совпадают
       *[false] не совпадают
    }</b>
profile-hook-note = Примечание: эти команды выполняются сервисом LACT с правами root, они не имеют доступа к графической среде рабочего стола, поэтому их нельзя использовать для запуска графических приложений.
default-profile = По умолчанию
remove-rule = Удалить правило
name = Имя
create = Создать
edit-rule = Изменить правило
save = Сохранить
cancel = Отмена
amd-cache-desc =
    Кэш L{ $level } { $types } размером { $size } { $shared ->
        [1] локальный для каждого CU
       *[other] общий между { $shared } CU
    }
generate-debug-snapshot = Создать отладочный снимок
dump-vbios = Сохранить дамп VBIOS
reset-all-config = Сбросить все настройки
stats-update-interval = Период обновления (мс)
reconnecting-to-daemon = Потеряно соединение с сервисом, переподключение...
daemon-connection-lost = Соединение потеряно
plot-show-detailed-info = Показать подробную информацию
show-process-monitor = Показать диспетчер задач
temperature-sensor = Датчик температуры
spindown-delay = Задержка снижения оборотов (мс)
speed-change-threshold = Порог изменения скорости (°C)
automatic-mode-threshold = Порог автоматического режима (°C)
spindown-delay-tooltip = Время удержания температуры ГП ниже порога перед снижением оборотов вентилятора
automatic-mode-threshold-tooltip =
    Переключает управление вентиляторами в автоматический режим, когда температура опускается ниже указанного значения.

    Многие видеокарты Nvidia поддерживают полную остановку вентилятора (Zero RPM) только в автоматическом режиме управления, тогда как при использовании пользовательской кривой скорость вращения ограничена, например, диапазоном 30-100 %.

    Эта настройка позволяет обойти это ограничение: при температуре выше заданного порога используется пользовательская кривая, а при более низкой — автоматический режим видеокарты с поддержкой режима Zero RPM.
revert-button = Сбросить
vram-usage = Использование VRAM:
performance-level = Уровень производительности
historical-data-title = История показаний сенсоров
graphs-per-row = Графиков в строке:
reset-all-graphs-tooltip = Сбросить все графики к значениям по умолчанию
add-graph = Добавить график
delete-graph = Удалить график
export-csv = Экспортировать в CSV
edit-graph-sensors = Редактировать сенсоры графика
apply-button = Применить
edit-graphs = Редактировать
time-period-seconds = Временной промежуток (сек.):
theme = Тема
theme-auto = Автоматическая
crash-page-title = Сбой приложения
exit = Выход
hw-ip-info = Информация о IP аппаратного обеспечения
hw-queues = Очереди
bytes = байты
kibibyte = КиБ
gibibyte = ГиБ
confirm = Подтвердить
confirm-settings = Подтвердить настройки
settings-confirmation = Сохранить новые настройки? (Возврат через { $seconds_left } с)
vf-curve-editor = Редактор кривой напряжение-частота
nvidia-vf-curve-warning =
    Редактор кривой напряжение-частота полагается на недокументированную функциональность драйвера.
    Никаких гарантий относительно его поведения, безопасности или доступности не даётся.
    <span weight = "heavy" underline = "single">Используйте на свой страх и риск</span>.
vf-curve-enable-editing = Включить правку
voltage = Напряжение
frequency = Частота
vf-active-curve = Активная кривая
vf-base-curve = Базовая кривая
vf-curve-visible-range = Видимый диапазон (%):
vf-curve-visible-range-to = до
vf-curve-flatten-right = Выровнять кривую вправо
menu = Меню
error-heading = Ошибка
daemon-info-heading = Информация о демоне
embedded-daemon-info =
    Не удалось подключиться к демону, работа во встроенном режиме.
    Пожалуйста, убедитесь, что служба lactd запущена.
    Во встроенном режиме вы не сможете изменять настройки.

    { $error_info }Чтобы включить демон, выполните следующую команду, затем перезапустите LACT:
version-mismatch = Несоответствие версий
version-mismatch-description =
    Несоответствие версий между графическим интерфейсом и демоном ({ $gui_version }-{ $gui_commit } против { $daemon_version }-{ $daemon_commit })!
    Если вы обновили LACT, вам нужно перезапустить службу с помощью:
close = Закрыть
preferences = Настройки
ui = Интерфейс
daemon = Демон
about = О программе
