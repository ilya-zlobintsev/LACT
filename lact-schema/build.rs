use vergen::EmitBuilder;

fn main() {
    EmitBuilder::builder().git_sha(true).emit().unwrap()
}
