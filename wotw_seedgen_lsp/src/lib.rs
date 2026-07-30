mod backend;
mod convert;
mod error;
mod logic;
mod seed;

use tower_lsp::{LanguageServer, LspService, Server};
use wotw_seedgen_data::assets::{
    AssetCache, AssetCacheValues, AssetFileAccess, DefaultFileAccess, PresetFileAccess,
    SnippetFileAccess,
};
use wotw_seedgen_server_shared::ServerState;

use crate::{
    backend::Backend, logic::cache::CacheValues as LogicCacheValues,
    seed::cache::CacheValues as SeedCacheValues,
};

pub fn start_seed() {
    start(AssetCache::<_, SeedCacheValues>::new(DefaultFileAccess).unwrap());
}

pub fn start_logic() {
    start(AssetCache::<_, LogicCacheValues>::new(DefaultFileAccess).unwrap());
}

fn start<F, V>(cache: AssetCache<F, V>)
where
    F: AssetFileAccess + SnippetFileAccess + PresetFileAccess + Send + Sync + 'static,
    V: AssetCacheValues + Send + Sync + 'static,
    Backend<ServerState<F, V>>: LanguageServer,
{
    let (runtime, cache) = wotw_seedgen_server_shared::start(cache).unwrap();

    runtime.block_on(async {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();

        let (service, socket) = LspService::build(|client| Backend::new(client, cache)).finish();

        Server::new(stdin, stdout, socket).serve(service).await;
    });
}
