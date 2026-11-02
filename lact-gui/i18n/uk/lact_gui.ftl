info-page = Інформація
oc-page = Розгін
thermals-page = Температура
software-page = Програми
hardware-info = Інформація про обладнання
watt = Вт
system-section = Система
device-name = Назва пристрою
version = Версія
show-button = Показати
cache-instruction = Дані
cache-cpu = Процесор
auto-page = Автоматично
lact-daemon = Демон LACT
lact-gui = Графічний інтерфейс LACT
kernel-version = Версія ядра
instance = Екземпляр
compute-units = Обчислювальні одиниці
platform-name = Назва Платформи
api-version = Версія API
driver-name = Назва драйвера
driver-version = Версія Драйвера
cl-c-version = Версія OpenCL C
workgroup-size = Розмір робочої групи
global-memory = Глобальна пам'ять
local-memory = Локальна пам'ять
features = Характеристики
extensions = Розширення
device-not-found = { $kind } прилад не знайдено
cache-info = Інформація про кеш
amd-cache-desc =
    { $size } L{ $level } { $types } кеш { $shared ->
        [1] локальний для кожного CU
       *[other] спільний між { $shared } CU
    }
nvidia-cache-desc = { $size } L{ $level }
cache-data = Дані
monitoring-section = Моніторинг
fan-control-section = Управління вентилятором
temperatures = Температури
oc-missing-fan-control-warning = Увага: Підтримка розгону вимкнена, функція керування вентиляторами недоступна.
fan-speed = Швидкість вентилятора
throttling = Дроселювання
curve-page = Крива
static-page = Статичний
target-temp = Цільова температура (°C)
acoustic-limit = Акустична межа (об/хв)
acoustic-target = Акустична ціль (об/хв)
min-fan-speed = Мінімальна швидкість вентилятора (%)
zero-rpm = Нульова швидкість обертання
zero-rpm-stop-temp = Температура зупинки при нульових обертах (°C)
static-speed = Статична швидкість (%)
reset-button = Скинути
pmfw-reset-warning = Увага: це скидає налаштування прошивки вентилятора!
temperature-sensor = Датчик температури
spindown-delay = Затримка спін-дауна (мс)
spindown-delay-tooltip = Як довго графічний процесор повинен залишатися на нижчому значенні температури, перш ніж зменшити швидкість вентилятора
speed-change-threshold = Поріг зміни швидкості (°C)
automatic-mode-threshold = Поріг автоматичного режиму (°C)
automatic-mode-threshold-tooltip =
    Перемикайте керування вентилятором в автоматичний режим, коли температура падає нижче цієї точки.

    Багато графічних процесорів Nvidia підтримують зупинку вентилятора лише в режимі автоматичного керування, тоді як користувацька крива має обмежений діапазон швидкості, наприклад, 30-100%.

    Ця опція дозволяє обійти це обмеження, використовуючи користувацьку криву лише тоді, коли температура перевищує певну, а вбудований автоматичний режим карти, який підтримує нульову швидкість обертання, використовуватиметься нижче неї.
amd-oc = Розгін AMD
amd-oc-disabled =
    Підтримка розгону AMD не ввімкнена!
    Ви все ще можете змінювати основні налаштування, але більш просунуті функції керування тактовою частотою та напругою будуть недоступні.
amd-oc-status =
    Розгін AMD наразі: <b>{ $status ->
        [true] Увімкнено
        [false] Вимкнено
       *[other] Невідомо
    }</b>
amd-oc-detected-system-config =
    Виявлена конфігурація системи: <b>{ $config ->
        [unsupported] Не підтримується
       *[other] { $config }
    }</b>
