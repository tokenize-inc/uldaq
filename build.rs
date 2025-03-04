use std::env::VarError;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::io;
use std::io::read_to_string;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use bindgen::{builder, BindgenError};

#[derive(Debug)]
struct UldaqBuilderError {
    message: String,
}

impl Error for UldaqBuilderError {}

impl Display for UldaqBuilderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<io::Error> for UldaqBuilderError {
    fn from(value: io::Error) -> Self {
        Self {
            message: format!("Failed due to IO error: {}", value),
        }
    }
}

impl From<BindgenError> for UldaqBuilderError {
    fn from(value: BindgenError) -> Self {
        Self {
            message: format!("Failed due to bindgen error: {}", value),
        }
    }
}

impl From<VarError> for UldaqBuilderError {
    fn from(value: VarError) -> Self {
        Self {
            message: format!("Failed due to missing environment variable: {}", value),
        }
    }
}

fn main() -> ExitCode {
    if let Err(e) = better_main() {
        eprintln!("Generation failed!\n{}", e);
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn better_main() -> Result<(), UldaqBuilderError> {
    let src_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?).join("src");
    // Re-run this build.rs if the src changes (i.e. file is added/modified)
    println!("cargo:rerun-if-changed={}", src_dir.to_string_lossy());

    generate_bindings(&src_dir)?;
    compile(&src_dir)?;

    println!("cargo:rustc-link-lib=usb-1.0");
    println!("cargo:rustc-link-lib=dylib=stdc++");

    Ok(())
}

fn build_source_list(src_dir: &Path) -> Result<Vec<String>, UldaqBuilderError> {
    Ok(read_to_string(File::open(src_dir.join("Makefile.am"))?)?
        .lines()
        .filter_map(|line| {
            if line.starts_with("libuldaq_la_SOURCES = ") {
                Some(
                    line.split_once(" = ")
                        .unwrap_or(("", ""))
                        .1
                        .split_whitespace()
                        .filter_map(|s| {
                            if s.ends_with(".h") {
                                None
                            } else {
                                Some(s.to_string())
                            }
                        })
                        .collect(),
                )
            } else {
                None
            }
        })
        .next()
        .unwrap_or_default())
}

fn generate_bindings(src_dir: &Path) -> Result<(), UldaqBuilderError> {
    // Configure and generate bindings.
    let bindings = builder()
        .header(src_dir.join("uldaq.h").to_string_lossy())
        .generate_cstr(true)
        .clang_arg(format!("-I{}", src_dir.to_string_lossy()))
        .generate()?;

    // Write the generated bindings to an output file.
    bindings.write_to_file(PathBuf::from(std::env::var("OUT_DIR")?).join("uldaq.rs"))?;
    Ok(())
}

fn compile(src_dir: &Path) -> Result<(), UldaqBuilderError> {
    cc::Build::new()
        .includes([src_dir.to_path_buf()])
        .files(
            build_source_list(src_dir)?
                .iter()
                .map(|file| src_dir.join(file)),
        )
        .flag("-w")
        .flag("-Wno-address") // Some weird bug in Clang where part of this warning is getting printed even when all warnings are disabled
        .compile("uldaq");
    Ok(())
}
