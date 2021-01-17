use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    // Detect libraries with pkg-config
    let libopenraw = pkg_config::Config::new().probe("libopenraw-0.3").unwrap();

    // Tell cargo to tell rustc to link the system libopenraw shared library
    for lib in &libopenraw.libs {
        println!("cargo:rustc-link-lib={}", lib);
    }

    // The bindgen::Builder is the main entry point to bindgen, and lets you
    // build up options for the resulting bindings
    let mut builder = bindgen::Builder::default();

    // Add include paths
    for path in &libopenraw.include_paths {
        let strpath = path.to_str().expect("Invalid unicode in include path");
        let arg = format!("-I{}", strpath);
        builder = builder.clang_arg(arg);
    }

    // The input header we would like to generate bindings for
    builder = builder.header("wrapper.h");

    // The symbols we want to generate wrappers for
    let included_functions = [
        "or_rawfile_new",
        "or_rawfile_get_orientation",
        "or_rawfile_get_thumbnail",
        "or_rawfile_release",
        "or_thumbnail_new",
        "or_thumbnail_format",
        "or_thumbnail_data_size",
        "or_thumbnail_data",
        "or_thumbnail_release",
    ];
    for func in &included_functions {
        builder = builder.whitelist_function(func);
    }

    // Generate bindings
    let bindings = builder
        // Generate module based enums
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings
        .generate()
        // Unwrap the Result and panic on failure
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
