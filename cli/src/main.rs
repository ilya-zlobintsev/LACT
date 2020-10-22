use daemon::daemon_connection::DaemonConnection;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "lower")]
enum Opt {
    ///Gets realtime GPU information
    Stats,
    Info,
    Start,
    Stop,
}

fn main() {
    let opt = Opt::from_args();

    let d = DaemonConnection::new().unwrap();

    match opt {
        Opt::Stats => {
            let gpu_stats = d.get_gpu_stats();
            println!("VRAM: {}/{}", gpu_stats.mem_used, gpu_stats.mem_total);
            println!("{:?}", gpu_stats);
        },
        Opt::Info => {
            let gpu_info = d.get_gpu_info();
            println!("GPU Vendor: {}", gpu_info.gpu_vendor);
            println!("GPU Model: {}", gpu_info.card_model);
            println!("Driver in use: {}", gpu_info.driver);
            print!("VBIOS Version: {}", gpu_info.vbios_version);
        },
        Opt::Start => {
            println!("{:?}", d.start_fan_control());
        },
        Opt::Stop => {
            println!("{:?}", d.stop_fan_control());
        }
    }
}
