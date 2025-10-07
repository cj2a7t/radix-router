fn main() {
    // Compile C source files
    cc::Build::new()
        .file("c_src/rax.c")
        .file("c_src/easy_rax.c")
        .include("c_src")
        .warnings(false) // Suppress C code warnings
        .compile("radixtree");

    // Tell cargo to rerun build script if C files change
    println!("cargo:rerun-if-changed=c_src/rax.c");
    println!("cargo:rerun-if-changed=c_src/easy_rax.c");
    println!("cargo:rerun-if-changed=c_src/rax.h");
    println!("cargo:rerun-if-changed=c_src/easy_rax.h");
}

