use std::ffi::{c_char, CString};

extern "C" {
    fn main_cpp(kernel_ptx: *const c_char) -> i32;
}

fn main() {
    println!("Hello from Rust!");
    let out_dir = env!("OUT_DIR");
    let kernel_ptx = CString::new(format!("{}/{}", out_dir, "kernel.ptx")).unwrap();
    unsafe { main_cpp(kernel_ptx.as_ptr()) };
}
