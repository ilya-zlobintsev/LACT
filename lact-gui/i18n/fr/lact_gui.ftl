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
gpu-temp = Température
vram-clock = Fréquence d’horloge VRAM
gpu-clock = Fréquence d’horloge cœur GPU
gpu-clock-avg = Fréquence d’horloge cœur GPU (moyenne)
gpu-clock-target = Fréquence d’horloge cœur GPU (cible)
power-usage = Consommation électrique
no-throttling = Aucune
unknown-throttling = Inconnue
performance-level-manual = Manuel
min-gpu-clock = Fréquence d’horloge GPU minimale (MHz)
enable-vram-locked-clocks = Activer VRAM Locked Clocks
oc-warning = Avertissement : la modification de ces valeurs peut entraîner une instabilité du système et endommager votre matériel !
pstates-manual-needed = Remarque : le niveau de performance doit être réglé sur « manuel » pour basculer entre les états d’alimentation
min-gpu-voltage = Tension GPU minimale (mV)
pstate-list-description = <b>Les valeurs suivantes correspondent à l’ajustement des fréquences d’horloge pour chaque P-State, du plus élevé au plus faible.</b>
min-vram-clock = Fréquence d’horloge VRAM minimale (MHz)
amd-cache-desc =
    Cache L{ $level } { $types } de { $size } { $shared ->
        [1] local à chaque CU
       *[other] partagé entre { $shared } CU
    }
missing-stat = N/A
performance-level-auto = Automatique
performance-level-auto-description = Ajuster automatiquement les fréquences d’horloge GPU et VRAM. (Par défaut)
power-profile-mode = Mode profil d’alimentation :
overclock-section = Fréquence d’horloge et tension
nvidia-oc-info = Informations sur Nvidia Overclocking
show-all-pstates = Afficher tous les P-States
no-clocks-data = Aucune donnée de fréquence d’horloge disponible
gpu-clock-offset = Décalage de la fréquence d’horloge GPU (MHz)
gpu-voltage-offset = Décalage de la tension GPU (mV)
pstates = États de puissance
enable-pstate-config = Activer la configuration des états de puissance
settings-profile = Profil de paramètres
import-profile = Importer un profil à partir d’un fichier
reset-oc-tooltip = Avertissement : cette opération réinitialise tous les réglages de fréquence d’horloge aux valeurs par défaut !
mem-pstate-clock-voltage = P-State VRAM { $pstate } Tension (mV)
performance-level-high-description = Toujours utiliser les fréquences d’horloge les plus élevées pour le GPU et la VRAM.
manual-level-needed = Le niveau de performance doit être réglé sur « manuel » pour utiliser les états et modes d’alimentation
performance-level-high = Fréquences d’horloge les plus élevées
performance-level-low = Fréquences d’horloge les plus faibles
performance-level-low-description = Toujours utiliser les fréquences d’horloge les plus faibles pour le GPU et la VRAM.
performance-level-manual-description = Contrôle manuel des performances.
max-gpu-clock = Fréquence d’horloge GPU maximale (MHz)
add-profile = Ajouter un nouveau profil
enable-gpu-locked-clocks = Activer GPU Locked Clocks
max-vram-clock = Fréquence d’horloge VRAM maximale (MHz)
max-gpu-voltage = Tension GPU maximale (mV)
vram-pstate-clock-offset = P-State VRAM { $pstate } Décalage de fréquence d’horloge (MHz)
gpu-pstate-clock = P-State GPU { $pstate } Fréquence d’horloge (MHz)
mem-pstate-clock = P-State VRAM { $pstate } Fréquence d’horloge (MHz)
gpu-pstate-clock-voltage = P-State GPU { $pstate } Tension (mV)
gpu-pstates = États de puissance GPU
vram-pstates = États de puissance VRAM
show-historical-charts = Afficher les graphiques historiques
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
gpu-pstate-clock-offset = P-State GPU { $pstate } Décalage de fréquence d’horloge (MHz)
name = Nom
create = Créer
profile-copy-from = Copier les paramètres à partir de :
create-profile = Créer un profil
any-rules-matched = Une des règles suivantes est satisfaite :
cancel = Annuler
save = Enregistrer
rename-profile = Renommer le profil
delete-profile = Supprimer le profil
edit-rules = Modifier les règles
remove-rule = Supprimer une règle
profile-rules = Règles du profil
export-to-file = Exporter vers un fichier
move-down = Se déplacer vers le bas
profile-activation = Activation
profile-hooks = Hooks
profile-hook-command = Exécuter une commande lorsque le profil "{ $cmd }" est :
profile-hook-activated = Activé :
profile-hook-deactivated = Désactivé :
profile-rule-process-tab = Un processus est en cours d’exécution
profile-rule-args-contain = Les arguments contiennent :
profile-rule-specific-process = Avec un processus spécifique :
profile-activation-desc = Activer le profil "{ $name }" lorsque :
profile-hook-note = Remarque : Ces commandes sont exécutées en tant que root par le démon LACT, et n’ont pas accès à l’environnement de bureau. Par conséquent, elles ne peuvent pas être directement utilisées pour lancer des applications graphiques.
all-rules-matched = Toutes les règles suivantes sont satisfaites :
default-profile = Par défaut
move-up = Se déplacer vers le haut
edit-rule = Modifier la règle
rename-profile-from = Renommer le profil <b>{ $old_name }</b> en :
activation-auto-switching-disabled = Le changement de profil automatique est actuellement désactivé
activation-settings-status =
    Les conditions d’activation sélectionnées { $matched ->
        [true] sont actuellement <b>réunies</b>
       *[false] ne sont actuellement <b>pas réunies</b>
    }
