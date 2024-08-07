use cargo_metadata::Message;
use std::{
    collections::HashMap,
    env,
    path::Path,
    process::{Command, Stdio},
};

fn main() {
    cc::Build::new()
        .cpp(true)
        .include("/opt/cuda/include")
        .file("src/main.cpp")
        .compile("main_cpp");
    println!("cargo:rustc-link-lib=dylib=cuda");
    println!("cargo:rerun-if-changed=src/main.cpp");
    println!("cargo::rustc-check-cfg=cfg(enzyme)");

    let capability = cuda_device_capability();
    println!("cargo:rustc-env=CARGO_CUDA_COMPUTE_CAPABILITY={capability}");

    let feature_enzyme = env::var("CARGO_FEATURE_ENZYME_DEVICE").is_ok();
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);
    let status = Command::new("llc")
        .arg(format!("-mcpu=sm_{capability}"))
        .arg("src/kernel.ll")
        .arg("-o")
        .arg(out_dir.join("kernel.ptx").to_str().unwrap())
        .status()
        .expect("Failed to run llc");
    assert!(status.success(), "Failed to compile kernel LLVM IR to PTX");
    println!("cargo:rerun-if-changed=src/kernel.ll");

    // Build dfunc crate
    println!("cargo:rerun-if-changed=dfunc/src/lib.rs");
    let mut command = Command::new(env::var("CARGO").unwrap())
        .arg("rustc")
        .arg("--release") // Kernels are failing when debug is present and we
        // don't do it on GPU anyway
        .arg("--package=dfunc")
        .arg(if feature_enzyme {
            "--features=enzyme"
        } else {
            "--features="
        })
        .arg("--message-format=json-render-diagnostics")
        .arg("--target=nvptx64-nvidia-cuda")
        .arg("--target-dir=target/device") // To avoid lock conflict with outer cargo
        //.arg("-Zbuild-std")
        .arg("--")
        .arg("-C")
        .arg(format!("target-cpu=sm_{capability}"))
        .arg("--emit=llvm-ir")
        .arg("-Clto=fat")
        .arg("-Cembed-bitcode=yes")
        .arg("-Zdylib-lto")
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn cargo build for dfunc on device");
    let reader = std::io::BufReader::new(command.stdout.take().unwrap());
    let mut llvm_ir = HashMap::new();
    for message in cargo_metadata::Message::parse_stream(reader) {
        match message.unwrap() {
            Message::CompilerMessage(msg) => {
                println!("{:?}", msg);
            }
            Message::CompilerArtifact(artifact) => {
                println!("{:?}", artifact);
                for file in artifact.filenames {
                    if !file.extension().is_some_and(|ext| ext == "rmeta") {
                        continue;
                    };
                    // libdfunc-9294c53df299f0c6.rmeta -> dfunc-9294c53df299f0c6.ll
                    let ll = file
                        .file_name()
                        .unwrap()
                        .strip_prefix("lib")
                        .expect(&format!("Expected to start with lib: {file}"));
                    let ll = file.parent().unwrap().join(ll).with_extension("ll");
                    if !ll.exists() {
                        eprintln!("Ignoring missing LLVM-IR: {}", ll);
                        continue;
                    }
                    llvm_ir.insert(artifact.target.name.clone(), ll);
                }
            }
            _ => (), // Unknown message
        }
    }
    let output = command.wait().expect("Couldn't get cargo's exit status");
    assert!(output.success(), "Building dfunc for device failed");
    println!("# LLVM-IR: {llvm_ir:?}");

    for (name, ll) in llvm_ir {
        println!(
            "cargo:rustc-env=CARGO_LLVM_IR_FILE_{}={}",
            name.to_uppercase(),
            ll
        );
    }
}

fn cuda_device_capability() -> i32 {
    let capability = Command::new("nvidia-smi")
        .args(["--query-gpu=compute_cap", "--format=csv,noheader"])
        .output()
        .expect("Failed to execute nvidia-smi to determine compute capability");
    // "8.6\n" (bytes)
    let capability = String::from_utf8(capability.stdout)
        .unwrap()
        .trim_end()
        .parse::<f64>()
        .expect("Failed to parse compute capability");
    (capability * 10.) as i32 // 86
}
