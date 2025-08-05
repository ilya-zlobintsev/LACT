info-page = Informations
oc-page = OC
thermals-page = Températures
software-page = Logiciel
hardware-info = Informations sur le matériel
system-section = Système
kernel-version = Version du noyau
device-name = Nom du périphérique
version = Version
features = Fonctionnalités
extensions = Extensions
platform-name = Nom de la plateforme
instance = Instance
driver-name = Nom du pilote
lact-gui = Interface graphique de LACT
api-version = Version de l’API
driver-version = Version du pilote
compute-units = Unités de calcul
cl-c-version = Version C d’OpenCL
workgroup-size = Taille du groupe de travail
global-memory = Mémoire globale
lact-daemon = Démon LACT
local-memory = Mémoire locale
temperatures = Températures
reset-config-description = Voulez-vous vraiment réinitialiser toute la configuration GPU ?
throttling = Régulation
device-not-found = Périphérique { $kind } introuvable
cache-info = Informations sur le cache
nvidia-cache-desc = { $size } L{ $level }
monitoring-section = Surveillance
fan-speed = Vitesse des ventilateurs
acoustic-limit = Limite acoustique (tr/min)
acoustic-target = Cible acoustique (tr/min)
min-fan-speed = Vitesse minimale des ventilateurs (%)
zero-rpm = Zéro tr/min
static-speed = Vitesse statique (%)
enable-amd-oc = Activer l’overclocking AMD
disable-amd-oc = Désactiver l’overclocking AMD
disable-amd-oc-description = Cela aura pour effet de désactiver la prise en charge de l’overclocking AMD (ou overdrive) au prochain redémarrage.
power-cap = Limite de consommation électrique
watt = W
mhz = MHz
ghz = GHz
reset-config = Réinitialiser la configuration
fan-control-section = Contrôle des ventilateurs
oc-missing-fan-control-warning = Avertissement : La prise en charge de l’overclocking est désactivée, la fonctionnalité de contrôle des ventilateurs n’est donc pas disponible.
curve-page = Courbe
target-temp = Température cible (°C)
enable-amd-oc-description = Cela aura pour effet d’activer la fonctionnalité overdrive du pilote amdgpu en créant un fichier à l’emplacement <b>{ $path }</b> et en mettant à jour l’initramfs. Voulez-vous vraiment continuer ?
zero-rpm-stop-temp = Température d’arrêt à zéro tr/min (°C)
amd-oc-disabled =
    La prise en charge de l’overclocking AMD n’est pas activée !
    Vous pouvez toujours modifier les paramètres de base, mais les réglages avancés de fréquence et de tension ne seront pas disponibles.
pmfw-reset-warning = Attention : cette opération réinitialise les paramètres des ventilateurs du micrologiciel !
show-button = Afficher
cache-data = Données
cache-instruction = Données
cache-cpu = CPU
auto-page = Automatique
static-page = Statique
reset-button = Réinitialiser
mebibyte = Mio
stats-section = Statistiques
gpu-usage = Utilisation du GPU
gpu-voltage = Tension du GPU
gpu-temp = Température du GPU
vram-clock = Fréquence d’horloge VRAM
gpu-clock = Fréquence d’horloge cœur GPU
gpu-clock-avg = Fréquence d’horloge cœur GPU (moyenne)
gpu-clock-target = Fréquence d’horloge cœur GPU (cible)
power-usage = Consommation électrique
no-throttling = Aucune
unknown-throttling = Inconnue
performance-level-manual = Manuel
min-gpu-clock = Fréquence d’horloge GPU minimale (MHz)
enable-vram-locked-clocks = Activer le verrouillage d’horloges VRAM
oc-warning = Attention : la modification de ces valeurs peut entraîner une instabilité du système et potentiellement endommager votre matériel !
pstates-manual-needed = Remarque : Le niveau de performance doit être défini sur « manuel » pour basculer entre les états d’alimentation.
min-gpu-voltage = Tension GPU minimale (mV)
pstate-list-description = <b>Les valeurs suivantes correspondent à l'ajustement des fréquences d’horloge pour chaque P-State, du plus élevé au plus faible.</b>
min-vram-clock = Fréquence d’horloge VRAM minimale (MHz)
amd-cache-desc =
    Cache L{ $level } { $types } de { $size } { $shared ->
        [1] local à chaque CU
       *[other] partagé entre { $shared } CU
    }
