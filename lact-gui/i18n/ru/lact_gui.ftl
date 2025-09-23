thermals-page = Температуры
oc-page = Разгон
platform-name = Имя платформы
instance = Инстанция
api-version = Версия API
info-page = Информация
lact-daemon = Сервис LACT
software-page = ПО
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
amd-oc-disabled =
    Разгон AMD выключен!
    Вы можете изменить базовые параметры, но управление частотами и напряжением будет недоступно.
gpu-clock-target = Целевая частота ядра GPU
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
zero-rpm = Режим нулевых об/мин (Zero RPM)
static-speed = Фиксированная скорость (%)
reset-button = Сброс
amd-oc = Разгон AMD
amd-oc-detected-system-config =
    Обнаружена конфигурация системы: <b>{ $config ->
        [unsupported] не поддерживается
       *[other] { $config }
    }</b>
enable-amd-oc-description = Функция разгона драйвера amdgpu будет включена путем создания файла по адресу <b>{ $path }</b> и обновления initramfs. Вы уверены, что хотите продолжить?
disable-amd-oc = Выключить разгон AMD
amd-oc-updating-configuration = Обновление конфигурации (может занять некоторое время)
amd-oc-updating-done = Конфигурация была обновлена, пожалуйста, перезагрузите систему для применения изменений.
watt = Вт
ghz = ГГц
mebibyte = МиБ
stats-section = Статистика
gpu-clock = Частота ядра GPU
gpu-clock-avg = Средняя частота ядра GPU
gpu-voltage = Напряжение GPU
gpu-usage = Использование GPU
power-usage = Потребляемая мощность
no-throttling = Нет
missing-stat = Н/Д
performance-level-high = Максимальные частоты
performance-level-low = Минимальные частоты
performance-level-low-description = Всегда использовать минимальные тактовые частоты для GPU и VRAM.
performance-level-manual-description = Ручное управление производительностью.
workgroup-size = Размер рабочей группы
features = Функции
cache-info = Информация о кэше
nvidia-cache-desc = { $size } L{ $level }
reset-config-description = Вы уверены, что хотите сбросить все настройки GPU?
zero-rpm-stop-temp = Выключение режима Zero RPM (°C)
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
        [rpm-ostree] Эта опция включит поддержку разгона AMD путем установки флагов запуска через  <b>rpm-ostree</b>.
        [unsupported]
            Текущая система не распознана как поддерживаемая для автоматической настройки разгона.
            Вы можете попробовать включить разгон через LACT, но для вступления изменений в силу может потребоваться ручная регенерация initramfs.
            Если это не сработает, альтернативно можно добавить параметр загрузки <b>amdgpu.ppfeaturemask=0xffffffff</b>  в загрузчик ОС.
       *[other] Эта опция включит разгон AMD путем создания файла в <b>{ $path }</b> и обновления initramfs.
    }

    См. <a href="https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)">вики</a> для получения дополнительной информации.
power-cap = Порог потребляемой мощности
gpu-temp = Температура
unknown-throttling = Неизвестно
vram-clock = Частота VRAM
performance-level-auto-description = Автоматическая регулировка частот GPU и VRAM (по умолчанию).
performance-level-high-description = Всегда использовать максимальные тактовые частоты для GPU и VRAM.
auto-page = Автоматически
performance-level-auto = Автоматически
performance-level-manual = Вручную
vram-pstates = Состояния питания VRAM
profile-hook-deactivated = Деактивирован:
power-profile-mode = Режим профиля питания:
overclock-section = Частота и напряжение
show-all-pstates = Показать все P-States
enable-gpu-locked-clocks = Включить фиксированные частоты GPU
gpu-clock-offset = Смещение частоты GPU (МГц)
max-vram-clock = Максимальная частота VRAM (МГц)
max-gpu-voltage = Максимальное напряжение GPU (мВ)
gpu-voltage-offset = Смещение напряжения GPU (мВ)
gpu-pstate-clock = Частота GPU для P-State { $pstate } (МГц)
mem-pstate-clock = Частота VRAM для P-State { $pstate } (МГц)
gpu-pstate-clock-voltage = Напряжение GPU для P-State { $pstate } (мВ)
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
nvidia-oc-info = Информация о разгоне Nvidia
min-gpu-clock = Минимальная частота GPU (МГц)
min-vram-clock = Минимальная частота VRAM (МГц)
gpu-pstate-clock-offset = Смещение частоты GPU для P-State { $pstate } (МГц)
vram-pstate-clock-offset = Смещение частоты VRAM для P-State { $pstate } (МГц)
gpu-pstates = Состояния питания GPU
rename-profile-from = Переименовать профиль <b>{ $old_name }</b>:
pstate-list-description = <b>Следующие значения являются смещениями частоты для каждого P-State, от самого высокого к самому низкому.</b>
max-gpu-clock = Максимальная частота GPU (МГц)
edit-rules = Изменить правила
export-to-file = Экспорт в файл
no-clocks-data = Данные о частотах недоступны
manual-level-needed = Чтобы использовать режимы питания, уровень производительности должен быть установлен на «вручную»
oc-warning = Внимание: изменение этих значений может привести к нестабильной работе системы, а также повредить ваше оборудование!
enable-vram-locked-clocks = Включить фиксированные частоты VRAM
profile-hook-command = Выполнить команду, когда профиль '{ $cmd }':
profile-hook-activated = Активирован:
nvidia-oc-description =
    Разгон на видеокартах Nvidia включает возможность задавать смещения для частот GPU и VRAM, а также ограничивать потенциальный диапазон частот с помощью функции «locked clocks» (фиксированные частоты).

    На многих моделях видеокарт смещение частоты видеопамяти фактически влияет на реальную скорость памяти только наполовину от заданного значения.
    Например, при установке смещения +1000 МГц прирост измеренной частоты VRAM может составить всего +500 МГц.
    Это нормальное поведение, связанное с тем, как Nvidia обрабатывает скорость передачи данных GDDR. Учитывайте это при настройке разгона.

    Прямое управление напряжением недоступно, так как оно отсутствует в драйвере Nvidia для Linux.

    Можно сделать «псевдо-андервольт» с помощью комбинации «locked clocks» и положительного смещения частоты.
    В этом случае GPU будет работать на напряжении, ограниченном фиксированными частотами, но при этом достигнет более высокой частоты за счет смещения.
    Чрезмерное увеличение параметров может привести к нестабильности системы.
import-profile = Импорт профиля из файла
reset-oc-tooltip = Внимание: все настройки частот будут сброшены к значениям по умолчанию!
auto-switch-profiles = Автоматическое переключение
add-profile = Добавить новый профиль
profile-activation = Активация
profile-activation-desc = Активировать профиль '{ $name }' при:
show-historical-charts = Показать графики сенсоров
move-down = Вниз
min-gpu-voltage = Минимальное напряжение GPU (мВ)
pstates-manual-needed = Примечание: уровень производительности должен быть установлен на «вручную» для переключения состояний питания
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
generate-debug-snapshot = Сгенерировать дебаг-лог
dump-vbios = Сделать дамп VBIOS
reset-all-config = Сбросить все настройки
stats-update-interval = Интервал обновления (мс)
reconnecting-to-daemon = Потеряно соединение с сервисом, переподключение...
daemon-connection-lost = Соединение потеряно
plot-show-detailed-info = Показать подробную информацию
show-process-monitor = Показать диспетчер задач
