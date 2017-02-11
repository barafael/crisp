extern crate bindgen;
extern crate gcc;

use std::env;
use std::path::PathBuf;

fn main() {
    gcc::compile_library("libmpc.a", &["mpc/mpc.c"]);

    // Tell cargo to tell rustc to link mpc library
    //println!("cargo:rustc-link-lib=libmpc.a");

    // The bindgen::Builder is the main entry point to bindgen, and lets you build up options for
    // the resulting bindings

    let bindings = bindgen::Builder::default()
        // Emit no unstable/nightly Rust code
        .no_unstable_rust()
        // The input header we would like to generate bindings for
        .header("mpc/mpc.h")
        // Finish the builder and generate the bindings
        .generate()
        // Unwrap the Result and panic on failure
        .expect("Unable to generate bindings!");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
