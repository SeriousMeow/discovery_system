use std::process::Command;

const MESSENGER_OPENAPI_PATH: &str = "../../openapi/messanger.yaml";

fn main() {
    println!("cargo::rerun-if-changed={MESSENGER_OPENAPI_PATH}");

    let status = match Command::new("oas3-gen")
        .args([
            "generate",
            "server-mod",
            "-i",
            MESSENGER_OPENAPI_PATH,
            "-o",
            "src/api/api_impl",
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
