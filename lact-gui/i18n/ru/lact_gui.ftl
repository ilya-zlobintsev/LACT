thermals-page = Датчики
oc-page = Разгон
platform-name = Название платформы
instance = Инстанция
api-version = Версия API
info-page = Информация
lact-daemon = Демон LACT
software-page = ПО
hardware-info = Сведения об оборудовании
lact-gui = Графический интерфейс LACT
compute-units = Compute Units
version = Версия
kernel-version = Версия ядра
device-name = Название устройства
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
static-page = Статично
target-temp = Целевая температура (°C)
acoustic-target = Акустическая цель (об/мин)
zero-rpm = Ноль об/мин (Zero RPM)
static-speed = Статичная скорость (%)
reset-button = Сброс
amd-oc = Разгон AMD
amd-oc-detected-system-config =
    Обнаружена конфигурация системы: <b>{ $config ->
        [unsupported] не поддерживается
       *[other] { $config }
    }</b>
enable-amd-oc-description = Это включит функцию разгона драйвера amdgpu, создав файл по адресу <b>{ $path }</b> и обновив initramfs. Вы уверены, что хотите продолжить?
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
workgroup-size = Размер группы
features = Функции
cache-info = Информация о кэше
nvidia-cache-desc = { $size } L{ $level }
reset-config-description = Вы уверены, что хотите сбросить все настройки ГП?
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
gpu-temp = Температура ГП
unknown-throttling = Неизвестно
vram-clock = Частота VRAM
performance-level-auto-description = Автоматическая регулировка частот ГП и VRAM (по умолчанию).
performance-level-high-description = Всегда использовать максимальные тактовые частоты для GPU и VRAM.
auto-page = Автоматически
performance-level-auto = Автоматически
performance-level-manual = Вручную
