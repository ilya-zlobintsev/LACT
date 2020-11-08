use daemon::daemon_connection::DaemonConnection;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "lower")]
enum Opt {
    ///Gets realtime GPU information
    Stats {
        gpu_id: u32,
    },
    Gpus,
    Info {
        gpu_id: u32,
    },
    StartFanControl {
        gpu_id: u32,
    },
    StopFanControl {
        gpu_id: u32,
    },
    GetFanControl {
        gpu_id: u32,
    },
    Stop,
}

fn main() {
    env_logger::init();
    
    let opt = Opt::from_args();

    let d = DaemonConnection::new().unwrap();
    log::trace!("connection established");

    match opt {
        Opt::Gpus => {
            let gpus = d.get_gpus();
            println!("{:?}", gpus);
        },
        Opt::Stats { gpu_id } => {
            let gpu_stats = d.get_gpu_stats(gpu_id).unwrap();
            println!("VRAM: {}/{}", gpu_stats.mem_used, gpu_stats.mem_total);
            println!("{:?}", gpu_stats);
        },
        Opt::Info { gpu_id } => {
            let gpu_info = d.get_gpu_info(gpu_id).unwrap();
            println!("GPU Vendor: {}", gpu_info.gpu_vendor);
            println!("GPU Model: {}", gpu_info.card_model);
            println!("Driver in use: {}", gpu_info.driver);
            print!("VBIOS Version: {}", gpu_info.vbios_version);
        },
        Opt::StartFanControl { gpu_id } => {
            println!("{:?}", d.start_fan_control(gpu_id));
        },
        Opt::StopFanControl { gpu_id } => {
            println!("{:?}", d.stop_fan_control(gpu_id));
        },
        Opt::GetFanControl { gpu_id } => {
            println!("{:?}", d.get_fan_control(gpu_id));
        },
        Opt::Stop => d.shutdown(),
    }
}
