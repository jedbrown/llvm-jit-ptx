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

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);
    let status = Command::new("llc")
        .args([
            "-mcpu=sm_86",
            "src/kernel.ll",
            "-o",
            out_dir.join("kernel.ptx").to_str().unwrap(),
        ])
        .status()
        .expect("Failed to run llc");
    assert!(status.success(), "Failed to compile kernel LLVM IR to PTX");
    println!("cargo:rerun-if-changed=src/kernel.ll");

    // Build dfunc crate
    println!("cargo:rerun-if-changed=dfunc/src/lib.rs");
    let mut command = Command::new(env::var("CARGO").unwrap())
        .args([
            "rustc",
            "--release", // Kernels are failing when debug is present
            "--package=dfunc",
            "--message-format=json-render-diagnostics",
            "--target=nvptx64-nvidia-cuda",
            "--target-dir=target/device",
            "--",
            "--emit=llvm-ir",
            // "-Zbuild-std",
        ])
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
                    if file.extension().unwrap() != "rmeta" {
                        continue;
                    };
                    // libdfunc-9294c53df299f0c6.rmeta -> dfunc-9294c53df299f0c6.ll
                    let ll = file
                        .file_name()
                        .unwrap()
                        .strip_prefix("lib")
                        .expect(&format!("Expected to start with lib: {file}"));
                    let ll = file.parent().unwrap().join(ll).with_extension("ll");
                    assert!(ll.exists(), "Missing LLVM-IR");
                    llvm_ir.insert(artifact.target.name.clone(), ll);
                }
            }
            // Message::BuildScriptExecuted(script) => {
            //     println!("{:?}", script);
            // }
            // Message::BuildFinished(finished) => {
            //     println!("{:?}", finished);
            // }
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
