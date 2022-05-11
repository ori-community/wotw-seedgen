use std::fs;

const SETTINGS_PATH: &str = "src/settings/mod.rs";

// very hacky yes
fn main() {
    println!("cargo:rerun-if-changed={SETTINGS_PATH}");

    let source = fs::read_to_string(SETTINGS_PATH).unwrap_or_else(|_| panic!("failed to read {}", SETTINGS_PATH));
    let syntax = syn::parse_file(&source).expect("failed to parse settings source");

    let trick_enum = find_enum(&syntax, "Trick").expect("failed to locate Trick enum in settings source");
    let trick_list = list_variants(trick_enum);

    let preset = format!("{{\"tricks\":[{}]}}", trick_list.join(","));
    fs::write("presets/glitches.json", preset).expect("failed to write glitches preset");
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
    item_enum.variants.iter().map(|variant| format!("\"{}\"", variant.ident)).collect()
}
