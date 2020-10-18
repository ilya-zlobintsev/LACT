use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "lower")]
enum Opt {
    ///Gets realtime GPU information
    Stats,
    Info,
}

fn main() {
    let opt = Opt::from_args();
    match opt {
        Opt::Stats => {
            let gpu_stats = daemon::get_gpu_stats();
            println!("VRAM: {}/{}", gpu_stats.mem_used, gpu_stats.mem_total);
        }
        Opt::Info => {
            let gpu_info = daemon::get_gpu_info();
            println!("GPU Vendor: {}", gpu_info.gpu_vendor);
            println!("GPU Model: {}", gpu_info.card_model);
            println!("Driver in use: {}", gpu_info.driver);
            print!("VBIOS Version: {}", gpu_info.vbios_version);
        }
    }
}
