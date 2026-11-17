# Standalone Android platform (experimental)

The WebGPU backend can run on Android without MapLibre's Java SDK or a Java VM
registered with MapLibre. The embedding application supplies the wgpu device,
queue operations and display texture as on desktop. Android ARM64 was exercised
on a Samsung SM-E225F running Android 13. Other ABIs are not device-qualified.

The standalone platform uses ALooper for work/timers and curl socket watches,
Rust image decoding, native Unicode case conversion/collation, and upstream
fallback number formatting/local glyph handling. Styles must provide remote
glyphs; platform-local font rasterization is not implemented by this adapter.
It does not initialize or use MapLibre's Java classes.

## Cross compilation

Install the Rust Android target, Android NDK, Ninja and target-built libcurl with
its TLS/compression dependencies. Native TLS must support `CURLOPT_CAINFO_BLOB`
(libcurl 7.77 or newer). The device run used NDK 29.0.14206865, API level 24 and
a static curl/OpenSSL distribution. GStreamer is not a dependency of this crate.

Configure the target tools and pkg-config separately from host dependencies:

```sh
rustup target add aarch64-linux-android
export ANDROID_NDK_HOME=/path/to/android-ndk
export ANDROID_API_LEVEL=24
ndk_llvm_dir="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64"
export CC_aarch64_linux_android="$ndk_llvm_dir/bin/aarch64-linux-android24-clang"
export CXX_aarch64_linux_android="$ndk_llvm_dir/bin/aarch64-linux-android24-clang++"
export AR_aarch64_linux_android="$ndk_llvm_dir/bin/llvm-ar"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$CC_aarch64_linux_android"
export PKG_CONFIG_ALLOW_CROSS=1
export PKG_CONFIG_LIBDIR_aarch64_linux_android=/path/to/android-curl/lib/pkgconfig
export PKG_CONFIG_SYSROOT_DIR_aarch64_linux_android=/
export BINDGEN_EXTRA_CLANG_ARGS_aarch64_linux_android="--sysroot=$ndk_llvm_dir/sysroot --target=aarch64-linux-android24"
MLN_PRECOMPILE=0 cargo build --target aarch64-linux-android --features wgpu
```

Use the NDK/tool paths for your host; the example above is for Linux x86_64.
CMake selects the Android ABI from Cargo's target and uses `ANDROID_API_LEVEL`
(default 24) for the core and the small nunicode support library. Do not pass a
Linux core archive to an Android build. The embedding APK must package a matching
`libc++_shared.so`.

For a precompiled core, set `MLN_PRECOMPILE=1`,
`MLN_CORE_LIBRARY_USE_AMALGAM=0`, `MLN_CORE_LIBRARY_PATH` and
`MLN_CORE_LIBRARY_HEADERS_PATH`, plus the usual WebGPU/internal header paths.
The Android support layer also needs the matching full core sources. Set
`MLN_LOCAL_REPOSITORY` to that checkout (with initialized submodules), or the
build fetches the pinned core revision automatically. The build compiles
nunicode itself; it needs no separately prepared support archive or private
machine-specific paths. The core and support sources must use the same revision.

## Networking and certificates

The curl multi loader uses readable and writable ALooper socket events.
Nonblocking renderer ticks also poll ALooper so completion notifications advance
while the renderer is driven by an external frame loop.

HTTPS verification stays enabled. The adapter builds its trust bundle from the
system certificate directories in the Conscrypt APEX and/or
`/system/etc/security/cacerts`; it fails if no system certificates can be loaded.
It does not add user-installed CAs or implement application-specific Android
network-security-config overrides.

## Validation and remaining work

The consuming Flutter Rust-shell application loaded street vector tiles and
hybrid satellite imagery, displayed site overlays/live simulated radar alerts,
and exercised dragging and cluster zoom. Its Android shell needed GPU device
and guarded queue accessors. Debug wgpu lock backtrace capture also needed to be
disabled before native initialization; profiling identified it as a major source
of CPU overhead. These are embedding/runtime changes, not guarantees made by
this crate.

The Android integration remains experimental. Text/icon corruption was observed
in the embedding UI during hybrid-map use. Shared Vulkan texture/resource
synchronization still needs validation on device, as do background/resume,
repeated map replacement, other GPUs/ABIs and release builds. Do not describe the
integration as production-qualified based on the successful map load.

The decoder tests cover RGBA/alpha preservation and rejection of malformed,
empty and null input. Build them with the same cross-compilation settings:

```sh
cargo test --target aarch64-linux-android --features wgpu --lib --no-run
```

Run the resulting test executable on an authorized Android device with the
matching `libc++_shared.so` available through `LD_LIBRARY_PATH`, filtering to
`android::tests` when only the decoder checks are desired. A successful cross
build does not establish that the tests executed on a phone.

The adapted C++ platform files retain MapLibre Native's BSD-2-Clause terms in
`src/android/LICENSE-MAPLIBRE`.

## Upstream platform sources

The build uses the matching MapLibre Native Android `RunLoop` and default
`HTTPFileSource` sources rather than maintaining copies in this crate. Android's
NDK `ALooper` supplies the event loop; no Java `Looper` or Java VM is needed.
The HTTP helpers under `src/mln/util` are already part of the native core; the
platform `HTTPFileSource` is what connects those helpers to curl.

Two socket-watch edits enable writable events needed by curl. The HTTP source
receives only a system-certificate hook. These changes are applied to generated
sources under Cargo's `OUT_DIR`; the upstream checkout is never modified. The standard
`src/android/platform-compat.patch` is applied with `git apply`; a context mismatch
fails the build so compatibility changes are reviewed when upgrading the core.
The original upstream run-loop header is included directly.
