use vergen_git2::{CargoBuilder,  Emitter, Git2Builder};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    println!("Regular debug message - visible with cargo build -v");
    if let Ok(profile_file) = env::var("ENV_PROFILE") {
        println!("cargo:rerun-if-changed={}", profile_file);
        dotenvy::from_path(profile_file).ok();
    }
    let vars_to_export = [
        "API_FILE_LISTING_URL",
        "API_LOGIN_URL",
        "API_REFRESH_TOKEN_URL",
        "APP_ENV", // dev / staging / production
    ];

    for var in vars_to_export {
        if let Ok(value) = env::var(var) {
            println!("cargo:rustc-env={}={}", var, value);
        }
    }
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
    println!("cargo:rerun-if-env-changed=PROFILE");
    println!("cargo:rerun-if-changed=.env");
    println!("cargo:rerun-if-changed=.env.local");
    println!("cargo:rerun-if-changed=.env.development");
    println!("cargo:rerun-if-changed=.env.production");
    println!("cargo:rerun-if-changed=.env.staging");
    Ok(())
}
