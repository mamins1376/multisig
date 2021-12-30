use std::env::var;

const PW: &str = "src/native/pw.c";

fn main() {
    if var("CARGO_CFG_TARGET_ARCH").unwrap() == "wasm32" {
        return;
    }

    let libs = system_deps::Config::new().probe().unwrap();

    let mut build = cc::Build::new();
    build.file(PW);

    println!("cargo:rerun-if-changed={}", PW);

    libs.all_defines()
        .iter()
        .map(|(k, v)| (k, v.as_ref().map(String::as_str)))
        .for_each(|(k, v)| drop(build.define(k, v)));

    build
        .includes(libs.all_include_paths().iter())
        .compile("pw");

    libs.iter()
        .into_iter()
        .map(|(_, l)| l.libs.iter())
        .flatten()
        .for_each(|l| println!("cargo:rustc-link-lib={}", l.name));
}
