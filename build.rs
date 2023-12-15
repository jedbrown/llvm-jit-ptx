use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    cc::Build::new()
        .cpp(true)
        .include("/opt/cuda/include")
        .file("src/main.cpp")
        .compile("main_cpp");
    println!("cargo:rustc-link-lib=dylib=cuda");
    println!("cargo:rerun-if-changed=src/main.cpp");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);
    let status = Command::new("llc")
        .args([
            "-mcpu=sm_86",
            "src/kernel.ll",
            "-o",
            out_dir.join("kernel.ptx").to_str().unwrap(),
        ])
        .status()
        .expect("Failed run llc");
    assert!(status.success(), "Failed to compile kernel LLVM IR to PTX");
    println!("cargo:rerun-if-changed=src/kernel.ll");
}
