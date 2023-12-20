use dfunc;
use std::ffi::{c_char, CString};
use std::process::{Command, Stdio};

extern "C" {
    fn main_cpp(kernel_ptx: *const c_char) -> i32;
}

fn main() {
    println!("Rust can math: 1.0 + 2.0^3 = {}", dfunc::add(1.0, 2.0));
    let out_dir = env!("OUT_DIR");

    let kernel_ptx = CString::new(format!("{}/{}", out_dir, "kernel.ptx")).unwrap();
    println!(
        "Kernel from ptx generated in build.rs: {}",
        kernel_ptx.to_string_lossy()
    );
    unsafe { main_cpp(kernel_ptx.as_ptr()) };

    let ll = env!("CARGO_LLVM_IR_FILE_DFUNC");
    println!("Kernel from online ptx: {}", kernel_ptx.to_string_lossy());
    let mut bitcode = Command::new("llvm-link")
        .args(["src/kernel_only.ll", ll])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let ptx = Command::new("llc")
        .args(["-mcpu=sm_86", "-o", "kernel_dfunc_online.ptx"])
        .stdin(bitcode.stdout.take().unwrap())
        .status()
        .unwrap();
    assert!(ptx.success());
    let kernel_ptx = CString::new("kernel_dfunc_online.ptx").unwrap();
    unsafe { main_cpp(kernel_ptx.as_ptr()) };
}