profile-rule-process-name = Nom de processus :
profile-rule-gamemode-tab = Le mode Jeu est actif
amd-oc = Overclocking AMD
amd-oc-updating-done = Configuration mise à jour, veuillez redémarrer pour appliquer les modifications.
amd-oc-updating-configuration = Mise à jour de la configuration (cela peut prendre un certain temps)
amd-oc-detected-system-config =
    Configuration système détectée : <b>{ $config ->
        [unsupported] Non prise en charge
       *[other] { $config }
    }</b>
amd-oc-status =
    L’overclocking AMD est actuellement : <b>{ $status ->
        [true] Activé
        [false] Désactivé
       *[other] Inconnu
    }</b>
amd-oc-description =
    { $config ->
        [rpm-ostree] Cette option activera ou désactivera la prise en charge de l’overdrive AMD en définissant des flags de boot à l’aide de <b>rpm-ostree</b>.
        [unsupported]
            Le système actuel n’est pas reconnu comme pris en charge pour la configuration automatique de l’overdrive.
            Vous pouvez essayer d’activer l’overclocking depuis LACT, mais une régénération manuelle de l’initramfs est susceptible d’être nécessaire pour que l’activation soit effective.
            Si cela ne fonctionne pas, vous pouvez essayer de définir le paramètre de boot <b>amdgpu.ppfeaturemask=0xffffffff</b> dans votre bootloader.
       *[other] Cette option activera ou désactivera la prise en charge de l’overdrive AMD en créant un fichier à l’emplacement <b>{ $path }</b>, puis en mettant à jour l’initramfs.
    }

    Consultez <a href="https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)">le wiki</a> pour plus d’informations.
generate-debug-snapshot = Générer un instantané de débogage
dump-vbios = Dumper le VBIOS
reset-all-config = Réinitialiser toute la configuration
stats-update-interval = Intervalle de mise à jour (ms)
reconnecting-to-daemon = Connexion au démon perdue, reconnexion...
daemon-connection-lost = Connexion perdue
plot-show-detailed-info = Afficher les détails
show-process-monitor = Afficher le moniteur de processus
temperature-sensor = Capteur de température
spindown-delay = Délai avant ralentissement (ms)
spindown-delay-tooltip = Durée pendant laquelle le GPU doit maintenir une valeur de température faible avant de ralentir le ventilateur
speed-change-threshold = Seuil de changement de vitesse (°C)
automatic-mode-threshold = Seuil du mode automatique (°C)
automatic-mode-threshold-tooltip =
    Passez le contrôle des ventilateurs en mode automatique lorsque la température passe en dessous de cette valeur.

    De nombreux GPU NVIDIA prennent en charge l'arrêt du ventilateur uniquement lorsque le contrôle des ventilateurs est en mode automatique, tandis que les courbes personnalisées sont limitées aux valeurs entre 30 et 100 %.

    Cette option permet de contourner cette limitation en utilisant uniquement la courbe personnalisée au-delà d'une température donnée, et en activant le mode automatique intégré à la carte dès que la température repasse en dessous de cette valeur, permettant d'atteindre un vitesse de 0 tr/min.
vram-usage = Utilisation de la VRAM :
performance-level = Niveau de performance
historical-data-title = Données historiques
graphs-per-row = Graphiques par ligne :
time-period-seconds = Période de temps (secondes) :
reset-all-graphs-tooltip = Réinitialiser tous les graphiques
add-graph = Ajouter un graphique
delete-graph = Supprimer le graphique
export-csv = Exporter en CSV
edit-graph-sensors = Modifier les capteurs du graphique
apply-button = Appliquer
revert-button = Rétablir
edit-graphs = Modifier
gibibyte = Gio
crash-page-title = L’application a planté
exit = Quitter
