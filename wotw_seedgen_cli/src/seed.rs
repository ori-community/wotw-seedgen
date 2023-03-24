use super::cli;
use super::log_init;
use super::play;

use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::time::Instant;

use log::LevelFilter;
use serde::{Deserialize, Serialize};

use wotw_seedgen::files::FILE_SYSTEM_ACCESS;
use wotw_seedgen::generator::{Seed, SeedSpoiler};
use wotw_seedgen::logic;
use wotw_seedgen::settings::UniverseSettings;

pub fn generate_seeds(args: cli::SeedArgs) -> Result<(), String> {
    let use_file = if args.verbose {
        Some("generator.log")
    } else {
        None
    };
    log_init::initialize_log(use_file, LevelFilter::Info, args.json_stderr)
        .unwrap_or_else(|err| eprintln!("Failed to initialize log: {}", err));

    let now = Instant::now();

    let mut universe_settings = UniverseSettings::default();

    let stdin = read_stdin()?;
    if !stdin.is_empty() {
        let preset = serde_json::from_str(&stdin)
            .map_err(|err| format!("Error parsing stdin as preset: {err}"))?;
        universe_settings
            .apply_preset(preset, &FILE_SYSTEM_ACCESS)
            .map_err(|err| format!("Error applying the stdin preset: {err}"))?;
    }

    parse_settings(args.settings, &mut universe_settings)?;

    let areas = fs::read_to_string(&args.areas)
        .map_err(|err| format!("Failed to read {}: {}", args.areas.display(), err))?;
    let locations = fs::read_to_string(&args.locations)
        .map_err(|err| format!("Failed to read {}: {}", args.locations.display(), err))?;
    let states = fs::read_to_string(&args.uber_states)
        .map_err(|err| format!("Failed to read {}: {}", args.uber_states.display(), err))?;
    let graph = logic::parse_logic(&areas, &locations, &states, &universe_settings, !args.trust)?;
    log::info!("Parsed logic in {:?}", now.elapsed());

    let worlds = universe_settings.world_count();
    let seed = wotw_seedgen::generate_seed(&graph, &FILE_SYSTEM_ACCESS, &universe_settings)
        .map_err(|err| format!("Error generating seed: {}", err))?;
    if worlds == 1 {
        log::info!("Generated seed in {:?}", now.elapsed());
    } else {
        log::info!("Generated {} worlds in {:?}", worlds, now.elapsed());
    }

    if args.tostdout {
        write_seeds_to_stdout(seed, args.json)?;
    } else {
        let filename = args.filename.unwrap_or_else(|| String::from("seed"));

        write_seeds_to_files(&seed, &filename, args.seed_folder, args.json)?;
    }

    if args.launch {
        if args.tostdout {
            log::warn!("Can't launch a seed that has been written to stdout");
        } else {
            play::play_last_seed()?;
        }
    }

    Ok(())
}

fn read_stdin() -> Result<String, String> {
    // If we do not have input, skip.
    if atty::is(atty::Stream::Stdin) {
        return Ok(String::new());
    }

    let stdin = io::stdin();
    let mut stdin = stdin.lock(); // locking is optional
    let mut output = String::new();

    loop {
        let result = stdin
            .read_to_string(&mut output)
            .map_err(|err| format!("failed to read standard input: {err}"))?;
        if result == 0 {
            break;
        }

        output.push('\n');
    }

    Ok(output)
}

fn parse_settings(
    args: cli::SeedSettings,
    universe_settings: &mut UniverseSettings,
) -> Result<(), String> {
    let preset = args.into_universe_preset()?;
    universe_settings
        .apply_preset(preset, &FILE_SYSTEM_ACCESS)
        .map_err(|err| format!("Error applying settings: {err}"))?;

    Ok(())
}

