use cxx_build::bridge as cxxbridge;
use fs_extra::dir;
use std::{
    env,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

static SPIDERMONKEY_BUILD_DIR: &str = "spidermonkey-wasm-build";
const WASI_SDK_VERSION: &str = "12.0";

struct WasiSdk {
    cxx: PathBuf,
    sysroot: PathBuf,
    ar: PathBuf,
    search_paths: Vec<PathBuf>,
}

fn main() {
    let sdk = derive_wasi_sdk();

    let out_dir = env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .expect("could not find OUT_DIR");
    let source_dir = PathBuf::from(SPIDERMONKEY_BUILD_DIR);
    let source_include_dir = source_dir.join("include");
    let source_lib_dir = source_dir.join("lib");

    let out_include_dir = out_dir.join("include");
    let out_lib_dir = out_dir.join("lib");

    if !source_dir.exists() {
        panic!("SpiderMonkey build directory not found. Try updating git submodules via git submodule update --recursive --init");
    }

    if !source_include_dir.exists() || !source_lib_dir.exists() {
        panic!("SpiderMonkey build artifacts not found.");
    }

    if !out_include_dir.exists() {
        let copy_options = dir::CopyOptions::new();
        dir::copy(source_include_dir, &out_dir, &copy_options)
            .expect("Could not copy header files to OUT directory");
    }

    if !out_lib_dir.exists() {
        let copy_options = dir::CopyOptions::new();
        dir::copy(source_lib_dir, &out_dir, &copy_options)
            .expect("Could not copy lib directory to OUT directory");
    }

    println!("cargo:rustc-link-search={}", out_lib_dir.display());

    for path in &sdk.search_paths {
        println!("cargo:rustc-link-search=native={}", path.display());
    }

    println!("cargo:rustc-link-lib=static=jsrust");
    println!("cargo:rustc-link-lib=static=js_static");
    println!("cargo:rustc-link-lib=static=c++abi");
    println!("cargo:rustc-link-lib=static=clang_rt.builtins-wasm32");
    bridge(&sdk, &out_lib_dir, &out_include_dir);

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/api.h");
    println!("cargo:rerun-if-changed=src/api.cpp");
    println!("cargo:rerun-if-changed=src/lib.rs");
}

fn bridge(wasi_sdk: &WasiSdk, lib_dir: impl AsRef<Path>, include_dir: impl AsRef<Path>) {
    let mut builder = cxxbridge("src/lib.rs");
    builder
        .cpp(true)
        .cpp_link_stdlib("c++")
        .compiler(&wasi_sdk.cxx)
        .archiver(&wasi_sdk.ar)
        .file("src/api.cpp")
        .include(include_dir)
        .include("src")
        .target("wasm32-wasi")
        .flag_if_supported("-Wall")
        .flag_if_supported("-Werror")
        .flag_if_supported("-Qunused-arguments")
        .flag_if_supported("-fno-sized-deallocation")
        .flag_if_supported("-fno-exceptions")
        .flag_if_supported("-fno-aligned-new")
        .flag_if_supported("-mthread-model")
        .flag_if_supported("single")
        .flag_if_supported("-fPIC")
        .flag_if_supported("-fno-rtti")
        .flag_if_supported("-fno-math-errno")
        .flag_if_supported("-pipe")
        .flag_if_supported("-fno-omit-frame-pointer")
        .flag_if_supported("-funwind-tables")
        .flag_if_supported("-Wno-invalid-offsetof")
        .flag_if_supported("-std=gnu++17")
        .flag_if_supported(&format!("--sysroot={}", &wasi_sdk.sysroot.display()));

    for entry in WalkDir::new(lib_dir)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let entry_path = entry.path();
        if entry_path.is_file() && entry_path.extension().unwrap() == "o" {
            builder.object(entry_path);
        }
    }

    builder.opt_level(2).compile("spidermonkey-wasm");
}

fn derive_wasi_sdk() -> WasiSdk {
    let root = env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .expect("could not retrieve root dir");
    let host = match std::env::consts::OS {
        p @ "linux" => p,
        p @ "macos" => p,
        p => panic!("{} platform not supported", p),
    };

    let base_path = root
        .join("vendor")
        .join(host)
        .join(format!("wasi-sdk-{}", WASI_SDK_VERSION));

    WasiSdk {
        cxx: base_path.join("bin").join("clang"),
        sysroot: base_path.join("share").join("wasi-sysroot"),
        ar: base_path.join("bin").join("ar"),
        search_paths: vec![
            base_path
                .join("share")
                .join("wasi-sysroot")
                .join("lib")
                .join("wasm32-wasi"),
            base_path
                .join("lib")
                .join("clang")
                .join("11.0.0")
                .join("lib")
                .join("wasi"),
        ],
    }
}
