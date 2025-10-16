use std::{env, path::PathBuf};

use bindgen::EnumVariation::Rust;
use cmake::Config;

fn main() {
    println!("cargo:rerun-if-changed=soxr");
    println!("cargo:rerun-if-changed=include/wrapper.h");

    let dynamic = if env::var("CARGO_FEATURE_DYNAMIC").is_ok() {
        println!("cargo:rustc-link-lib=dylib=soxr");
        true
    } else {
        println!("cargo:rustc-link-lib=static=soxr");
        false
    };

    let libs = Config::new("soxr")
        .define("BUILD_SHARED_LIBS", if dynamic { "ON" } else { "OFF" })
        .define("BUILD_TESTS", "OFF")
        .define("WITH_DEV_TRACE", "OFF")
        .define("WITH_OPENMP", "OFF")
        .define("WITH_PFFFT", "ON")
        .define("WITH_LSR_BINDINGS", "OFF")
        .build();

    println!("cargo:rustc-link-search=native={}", libs.join("lib").display());

    let include_path = libs.join("include");
    println!("cargo:include={}", include_path.display());

    let builder = bindgen::builder()
        .header("include/wrapper.h")
        .clang_arg(format!("-I{}", include_path.display()))
        .default_enum_style(Rust {
            non_exhaustive: false,
        })
        .generate_comments(false)
        .layout_tests(false)
        .merge_extern_blocks(true);

    builder.generate().unwrap().write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("soxr.rs")).unwrap();
}
