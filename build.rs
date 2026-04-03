//! 
//! Copyright (c) 2019 Embark Studios
//! Copyright (c) 2025 Matis Granger <matis@devyos.com>
//!
//! Permission is hereby granted, free of charge, to any
//! person obtaining a copy of this software and associated
//! documentation files (the "Software"), to deal in the
//! Software without restriction, including without
//! limitation the rights to use, copy, modify, merge,
//! publish, distribute, sublicense, and/or sell copies of
//! the Software, and to permit persons to whom the Software
//! is furnished to do so, subject to the following
//! conditions:
//! 
//! The above copyright notice and this permission notice
//! shall be included in all copies or substantial portions
//! of the Software.
//! 
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
//! ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
//! TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
//! PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
//! SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
//! CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
//! OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
//! IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
//! DEALINGS IN THE SOFTWARE.
//!

use std::env;
use std::path::PathBuf;

fn main() {
    let mut path: PathBuf = PathBuf::from(&env::var("CARGO_MANIFEST_DIR").unwrap());
    path.push("rpmalloc");

    if pkg_config::find_library("librpmalloc").is_ok() {
        return;
    }

    let mut build = cc::Build::new();
    let c_file = path.join("rpmalloc.c");
    println!("cargo:rerun-if-changed={}", c_file.display());
    let mut build = build.file(c_file).opt_level(2);
    // add defines for enabled features

    #[rustfmt::skip]
    let features = [
        // ( "ENABLE_PRELOAD", cfg!(feature = "preload") ),
        ( "ENABLE_STATISTICS", cfg!(feature = "statistics") ),
        // ( "ENABLE_VALIDATE_ARGS", cfg!(feature = "validate_args") ),
        ( "ENABLE_ASSERTS", cfg!(feature = "asserts") ),
        ( "ENABLE_GUARDS", cfg!(feature = "guards") ),
        ( "ENABLE_UNLIMITED_CACHE", cfg!(feature = "unlimited_cache") ),
        ( "ENABLE_UNLIMITED_GLOBAL_CACHE", cfg!(feature = "unlimited_global_cache") ),
        ( "ENABLE_UNLIMITED_THREAD_CACHE", cfg!(feature = "unlimited_thread_cache") ),
        ( "ENABLE_GLOBAL_CACHE", cfg!(feature = "global_cache") ),
        ( "ENABLE_THREAD_CACHE", cfg!(feature = "thread_cache") ),
        ( "ENABLE_ADAPTIVE_THREAD_CACHE", cfg!(feature = "adaptive_thread_cache") ),
        ( "RPMALLOC_FIRST_CLASS_HEAPS", cfg!(feature = "first_class_heaps") ),
    ];

    for (name, value) in features.iter() {
        if *value {
            build = build.define(name, "1");
        }
    }

    // set platform-specific compile and link flags

    match env::var("CARGO_CFG_TARGET_OS").unwrap().as_str() {
        "linux" => {
            build = build.define("_GNU_SOURCE", "1");
            println!("cargo:rustc-link-lib=pthread");
        }
        "macos" => {
            build = build
                .flag("-Wno-padded")
                .flag("-Wno-documentation-unknown-command")
                .flag("-Wno-static-in-inline")
                .flag("-Wno-unused-parameter");
        }
        _ => (),
    }

    build.compile("librpmalloc.a");
}
