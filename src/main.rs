use dfunc;
use std::ffi::{c_char, CString};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

extern "C" {
    fn main_cpp(kernel_ptx: *const c_char) -> i32;
}

fn main() {
    println!("Rust can swirl(1.0, 2.0) = {}", dfunc::swirl(1.0, 2.0));
    println!("Rust can d_swirl(1.0, 2.0) = {}", dfunc::d_swirl(1.0, 2.0));
    let capability = env!("CARGO_CUDA_COMPUTE_CAPABILITY");
    let out_dir = env!("OUT_DIR");

    let kernel_ptx = CString::new(format!("{}/{}", out_dir, "kernel.ptx")).unwrap();
    println!(
        "Kernel from ptx generated in build.rs: {}",
        kernel_ptx.to_string_lossy()
    );
    unsafe { main_cpp(kernel_ptx.as_ptr()) };

    let kernel_ptx = CString::new("kernel.ptx").unwrap();
    println!("kernel.ptx: {}", kernel_ptx.to_string_lossy());
    unsafe { main_cpp(kernel_ptx.as_ptr()) };

    if false {
        let kernel_ptx = jit_compile_ptx(Path::new("src/kernel_only.ll"), capability);
        println!("Kernel from online ptx: {}", kernel_ptx.to_string_lossy());
        let kernel_ptx = CString::new(kernel_ptx.to_str().unwrap()).unwrap();
        unsafe { main_cpp(kernel_ptx.as_ptr()) };

        let kernel_ptx = jit_compile_ptx(Path::new("src/kernel_only_d.ll"), capability);
        println!("Kernel from online ptx: {}", kernel_ptx.to_string_lossy());
        let kernel_ptx = CString::new(kernel_ptx.to_str().unwrap()).unwrap();
        unsafe { main_cpp(kernel_ptx.as_ptr()) };
    }
}

fn jit_compile_ptx(kernel_only: &Path, capability: &str) -> PathBuf {
    let ptx_output = kernel_only
        .with_extension("dfunc.ptx")
        .strip_prefix("src")
        .unwrap()
        .to_path_buf();
    // let ll = env!("CARGO_LLVM_IR_FILE_DFUNC");
    let ll = "target/device/nvptx64-nvidia-cuda/release/libdfunc.rlib";
    //    let ll = "src/dfunc-cuda-nvptx64-nvidia-cuda-sm_86.ll";
    let mut bitcode = Command::new("llvm-link")
        .args([
            kernel_only.to_str().unwrap(),
            ll,
            "/opt/cuda/nvvm/libdevice/libdevice.10.bc",
            "--ignore-non-bitcode",
            "--internalize",
            "--only-needed",
        ])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut opt = Command::new("opt")
        .args([
            "-passes=internalize,inline",
            "--internalize-public-api-list=kernel",
        ])
        .stdin(bitcode.stdout.take().unwrap())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let ptx = Command::new("llc")
        .arg("-O3")
        .arg(format!("-mcpu=sm_{capability}"))
        .arg("-o")
        .arg(ptx_output.to_str().unwrap())
        .stdin(opt.stdout.take().unwrap())
        .status()
        .unwrap();
    assert!(ptx.success());
    ptx_output
}
