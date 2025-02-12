use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() {
    if env::var("DOCS_RS").is_ok() {
        println!("cargo:rustc-cfg=docs_rs");
        return;
    }

    let out = cmake::Config::new("CEO").build();

    println!(
        "cargo:rustc-link-search=native={}",
        out.join("lib").display()
    );
    println!("cargo:rustc-link-lib=static=ceo");
    println!("cargo:rustc-link-search=/usr/local/cuda/lib64");
    println!("cargo:rustc-link-lib=stdc++");
    println!("cargo:rustc-link-lib=cudart");
    println!("cargo:rustc-link-lib=cudadevrt");
    println!("cargo:rustc-link-lib=cublas");
    println!("cargo:rustc-link-lib=cufft");
    println!("cargo:rustc-link-lib=cusparse");
    println!("cargo:rustc-link-lib=curand");
    println!("cargo:rustc-link-lib=cusolver");
    println!("cargo:rerun-if-changed=wrapper.hpp");
    println!("cargo:rustc-cfg=bindings");

    let bindings = bindgen::Builder::default()
        .header("wrapper.hpp")
        .clang_arg(&format!("-I{}", out.join("include").display()))
        .clang_arg("-I/usr/local/cuda/include")
        .clang_arg("-v")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_type("gpu_float")
        .allowlist_type("gpu_int")
        .allowlist_type("gpu_double")
        .allowlist_type("mask")
        .allowlist_function("set_device")
        .allowlist_function("get_device_count")
        .allowlist_function("host2dev_char")
        .allowlist_function("host2dev")
        .allowlist_function("dev2host")
        .allowlist_function("dev2host_int")
        .allowlist_type("source")
        .allowlist_type("pssn")
        .allowlist_type("centroiding")
        .allowlist_type("imaging")
        .allowlist_type("shackHartmann")
        .allowlist_type("geometricShackHartmann")
        .allowlist_type("coordinate_system")
        .allowlist_type("gmt_m1")
        .allowlist_type("gmt_m2")
        .allowlist_type("atmosphere")
        .allowlist_type("LMMSE")
        .allowlist_type("pyramid")
        .allowlist_type("conic")
        .allowlist_type("segmentPistonSensor")
        .allowlist_function("transform_to_S")
        .allowlist_function("transform_to_R")
        .allowlist_function("intersect")
        .allowlist_function("reflect")
        .allowlist_function("refract")
        .allowlist_function("geqrf")
        .allowlist_function("ormqr")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
    fs::copy(
        out_path.join("bindings.rs"),
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("bindings.rs"),
    )
    .expect("failed to copy bindings to src");
}
