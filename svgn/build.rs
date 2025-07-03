// this_file: svgn/build.rs
// Build script to set version from git tags

use std::process::Command;

fn main() {
    // Try to get version from git tag
    let git_version = Command::new("git")
        .args(["describe", "--tags", "--abbrev=0"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        });

    // If we have a git tag, use it (stripping 'v' prefix if present)
    if let Some(tag) = git_version {
        let version = tag.strip_prefix('v').unwrap_or(&tag);
        println!("cargo:rustc-env=CARGO_PKG_VERSION={}", version);
        println!("cargo:rustc-env=SVGN_VERSION={}", version);
    } else {
        // Fall back to Cargo.toml version
        println!("cargo:rustc-env=SVGN_VERSION={}", env!("CARGO_PKG_VERSION"));
    }
}
