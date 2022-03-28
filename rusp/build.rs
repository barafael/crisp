use bindgen::RustTarget;
use std::env;
use std::path::PathBuf;

fn main() {
    // Let the gcc crate handle all the C library compilation and linking
    cc::Build::new().file("mpc/mpc.c").compile("mpc");

    // Use the bindgen builder create a binding, adding options
    let bindings = bindgen::Builder::default()
        // .raw_line("#[allow(improper_ctypes)]") // what does this do?
        .generate_comments(true)
        // Output bindings for builtin definitions, e.g. __builtin_va_list (which mpc uses)
        .rust_target(RustTarget::Nightly)
        .emit_builtins()
        .derive_default(true)
        .blocklist_function("strtold")
        .blocklist_function("gcvt")
        .blocklist_function("qecvt")
        .blocklist_function("qfcvt")
        .blocklist_function("qgcvt")
        .blocklist_function("ecvt_r")
        .blocklist_function("fcvt_r")
        .blocklist_function("qfcvt_r")
        .blocklist_function("qecvt_r")
        // The input header we would like to generate bindings for
        .header("mpc/mpc.h")
        // Finish the builder and generate the bindings
        .generate()
        // Unwrap the Result and panic on failure
        .expect("Unable to generate bindings!");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
