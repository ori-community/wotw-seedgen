use std::{env, fs, io::ErrorKind};

use serde::Deserialize;
use vergen::EmitBuilder;

const SETTINGS_PATH: &str = "src/settings/mod.rs";

#[derive(Deserialize)]
struct PackageMeta {
    git: PackageMetaGit,
}
#[derive(Deserialize)]
struct PackageMetaGit {
    sha1: String,
}

// very hacky yes
fn main() {
    match fs::read_to_string(".cargo_vcs_info.json") {
        Ok(package_meta) => {
            let package_meta: PackageMeta = serde_json::from_str(&package_meta).unwrap();
            println!("cargo:rustc-env=VERGEN_GIT_SHA={}", package_meta.git.sha1);
        }
        Err(err) if matches!(err.kind(), ErrorKind::NotFound) => {
            EmitBuilder::builder()
                .git_sha(false)
                .fail_on_error()
                .emit_and_set()
                .unwrap();
        }
        Err(err) => panic!("{err:?}"),
    }

    // TODO I think the issue here was writing arbitrary files, this might be solvable by properly structuring the build output
    if env::var("DOCS_RS").is_ok() {
        return;
    }

    let source = fs::read_to_string(SETTINGS_PATH)
        .unwrap_or_else(|_| panic!("failed to read {}", SETTINGS_PATH));
    let syntax = syn::parse_file(&source).expect("failed to parse settings source");

    let trick_enum =
        find_enum(&syntax, "Trick").expect("failed to locate Trick enum in settings source");
    let trick_list = list_variants(trick_enum);

    let preset = format!(
        concat!(
            "{{\n",
            "  \"info\": {{\n",
            "    \"name\": \"Glitches\",\n",
            "    \"description\": \"Requires glitches\"\n",
            "  }},\n",
            "  \"tricks\": [\n",
            "    {}\n",
            "  ]\n",
            "}}",
        ),
        trick_list.join(",\n    ")
    );
    fs::write("world_presets/glitches.json", preset).expect("failed to write glitches preset");
}

fn find_enum<'a>(syntax: &'a syn::File, ident: &str) -> Option<&'a syn::ItemEnum> {
    syntax.items.iter().find_map(|item| {
        if let syn::Item::Enum(item_enum) = item {
            if item_enum.ident == ident {
                return Some(item_enum);
            }
        }
        None
    })
}
fn list_variants(item_enum: &syn::ItemEnum) -> Vec<String> {
    item_enum
        .variants
        .iter()
        .map(|variant| format!("\"{}\"", variant.ident))
        .collect()
}
