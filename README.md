# CRisp: C and Rust implementations of a simple lisp language
Following the book ["Build Your Own Lisp"](http://www.buildyourownlisp.com/) by Daniel Holden.

This book describes writing a C implementation of a simple lisp-like language. The C implementation from the book is eventually going to be in [crisp/cisp](https://github.com/medium-endian/crisp/tree/master/cisp).

In [crisp/rusp](https://github.com/medium-endian/crisp/tree/master/rusp), I will try to get a similar program going, using the same parser generator and syntax as in the book, but written in Rust. This currently works with stable Rust, but in Nightly Rust there are benefits. First: warnings about 'improper C-Types' (types which are used in FFI but don't have #[repr(C)]) disappear. Rust-Bindgen asserts that all types are represented in a C-friendly way but there seems to be a problem in the linter that causes the warnings. See https://github.com/rust-lang/rust/issues/34798. This is fixed in nightly Rust.
Second, the LLVM leak sanitizer (and all other sanitizers) only work in nightly.
