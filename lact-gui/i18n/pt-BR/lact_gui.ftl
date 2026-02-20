oc-page = OC
thermals-page = Térmicos
software-page = Software
hardware-info = Informação de hardware
info-page = Informações
lact-gui = LACT GUI
lact-daemon = Daemon do LACT
kernel-version = Versão do kernel
system-section = Sistema
instance = Instância
device-name = Nome do dispositivo
version = Versão
extensions = Extensões
features = Recursos
platform-name = Nome da plataforma
api-version = Versão da API
driver-name = Nome do driver
driver-version = Versão do driver
compute-units = Unidades de computação
cl-c-version = Versão de OpenCL C
workgroup-size = Tamanho do workgroup
global-memory = Memória global
local-memory = Memória local
cache-info = Informações de cache
device-not-found = Dispositivo { $kind } não encontrado
show-button = Mostrar
reset-config-description = Tem certeza de que deseja redefinir todas as configurações da GPU?
acoustic-target = Alvo acústico (RPM)
gpu-pstate-clock-offset = Offset de clock do P-State { $pstate } da GPU (MHz)
cache-data = Dados
cache-instruction = Dados
cache-cpu = CPU
monitoring-section = Monitoramento
temperatures = Temperaturas
fan-speed = Velocidade da ventoinha
throttling = Aceleração
curve-page = Curva
target-temp = Temperatura alvo (ºC)
acoustic-limit = Limite acústico (RPM)
zero-rpm-stop-temp = Temperatura de parada de RPM zero (°C)
zero-rpm = RPM zero
static-speed = Velocidade estática (%)
enable-amd-oc-description = Isso habilitará o recurso de overdrive do driver amdgpu criando um arquivo em <b>{ $path }</b> e atualizando o initramfs. Tem certeza de que deseja fazer isso?
disable-amd-oc = Desabilitar overclocking de AMD
enable-amd-oc = Habilitar overclocking de AMD
power-cap = Limite de uso de energia
watt = W
ghz = GHz
mhz = MHz
stats-section = Estatísticas
gpu-clock = Clock de núcleo da GPU
gpu-clock-target = Clock do núcleo da GPU (alvo)
gpu-voltage = Voltagem da GPU
gpu-temp = Temperatura
vram-clock = Clock de VRAM
no-throttling = Não
missing-stat = N/D
performance-level-low = Clocks mais baixos
performance-level-auto-description = Ajusta automaticamente os clocks da GPU e da VRAM. (Padrão)
performance-level-low-description = Usa sempre as velocidades de clock mais baixas para GPU e VRAM.
performance-level-manual-description = Controle de desempenho manual.
power-profile-mode = Modo de perfil de energia:
manual-level-needed = O nível de desempenho deve ser definido como "manual" para usar estados e modos de energia
overclock-section = Velocidade de clock e voltagem
nvidia-oc-info = Informações de overclocking de Nvidia
oc-warning = Aviso: alterar esses valores pode causar instabilidade no sistema e danificar seu hardware!
show-all-pstates = Mostrar todos os P-States
enable-gpu-locked-clocks = Habilitar clocks travados de GPU
no-clocks-data = Nenhum dado de clock disponível
gpu-clock-offset = Offset do clock da GPU (MHz)
reset-oc-tooltip = Aviso: isso redefine todas as definições de clock para os valores padrão!
max-gpu-clock = Clock máximo da GPU (MHz)
max-vram-clock = Clock máximo da VRAM (MHz)
min-gpu-clock = Clock mínimo da GPU (MHz)
min-vram-clock = Clock mínimo da VRAM (MHz)
min-gpu-voltage = Voltagem mínima da GPU (mV)
gpu-voltage-offset = Offset de voltagem da GPU (mV)
nvidia-cache-desc = { $size } L{ $level }
min-fan-speed = Velocidade mínima da ventoinha (%)
gpu-usage = Uso da GPU
amd-cache-desc =
    { $size } L{ $level } { $types } cache { $shared ->
        [1] local para cada CU
       *[other] compartilhado entre { $shared } CUs
    }
