use bindgen;

fn main() {
    let bindings = bindgen::Builder::default()
        .header("wrapper.hpp")
        .clang_arg("-I/usr/local/cuda/include")
        .clang_arg("-v")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .whitelist_type("gpu_float")
        .whitelist_type("gpu_double")
        .whitelist_type("mask")
        .whitelist_function("set_device")
        .whitelist_function("host2dev_char")
        .whitelist_function("host2dev")
        .whitelist_function("dev2host")
        .whitelist_function("dev2host_int")
        .whitelist_type("source")
        .whitelist_type("pssn")
        .whitelist_type("centroiding")
        .whitelist_type("imaging")
        .whitelist_type("shackHartmann")
        .whitelist_type("geometricShackHartmann")
        .whitelist_type("coordinate_system")
        .whitelist_type("gmt_m1")
        .whitelist_type("gmt_m2")
        .whitelist_type("atmosphere")
        .whitelist_type("LMMSE")
        .whitelist_type("pyramid")
        .whitelist_type("conic")
        .whitelist_function("transform_to_S")
        .whitelist_function("transform_to_R")
        .whitelist_function("intersect")
        .whitelist_function("intersect")
        .whitelist_function("reflect")
        .whitelist_function("refract")
        .whitelist_function("geqrf")
        .whitelist_function("ormqr")
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
