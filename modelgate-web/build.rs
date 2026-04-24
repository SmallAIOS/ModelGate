//! Fails the build with a helpful message when the frontend bundle is
//! missing. The bundle is produced by `npm run build` in
//! `ui/modelgate-web/`. The error here is louder and more actionable
//! than the `include_dir!` compile error would be on its own.

use std::path::Path;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let dist = Path::new(&manifest_dir)
        .join("..")
        .join("ui")
        .join("modelgate-web")
        .join("dist");

    // Re-run build.rs when anything in dist/ changes so rebuilds pick up
    // a fresh frontend without a manual `cargo clean`.
    println!("cargo:rerun-if-changed=../ui/modelgate-web/dist");

    if !dist.join("index.html").exists() {
        eprintln!(
            "\n\
             error: modelgate-web frontend bundle missing at {}.\n\
             \n\
             The modelgate-web crate embeds the SPA built from\n\
             ui/modelgate-web/ via include_dir!. That directory needs\n\
             to have been built at least once before the crate can\n\
             compile.\n\
             \n\
             Remediation:\n\
                 cd ui/modelgate-web && npm install && npm run build\n",
            dist.display()
        );
        std::process::exit(1);
    }
}