fan-control-section = Controle de ventoinha
oc-missing-fan-control-warning = Aviso: o suporte a overclock está desabilitado, a funcionalidade de controle de ventoinha não está disponível.
pmfw-reset-warning = Aviso: isso redefine as configurações de firmware da ventoinha!
reset-config = Redefinir configuração
amd-oc-disabled =
    O suporte para overclocking da AMD não está habilitado!
    Você ainda pode alterar as configurações básicas, mas o controle mais avançado de clocks e voltagem não estará disponível.
performance-level-high-description = Usa sempre as velocidades de clock mais altas para GPU e VRAM.
disable-amd-oc-description = Isso desabilitará o suporte para overclocking de AMD (overdrive) na próxima reinicialização.
power-usage = Uso de energia
performance-level-high = Clocks mais altos
gpu-clock-avg = Clock de núcleo da GPU (médio)
max-gpu-voltage = Voltagem máxima da GPU (mV)
nvidia-oc-description =
    A funcionalidade de overclocking na Nvidia inclui a configuração de offsets para clocks de GPU/VRAM e a limitação da faixa potencial de clocks usando o recurso "clocks travados".

    Em muitas placas, o offset de clock da VRAM afetará o clock da memória real apenas pela metade do valor do offset.
    Por exemplo, um offset de VRAM de +1000 MHz pode aumentar a velocidade medida da VRAM em apenas 500 MHz.
    Isso é normal e é como a Nvidia lida com taxas de dados de GDDR. Ajuste seu overclock adequadamente.

    O controle direto de tensão não é suportado, pois não existe no driver Linux da Nvidia.

    É possível obter uma pseudo-subtensão combinando a opção de clocks travados com um offset positivo de clock.
    Isso forçará a GPU a operar em uma tensão limitada pelos clocks travados, enquanto atinge uma velocidade de clock maior devido ao offset.
    Isso pode causar instabilidade no sistema se for muito alto.
enable-vram-locked-clocks = Habilitar clocks travados de VRAM
pstate-list-description = <b>Os valores a seguir são offsets de clock para cada P-State, do maior para o menor.</b>
auto-page = Automático
static-page = Estático
reset-button = Redefinir
unknown-throttling = Desconhecido
performance-level-auto = Automático
performance-level-manual = Manual
mebibyte = MiB
gpu-pstate-clock = Clock do P-State { $pstate } da GPU (MHz)
mem-pstate-clock = Clock do P-State { $pstate } da VRAM (MHz)
gpu-pstate-clock-voltage = Voltagem do P-State { $pstate } da GPU (mV)
mem-pstate-clock-voltage = Voltagem do P-State { $pstate } da VRAM (mV)
pstates = Estados de energia
gpu-pstates = Estados de energia da GPU
vram-pstates = Estados de energia da VRAM
enable-pstate-config = Habilitar configuração do estado de energia
show-historical-charts = Mostrar gráficos de histórico
settings-profile = Perfil das configurações
auto-switch-profiles = Trocar automaticamente
add-profile = Adiciona novo perfil
import-profile = Importa perfil de arquivo
vram-pstate-clock-offset = Offset de clock do P-State { $pstate } da VRAM (MHz)
pstates-manual-needed = O nível de desempenho deve ser definido como 'manual' para alternar os estados de energia
profile-hook-deactivated = Desativado:
create-profile = Criar perfil
rename-profile = Renomear perfil
profile-activation-desc = Ativar o perfil "{ $name }" quando:
all-rules-matched = Todas as seguintes regras forem atendidas:
edit-rule = Editar regra
profile-activation = Ativação
cancel = Cancelar
default-profile = Padrão
profile-copy-from = Copiar configurações de:
name = Nome
create = Criar
save = Salvar
delete-profile = Excluir perfil
edit-rules = Editar regras
remove-rule = Remover regra
profile-rules = Regras do perfil
move-up = Mover para cima
move-down = Mover para baixo
profile-hooks = Ganchos
any-rules-matched = Qualquer uma das seguintes regras forem atendidas:
profile-hook-command = Executar um comando quando o perfil "{ $cmd }" for:
rename-profile-from = Renomear o perfil <b>{ $old_name }</b> para:
export-to-file = Exportar para arquivo
activation-settings-status =
    As configurações de ativação selecionadas atualmente <b>{ $matched ->
        [true] foram atendidas
       *[false] não foram atendidas
    }</b>
