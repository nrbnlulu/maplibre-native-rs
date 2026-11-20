use downloader::{Download, Downloader};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{env, fs};

const RELEASE: &str =
    "https://github.com/nrbnlulu/maplibre-flutter/releases/download/mln-core-a33d3f00-r1";
const REQUIRED: &[&str] = &[
    "libmbgl-core.a",
    "libmbgl-freetype.a",
    "libmbgl-harfbuzz.a",
    "libmbgl-vendor-csscolorparser.a",
    "libmbgl-vendor-parsedate.a",
    "libmbgl-vendor-sqlite.a",
    "vendor/maplibre-tile-spec/cpp/libmlt-cpp.a",
    "vendor/maplibre-tile-spec/cpp/libfastpfor-lib.a",
    "maplibre-native-headers.tar.gz",
    "webgpu-cpp/webgpu.hpp",
    "src/mln/gfx/offscreen_texture.hpp",
];

pub(super) struct Bundle {
    asset: &'static str,
    sha256: &'static str,
    unicode: &'static str,
}

impl Bundle {
    pub(super) fn for_target(target: &str) -> Option<Self> {
        match target {
            "x86_64-unknown-linux-gnu" => Some(Self {
                asset: "maplibre-native-linux-x86_64-ffi-a33d3f00.tar.gz",
                sha256: "03905d6b93ba7234fc7f1f3ca3b07e19527809aa8c85cc4ea186e121f5dfc7ce",
                unicode: "libmbgl-vendor-nunicode.a",
            }),
            "aarch64-linux-android" => Some(Self {
                asset: "maplibre-native-android-arm64-v8a-api24-ffi-a33d3f00-relwithdebinfo.tar.gz",
                sha256: "762785758ab044c14d52ac327caaefb941ea4fc8e9509d808aff0517ae7f70af",
                unicode: "libmbgl-vendor-icu.a",
            }),
            _ => None,
        }
    }

    fn files(&self) -> impl Iterator<Item = &str> {
        REQUIRED.iter().copied().chain([self.unicode])
    }

    fn is_complete(&self, path: &Path) -> bool {
        self.files().all(|file| {
            path.join(file)
                .metadata()
                .is_ok_and(|metadata| metadata.is_file() && metadata.len() != 0)
        })
    }

    pub(super) fn install(&self) -> Result<PathBuf, Box<dyn Error>> {
        println!("cargo:rerun-if-env-changed=MLN_CORE_CACHE_DIR");
        let root = env::var_os("MLN_CORE_CACHE_DIR").map_or_else(
            || {
                PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo"))
                    .join("released-cores")
            },
            PathBuf::from,
        );
        fs::create_dir_all(&root)?;
        let root = root.canonicalize()?;
        let ready = root.join(self.sha256);

        // An optional shared cache can be used by several independent Cargo builds.
        let lock = fs::File::options()
            .write(true)
            .create(true)
            .truncate(false)
            .open(root.join(format!("{}.lock", self.sha256)))?;
        lock.lock()?;
        if !self.is_complete(&ready) {
            let stage = tempfile::tempdir_in(&root)?;
            let url = format!("{RELEASE}/{}", self.asset);
            println!("cargo:warning=Downloading precompiled MapLibre Native from {url}");
            let mut downloader = Downloader::builder()
                .download_folder(stage.path())
                .timeout(Duration::from_secs(1800))
                .build()?;
            for result in downloader.download(&[Download::new(&url)])? {
                result?;
            }
            let archive = stage.path().join(self.asset);
            let actual = sha256::try_digest(archive.as_path())?;
            if actual != self.sha256 {
                return Err(format!(
                    "SHA-256 mismatch for {}: expected {}, got {actual}",
                    self.asset, self.sha256
                )
                .into());
            }
            let tar = flate2::read::GzDecoder::new(fs::File::open(archive)?);
            tar::Archive::new(tar).unpack(stage.path())?;
            let extracted = stage.path().join("mln-core");
            if !self.is_complete(&extracted) {
                return Err(format!("Incomplete MapLibre Native bundle: {}", self.asset).into());
            }
            if ready.exists() {
                fs::remove_dir_all(&ready)?;
            }
            fs::rename(extracted, &ready)?;
        }
        for file in self.files() {
            println!("cargo:rerun-if-changed={}", ready.join(file).display());
        }
        Ok(ready)
    }
}
