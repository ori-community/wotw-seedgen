use std::{slice, sync::LazyLock};

use criterion::{criterion_group, criterion_main, Criterion};
use rand_pcg::Pcg64Mcg;
use wotw_seedgen_data::{
    assets::{AssetCacheValues, AssetFileAccess, SnippetAccess, TEST_ASSETS},
    logic_language::{ast::Paths, output::Graph},
    seed_language::{
        ast::Snippet,
        compile::{self, Compiler},
        output::{CommandInteger, CommandVoid, CommandsOutput},
        simulate::{Simulation, WorldState},
    },
    Difficulty, UberIdentifier, WorldSettings,
};

static PATHS: LazyLock<Paths> = LazyLock::new(|| {
    let source = TEST_ASSETS.values.paths();
    Paths::parse(&source.content).eprint_errors(source).unwrap()
});

fn logic_assets(c: &mut Criterion) {
    let mut group = c.benchmark_group("logic_assets");

    group.bench_function("paths", |b| {
        b.iter(|| Paths::parse(include_str!("../../assets/logic/paths.wotwl")))
    });

    let paths = &*PATHS;
    let loc_data = TEST_ASSETS.loc_data().unwrap();
    let state_data = TEST_ASSETS.state_data().unwrap();
    let settings = WorldSettings::difficulty_default(Difficulty::Unsafe);

    group.bench_function("compile", |b| {
        b.iter(|| {
            Graph::compiler()
                .with_settings(slice::from_ref(&settings))
                .compile(paths.clone(), loc_data.clone(), state_data.clone())
        })
    });

    group.finish();
}

fn snippets(c: &mut Criterion) {
    let mut group = c.benchmark_group("snippets");

    let stats = TEST_ASSETS.read_snippet("stats").unwrap();

    group.bench_function("ast_stats", |b| b.iter(|| Snippet::parse(&stats.content)));

    let available_snippets = TEST_ASSETS.available_snippets();
    let snippet_sources = available_snippets
        .iter()
        .map(|identifier| TEST_ASSETS.read_snippet(identifier).unwrap())
        .collect::<Vec<_>>();

    group.bench_function("ast_snippets", |b| {
        b.iter(|| {
            snippet_sources
                .iter()
                .map(|source| Snippet::parse(&source.content))
                .collect::<Vec<_>>()
        })
    });

    group.bench_function("compile_snippets", |b| {
        b.iter(|| {
            let mut rng = Pcg64Mcg::new(0);
            let mut compiler = Compiler::new(
                &mut rng,
                &*TEST_ASSETS,
                TEST_ASSETS.values.loc_data(),
                TEST_ASSETS.values.uber_state_data(),
            )
            .with_lint(true);

            for identifier in &available_snippets {
                compiler.compile_snippet(identifier).unwrap();
            }

            compiler.finish()
        })
    });

    group.finish();
}

fn simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("simulation");

    let mut world = WorldState::new(TEST_ASSETS.uber_states.clone(), &mut []);

    let uber_identifier = UberIdentifier::GORLEK_ORE;

    group.bench_function("gorlek_ore", |b| b.iter(|| world.gorlek_ore()));

    group.bench_function("fetch_integer ore", |b| {
        b.iter(|| world.fetch_integer(uber_identifier))
    });

    group.bench_function("fetch ore", |b| b.iter(|| world.fetch(uber_identifier)));

    group.bench_function("simulate fetch ore", |b| {
        b.iter(|| {
            world.simulate(
                &CommandInteger::FetchInteger { uber_identifier },
                &CommandsOutput::NONE,
            )
        })
    });

    group.bench_function("add_gorlek_ore", |b| {
        b.iter(|| world.add_gorlek_ore(1, &CommandsOutput::NONE))
    });

    group.bench_function("add_integer ore", |b| {
        b.iter(|| world.add_integer(uber_identifier, 1, &CommandsOutput::NONE))
    });

    group.bench_function("simulate ore", |b| {
        b.iter(|| world.simulate(&compile::gorlek_ore(), &CommandsOutput::NONE))
    });

    let mut compiler = Compiler::new(
        &mut Pcg64Mcg::new(0),
        &*TEST_ASSETS,
        TEST_ASSETS.values.loc_data(),
        TEST_ASSETS.values.uber_state_data(),
    );
    compiler.compile_snippet("launch_fragments").unwrap();
    let output = compiler.finish().output;
    let launch_fragment = output
        .modifiers
        .item_pool_changes
        .keys()
        .find(|item| matches!(item, CommandVoid::CallFunction { .. }))
        .unwrap();

    group.bench_function("simulate launch_fragment", |b| {
        b.iter(|| world.simulate(launch_fragment, &output.commands))
    });

    group.finish();
}

criterion_group!(all, logic_assets, snippets, simulation);
criterion_main!(all);
