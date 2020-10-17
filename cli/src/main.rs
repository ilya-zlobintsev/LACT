use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "lower")]
enum Opt {
    ///Get information about the GPU.
    Info,
}

fn main() {
    let opt = Opt::from_args();
    match opt {
        Opt::Info => daemon::get_info(),
    }
}
