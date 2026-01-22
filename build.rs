// SPDX-License-Identifier: LGPL-3.0-or-later
//! Build script for guestkit
//!
//! This generates Rust FFI bindings to libguestfs using bindgen

use std::env;
use std::path::PathBuf;

fn main() {
    // Only build FFI bindings if the feature is enabled
    #[cfg(feature = "ffi-bindings")]
    {
        println!("cargo:rerun-if-changed=wrapper.h");

        // Try to find libguestfs using pkg-config
        match pkg_config::probe_library("libguestfs") {
            Ok(library) => {
                // Found libguestfs - generate bindings
                generate_bindings(&library);
            }
            Err(e) => {
                // libguestfs not found - emit warning but don't fail the build
                // This allows the rest of guestkit to build without libguestfs
                eprintln!("Warning: libguestfs not found via pkg-config: {}", e);
                eprintln!("FFI bindings will not be generated.");
                eprintln!("To use libguestfs features, install libguestfs-devel:");
                eprintln!("  sudo dnf install libguestfs-devel  # Fedora/RHEL");
                eprintln!("  sudo apt install libguestfs-dev    # Ubuntu/Debian");
            }
        }
    }
}

#[cfg(feature = "ffi-bindings")]
fn generate_bindings(_library: &pkg_config::Library) {
    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Allow specific types and functions
        .allowlist_function("guestfs_.*")
        .allowlist_type("guestfs_.*")
        .allowlist_var("GUESTFS_.*")
        // Generate comments from doc strings
        .generate_comments(true)
        // Use core instead of std for no_std compatibility
        .use_core()
        // Derive common traits
        .derive_debug(true)
        .derive_default(true)
        .derive_eq(true)
        .derive_hash(true)
        .derive_ord(true)
        .derive_partialeq(true)
        .derive_partialord(true)
        // Size of types
        .size_t_is_usize(true)
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:warning=Generated libguestfs bindings successfully");
}
