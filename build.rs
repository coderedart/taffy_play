fn main() {
    let gitcl = vergen_gitcl::GitclBuilder::all_git().expect("failed to buidl gitcl indstructions");
    vergen_gitcl::Emitter::default()
        .add_instructions(&gitcl)
        .expect("failed to add gitcl instructions")
        .emit()
        .expect("failed to emit instructions");
}
