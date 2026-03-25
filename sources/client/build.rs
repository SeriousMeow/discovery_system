use std::process::Command;

const OPENAPI_PATH: &str = "../../openapi/api.yaml";

fn main() {
    println!("cargo::rerun-if-changed={OPENAPI_PATH}");

    let status = match Command::new("oas3-gen")
        .args([
            "generate",
            "client-mod",
            "-i",
            OPENAPI_PATH,
            "-o",
            "src/api",
        ])
        .status()
    {
        Ok(status) => status,
        Err(error) => {
            println!("cargo:warning=failed to run oas3-gen ({error})");
            return;
        }
    };

    if !status.success() {
        panic!("oas3-gen failed with status: {status}");
    }
}
