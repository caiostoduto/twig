use vergen_gitcl::{Emitter, GitclBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate git commit info
    let gitcl = GitclBuilder::default().sha(true).branch(true).build()?;
    Emitter::default().add_instructions(&gitcl)?.emit()?;

    // Compile the protobuf files
    tonic_prost_build::configure()
        .compile_protos(&["./proto/minecraft_bridge.proto"], &["./proto"])
        .unwrap();

    Ok(())
}
