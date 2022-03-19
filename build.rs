use bindgen;

fn main() {
    let bindings = bindgen::Builder::default()
        .header("wrapper.hpp")
        .clang_arg("-I/usr/local/cuda/include")
        .clang_arg("-v")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .allowlist_type("gpu_float")
        .allowlist_type("gpu_double")
        .allowlist_type("mask")
        .allowlist_function("set_device")
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
        .allowlist_function("transform_to_S")
        .allowlist_function("transform_to_R")
        .allowlist_function("intersect")
        .allowlist_function("intersect")
        .allowlist_function("reflect")
        .allowlist_function("refract")
        .allowlist_function("geqrf")
        .allowlist_function("ormqr")
        .generate()
        .expect("Unable to generate bindings");
    //    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file("src/bindings.rs")
        .expect("Couldn't write bindings!");
    println!("cargo:rustc-link-search=native=CEO/lib/");
    println!("cargo:rustc-link-search=native=CEO/jsmn");
    println!("cargo:rustc-link-lib=static=ceo");
    println!("cargo:rustc-link-lib=static=jsmn");
    println!("cargo:rustc-link-lib=curl");
    println!("cargo:rustc-link-search=native=/usr/local/cuda/lib64");
    println!("cargo:rustc-link-lib=cudart");
    println!("cargo:rustc-link-lib=cudadevrt");
    println!("cargo:rustc-link-lib=cublas");
    println!("cargo:rustc-link-lib=cufft");
    println!("cargo:rustc-link-lib=cusparse");
    println!("cargo:rustc-link-lib=curand");
    println!("cargo:rustc-link-lib=cusolver");
    println!("cargo:include=CEO/include");
    println!("cargo:include=/usr/local/cuda/include");
    println!("cargo:lib=CEO/lib");
    println!("cargo:rerun-if-changed=wrapper.hpp");
}
