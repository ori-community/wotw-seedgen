use serde::Deserialize;
use std::{fs, io::ErrorKind};
// use vergen::EmitBuilder;

#[derive(Deserialize)]
struct PackageMeta {
    git: PackageMetaGit,
}

#[derive(Deserialize)]
struct PackageMetaGit {
    sha1: String,
}

fn main() {
    match fs::read_to_string(".cargo_vcs_info.json") {
        Ok(package_meta) => {
            let package_meta: PackageMeta = serde_json::from_str(&package_meta).unwrap();
            println!("cargo:rustc-env=VERGEN_GIT_SHA={}", package_meta.git.sha1);
        }
        Err(err) if matches!(err.kind(), ErrorKind::NotFound) => {
            // TODO
            // EmitBuilder::builder()
            //     .git_sha(false)
            //     .fail_on_error()
            //     .emit_and_set()
            //     .unwrap();
        }
        Err(err) => panic!("{err:?}"),
    }
}
