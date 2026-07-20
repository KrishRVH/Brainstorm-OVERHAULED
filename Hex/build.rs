#![allow(clippy::panic)]

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/cuda/brainstorm_cuda.cu");
    println!("cargo:rerun-if-env-changed=BRAINSTORM_SKIP_CUDA_BUILD");
    println!("cargo:rerun-if-env-changed=BRAINSTORM_CUDA_ARCHES");
    println!("cargo:rerun-if-env-changed=BRAINSTORM_CUDA_PTX_ARCH");
    println!("cargo:rerun-if-env-changed=CUDAHOSTCXX");
    println!("cargo:rerun-if-env-changed=NVCC");

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by cargo"));
    let module_path = out_dir.join("brainstorm_cuda.fatbin");

    if env::var_os("BRAINSTORM_SKIP_CUDA_BUILD").is_some_and(|value| value == "1") {
        std::fs::write(&module_path, b"").expect("write empty CUDA module");
        return;
    }

    let nvcc = env::var("NVCC").unwrap_or_else(|_| "nvcc".to_owned());
    let ccbin = env::var("CUDAHOSTCXX").unwrap_or_else(|_| "gcc-12".to_owned());
    let arches = env::var("BRAINSTORM_CUDA_ARCHES")
        .unwrap_or_else(|_| "50,52,60,61,70,75,80,86,89,90".to_owned());
    let ptx_arch = env::var("BRAINSTORM_CUDA_PTX_ARCH").unwrap_or_else(|_| "89".to_owned());

    let mut command = Command::new(&nvcc);
    command.args([
        "--fatbin",
        "-O3",
        "--Werror",
        "all-warnings",
        "--std=c++17",
        "--fmad=false",
        "-Xptxas",
        "-fmad=false",
        "-prec-div=true",
        "-prec-sqrt=true",
        "-ccbin",
        &ccbin,
    ]);
    for arch in arches.split(',').map(str::trim) {
        validate_arch(arch);
        command.arg(format!(
            "--generate-code=arch=compute_{arch},code=sm_{arch}"
        ));
    }
    validate_arch(&ptx_arch);
    command.arg(format!(
        "--generate-code=arch=compute_{ptx_arch},code=compute_{ptx_arch}"
    ));
    let status = command
        .arg("-o")
        .arg(&module_path)
        .arg("src/cuda/brainstorm_cuda.cu")
        .status();

    match status {
        Ok(status) if status.success() => {},
        Ok(status) => {
            panic!(
                "nvcc failed with status {status}. Install CUDA Toolkit 12.4 or set BRAINSTORM_SKIP_CUDA_BUILD=1"
            );
        },
        Err(err) => {
            panic!(
                "failed to run {nvcc}: {err}. Install CUDA Toolkit 12.4 or set BRAINSTORM_SKIP_CUDA_BUILD=1"
            );
        },
    }
}

fn validate_arch(arch: &str) {
    assert!(
        !arch.is_empty() && arch.bytes().all(|byte| byte.is_ascii_digit()),
        "CUDA architecture must contain only digits, found {arch:?}"
    );
}
