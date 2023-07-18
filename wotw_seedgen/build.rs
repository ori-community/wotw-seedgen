use std::{env, fs, process::Command};

const SETTINGS_PATH: &str = "src/settings/mod.rs";

// very hacky yes
fn main() {
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .unwrap();
    let git_hash = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);

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