fn write_seeds_to_files(
    seed: &Seed,
    filename: &str,
    mut folder: PathBuf,
    json_spoiler: bool,
) -> Result<(), String> {
    let seeds = seed.seed_files()?;
    let multiworld = seeds.len() > 1;

    if multiworld {
        let mut multi_folder = folder.clone();
        multi_folder.push(filename);
        folder = create_multiworld_folder(multi_folder)
            .map_err(|err| format!("Error creating seed folder: {err}"))?;
    }

    let mut first = true;
    for (index, seed) in seeds.iter().enumerate() {
        let mut path = folder.clone();
        if multiworld {
            path.push(format!("world_{}", index));
        } else {
            path.push(filename);
        }
        path.set_extension("wotwr");

        let file =
            create_seedfile(path, seed).map_err(|err| format!("Error writing seed file: {err}"))?;
        log::info!("Wrote seed for World {} to {}", index, file.display());

        if first {
            first = false;
            if let Some(path) = file.to_str() {
                fs::write(".currentseedpath", path)
                    .unwrap_or_else(|err| log::warn!("Unable to write .currentseedpath: {}", err));
            } else {
                log::warn!("Unable to write .currentseedpath: path is not valid unicode");
            }
        }
    }

    let mut path = folder;
    path.push(format!("{filename}_spoiler"));

    let contents = match json_spoiler {
        true => {
            path.set_extension("json");
            seed.spoiler.to_json()
        }
        false => {
            path.set_extension("txt");
            seed.spoiler.to_string()
        }
    };

    let file =
        create_seedfile(path, &contents).map_err(|err| format!("Error writing spoiler: {err}"))?;
    log::info!("Wrote spoiler to {}", file.display());

    Ok(())
}

fn create_seedfile(path: PathBuf, contents: &str) -> Result<PathBuf, io::Error> {
    let mut index = 0;
    loop {
        let mut filename = path.file_stem().unwrap().to_os_string();
        if index > 0 {
            filename.push(format!("_{}", index));
        }
        let extension = path.extension().unwrap_or_default();
        let mut path = path.with_file_name(filename);
        path.set_extension(extension);

        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut file) => {
                file.write_all(contents.as_bytes())?;
                return Ok(path);
            }
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => index += 1,
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                fs::create_dir_all(path.parent().unwrap())?
            }
            Err(err) => return Err(err),
        }
    }
}
fn create_multiworld_folder(path: PathBuf) -> Result<PathBuf, io::Error> {
    let mut index = 0;
    loop {
        let mut filename = path.file_stem().unwrap().to_os_string();
        if index > 0 {
            filename.push(format!("_{}", index));
        }
        let path = path.with_file_name(filename);

        match fs::create_dir(&path) {
            Ok(_) => return Ok(path),
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => index += 1,
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                fs::create_dir_all(path.parent().unwrap())?
            }
            Err(err) => return Err(err),
        }
    }
}

fn write_seeds_to_stdout(seed: Seed, json: bool) -> Result<(), String> {
    let files = seed.seed_files()?;

    if json {
        let spoiler_text = seed.spoiler.to_string();
        let output = SeedgenCliJsonOutput {
            seed_files: files,
            spoiler: seed.spoiler,
            spoiler_text,
        };

        println!("{}", output.to_json())
    } else {
        if files.len() > 1 {
            for (index, file) in files.iter().enumerate() {
                println!("======= World {index} =======");
                println!("{file}");
            }
        } else {
            println!("{}", files[0]);
        }

        println!();
        println!("======= Spoiler =======");
        println!("{}", seed.spoiler);
    }

    Ok(())
}

/// Struct that is used for JSON output to stdout
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SeedgenCliJsonOutput {
    /// The seed file contents (i.e. text that goes into .wotwr files)
    pub seed_files: Vec<String>,
    /// Spoiler for this seed
    pub spoiler: SeedSpoiler,
    /// Text representation of the spoiler
    pub spoiler_text: String,
}

impl SeedgenCliJsonOutput {
    /// Serialize into json format
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}
