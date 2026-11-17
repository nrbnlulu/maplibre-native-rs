//! Reuse the matching core's platform implementations with narrowly scoped fixes.
use std::fs;
use std::path::{Path, PathBuf};

fn replace_once(source: String, before: &str, after: &str, description: &str) -> String {
    assert_eq!(source.matches(before).count(), 1,
        "MapLibre Android compatibility fix no longer matches: {description}; review the core revision");
    source.replacen(before, after, 1)
}

pub fn platform_sources(native: &Path, output: &Path) -> [PathBuf; 2] {
    fs::create_dir_all(output).expect("Create Android platform output directory");
    let run_loop = native.join("platform/android/src/run_loop.cpp");
    let http = native.join("platform/default/src/mln/storage/http_file_source.cpp");
    for path in [&run_loop, &http, &native.join("platform/android/src/run_loop_impl.hpp")] {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    println!("cargo:rerun-if-changed=build/android.rs");
    println!("cargo:rerun-if-changed=src/android");

    // Upstream Android watches only readable sockets. The upstream curl multi
    // source also registers Write/ReadWrite events while establishing connections.
    let source = fs::read_to_string(run_loop).expect("Read upstream Android RunLoop");
    let source = replace_once(
        source,
        include_str!("../src/android/socket_callback.before"),
        include_str!("../src/android/socket_callback.after"),
        "socket readiness dispatch",
    );
    let source = replace_once(
        source,
        include_str!("../src/android/socket_watch.before"),
        include_str!("../src/android/socket_watch.after"),
        "socket watch registration",
    );
    let run_loop_output = output.join("run_loop.cpp");
    fs::write(&run_loop_output, source).expect("Write Android RunLoop compatibility source");

    // Preserve the upstream HTTP implementation, including cache/error semantics.
    // Only supply the Android system trust bundle; TLS verification stays enabled.
    let source = fs::read_to_string(http).expect("Read upstream HTTPFileSource");
    let source = replace_once(
        source,
        "    handleError(curl_easy_setopt(handle, CURLOPT_PRIVATE, this));",
        r#"    const auto& certificates = androidTrustBundle();
    if (certificates.empty()) throw std::runtime_error("Android system trust store is empty");
    curl_blob trust{const_cast<char*>(certificates.data()), certificates.size(), CURL_BLOB_COPY};
    handleError(curl_easy_setopt(handle, CURLOPT_CAINFO_BLOB, &trust));
    handleError(curl_easy_setopt(handle, CURLOPT_PRIVATE, this));"#,
        "Android TLS trust bundle",
    );
    let http_output = output.join("http_file_source.cpp");
    fs::write(&http_output, format!("#include \"system_certificates.hpp\"\n{source}"))
        .expect("Write Android HTTP compatibility source");
    [run_loop_output, http_output]
}
