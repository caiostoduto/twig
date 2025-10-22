use vergen_gitcl::{Emitter, GitclBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gitcl = GitclBuilder::default().sha(true).branch(true).build()?;

    Emitter::default().add_instructions(&gitcl)?.emit()?;

    Ok(())
}
