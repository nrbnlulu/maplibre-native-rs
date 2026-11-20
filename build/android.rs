//! Reuse the matching core's platform implementations with narrowly scoped fixes.
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn platform_sources(native: &Path, output: &Path) -> [PathBuf; 3] {
    fs::create_dir_all(output).expect("Create Android platform output directory");
    let sources = [
        ("platform/android/src/run_loop.cpp", "run_loop.cpp"),
        ("platform/android/src/thread.cpp", "thread.cpp"),
        ("platform/default/src/mln/storage/http_file_source.cpp", "http_file_source.cpp"),
    ];
    // Always start from upstream so a build-script rerun never reapplies the patch.
    for (source, destination) in sources {
        let source = native.join(source);
        println!("cargo:rerun-if-changed={}", source.display());
        fs::copy(source, output.join(destination)).expect("Copy upstream Android platform source");
    }
    println!(
        "cargo:rerun-if-changed={}",
        native.join("platform/android/src/run_loop_impl.hpp").display()
    );
    println!("cargo:rerun-if-changed=build/android.rs");
    println!("cargo:rerun-if-changed=src/android");

    // Git is already required to prepare native sources. Apply the compatibility
    // patch only to private build outputs, leaving the upstream checkout intact.
    let patch = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/android/platform-compat.patch");
    let result = Command::new("git")
        .args(["apply", "--unsafe-paths", "--directory"])
        .arg(fs::canonicalize(output).expect("Resolve Android platform output directory"))
        .arg(&patch)
        .output()
        .expect("Run git apply for Android platform compatibility");
    assert!(
        result.status.success(),
        "Cannot apply Android platform compatibility patch; review the core revision:\n{}",
        String::from_utf8_lossy(&result.stderr)
    );
    [output.join("run_loop.cpp"), output.join("http_file_source.cpp"), output.join("thread.cpp")]
}
