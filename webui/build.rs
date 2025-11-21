use vergen_git2::{CargoBuilder,  Emitter, Git2Builder};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
// Configure Git2Builder for key Git constants (customize as needed)
    let git = Git2Builder::default()
        .commit_timestamp(true)  // Emits VERGEN_GIT_COMMIT_TIMESTAMP
        .dirty(true)             // Emits VERGEN_GIT_DIRTY
        .sha(true)               // Emits VERGEN_GIT_SHA (short/long commit hash)
        .describe(true, true, None)  // Emits VERGEN_GIT_DESCRIBE (tag + commits since)
        .build()?;

    // Optional: Configure CargoBuilder for Cargo info
    let cargo = CargoBuilder::default()
        .opt_level(true)         // Emits VERGEN_CARGO_OPT_LEVEL
        .build()?;

    Emitter::default()
        .add_instructions(&git)?
        .add_instructions(&cargo)?
        .emit()?;

    // Rerun if Git changes (for incremental builds)
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs");
    Ok(())
}
