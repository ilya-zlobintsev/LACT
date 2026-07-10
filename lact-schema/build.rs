use vergen_gitcl::{Emitter, Gitcl};

fn main() {
    Emitter::default()
        .add_instructions(&Gitcl::builder().sha(true).build())
        .unwrap()
        .emit()
        .unwrap()
}