activation-auto-switching-disabled = A troca automática de perfil está atualmente desabilitada
profile-hook-activated = Ativado:
profile-hook-note = Observação: esses comandos são executados como root pelo daemon do LACT e não têm acesso ao ambiente de trabalho. Portanto, não podem ser usados diretamente para iniciar aplicativos gráficos.
profile-rule-process-tab = Um processo está em execução
profile-rule-gamemode-tab = Gamemode está ativo
profile-rule-process-name = Nome do processo:
profile-rule-args-contain = Os argumentos contêm:
profile-rule-specific-process = Com um processo específico:
amd-oc = Overclocking da AMD
amd-oc-updating-configuration = Atualizando a configuração (isso pode levar um tempo)
amd-oc-updating-done = Configuração atualizada. Por favor, reinicie para aplicar alterações.
amd-oc-detected-system-config =
    Detectada configuração do sistema: <b>{ $config ->
        [unsupported] sem suporte
       *[other] { $config }
    }</b>
amd-oc-status =
    Overclocking da AMD está atualmente: <b>{ $status ->
        [true] habilitado
        [false] desabilitado
       *[other] desconhecido
    }</b>
amd-oc-description =
    { $config ->
        [rpm-ostree] Esta opção vai alternar o suporte a overdrive da AMD definindo sinalizadores de inicialização por meio de <b>rpm-ostree</b>.
        [unsupported]
            O sistema atual não é reconhecido como suportado por configuração automática do overdrive.
            Você pode tentar habilitar overclocking do LACT, mas uma regeneração manual do initramfs pode exigida para que tenha efeito.
            Se isso falhar, uma opção alternativa é adicionar <b>amdgpu.ppfeaturemask=0xffffffff</b> como um parâmetro de inicialização no seu bootloader.
       *[other] Esta opção alternará o suporte ao overdrive da AMD criando um arquivo em <b>{ $path }</b> e atualizando o initramfs.
    }

    Veja <a href="https://github.com/ilya-zlobintsev/LACT/wiki/Overclocking-(AMD)">a wiki</a> para mais informações.
reconnecting-to-daemon = Conexão do daemon perdida, reconectando...
daemon-connection-lost = Conexão perdida
plot-show-detailed-info = Mostrar informações detalhadas
generate-debug-snapshot = Gerar snapshot de depuração
dump-vbios = Exportar VBIOS
reset-all-config = Redefinir todas configurações
stats-update-interval = Intervalo de atualização (ms)
show-process-monitor = Mostrar monitor de processos
temperature-sensor = Sensor de temperatura
spindown-delay = Atraso da desaceleração (ms)
spindown-delay-tooltip = Por quanto tempo a GPU precisa permanecer em uma temperatura mais baixa antes de desacelerar a ventoinha
speed-change-threshold = Limiar de mudança de velocidade (ºC)
automatic-mode-threshold = Limiar do Modo Automático
automatic-mode-threshold-tooltip =
    Altera o controle de ventoinha para o modo automático quando a temperatura estiver abaixo deste ponto.

    Várias GPUs Nvidia suportam parar as ventoinhas apenas no modo de controle automático das ventoinhas, enquanto curvas personalizadas tem uma faixa de velocidade limitada entre 30-100%.

    Esta opção permite lidar com esta limitação utlizando a curva personalizada apenas acima de uma temperatura específica, e abaixo desta, utilizando o modo automático que suporta parar as ventoinhas (zero RPM).
apply-button = Aplicar
revert-button = Reverter
vram-usage = Uso de VRAM:
performance-level = Nível de Desempenho
historical-data-title = Dados Históricos
graphs-per-row = Gráficos Por Linha:
time-period-seconds = Período de Tempo (Segundos):
reset-all-graphs-tooltip = Resetar Todos os Gráficos para o Padrão
add-graph = Adicionar Gráfico
delete-graph = Deletar Gráfico
edit-graphs = Editar
export-csv = Exportar como CSV
edit-graph-sensors = Editar Gráfico de Sensores
gibibyte = GiB
crash-page-title = O aplicativo travou
exit = Sair
bytes = bytes
kibibyte = KiB