missing-stat = N/A
performance-level-auto = Automatique
performance-level-auto-description = Ajuster automatiquement les fréquences d’horloge GPU et VRAM. (Par défaut)
power-profile-mode = Mode de profil d’alimentation :
overclock-section = Fréquence d’horloge et tension
nvidia-oc-info = Informations sur l’overclocking NVIDIA
show-all-pstates = Afficher tous les P-States
no-clocks-data = Aucune donnée de fréquence d’horloge disponible
gpu-clock-offset = Ajustement de la fréquence d’horloge GPU (MHz)
gpu-voltage-offset = Ajustement de la tension GPU (mV)
pstates = États de puissance
enable-pstate-config = Activer la configuration des états de puissance
settings-profile = Profil de paramètres
import-profile = Importer un profil à partir d’un fichier
reset-oc-tooltip = Attention : cette opération réinitialisera tous les réglages de fréquence d’horloge !
mem-pstate-clock-voltage = Tension pour le P-State VRAM { $pstate } (mV)
performance-level-high-description = Toujours utiliser les fréquences d’horloge GPU et VRAM maximales.
manual-level-needed = Le niveau de performance doit être défini sur « manuel » pour utiliser les états et modes d’alimentation
show-process-montor = Afficher le moniteur de processus
performance-level-high = Fréquences d’horloge les plus élevées
performance-level-low = Fréquences d’horloge les plus faibles
performance-level-low-description = Toujours utiliser les fréquences d’horloge GPU et VRAM minimales.
performance-level-manual-description = Contrôle manuel des performances.
max-gpu-clock = Fréquence d’horloge GPU maximale (MHz)
add-profile = Ajouter un nouveau profil
enable-gpu-locked-clocks = Activer le verrouillage d’horloges GPU
max-vram-clock = Fréquence d’horloge VRAM maximale (MHz)
max-gpu-voltage = Tension GPU maximale (mV)
vram-pstate-clock-offset = Ajustement de fréquence d’horloge pour le P-State VRAM { $pstate } (MHz)
gpu-pstate-clock = Fréquence d’horloge pour le P-State GPU { $pstate } (MHz)
mem-pstate-clock = Fréquence d’horloge pour le P-State VRAM { $pstate } (MHz)
gpu-pstate-clock-voltage = Tension pour le P-State GPU { $pstate } (mV)
gpu-pstates = États de puissance GPU
vram-pstates = États de puissance VRAM
show-historical-charts = Afficher les graphiques d’historique
auto-switch-profiles = Basculer automatiquement
nvidia-oc-description =
    La fonctionnalité d’overclocking NVIDIA comprend le réglage des fréquences d’horloge GPU/VRAM et la limitation de la plage potentielle des fréquences d’horloge à l’aide de la fonctionnalité « locked clocks ».

    Sur de nombreuses cartes, l’ajustement de la fréquence d’horloge VRAM n’affectera la fréquence d’horloge réelle de la mémoire que de la moitié de la valeur d’ajustement.
    Par exemple, un ajustement VRAM de +1000 MHz augmente la fréquence VRAM mesurée de seulement 500 MHz.
    Ce comportement est normal et est cohérent avec la manière dont NVIDIA gère les débits de données GDDR. Ajustez votre overclocking en conséquence.

    Le contrôle direct de la tension n’est pas pris en charge, car il n’existe pas dans le pilote Linux NVIDIA.

    Il est possible d’obtenir un pseudo-undervolt en combinant l’option « locked clocks » à un ajustement positif de la fréquence d’horloge.
    Cela forcera le GPU à fonctionner à une tension limitée par le verrouillage d’horloges, tout en atteignant une fréquence d’horloge plus élevée grâce à l’ajustement.
    Cela est susceptible de rendre le système instable si la valeur d’ajustement est trop élevée.
gpu-pstate-clock-offset = Ajustement de fréquence d’horloge pour le P-State GPU { $pstate } (MHz)
name = Nom
create = Créer
profile-copy-from = Copier les paramètres à partir de :
create-profile = Créer un profil
