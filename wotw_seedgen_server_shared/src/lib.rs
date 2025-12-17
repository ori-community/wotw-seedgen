mod error;

use tokio::{runtime::Runtime, sync::RwLock};
use wotw_seedgen_assets::{
    AssetCache, AssetCacheValues, AssetFileAccess, PresetFileAccess, SnippetFileAccess, Watcher,
};

pub use crate::error::{Error, Result};

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

pub type ServerState<F, V> = Arc<RwLock<AssetCache<F, V>>>;

pub fn start<F, V>(cache: AssetCache<F, V>) -> Result<(Runtime, ServerState<F, V>)>
where
    F: AssetFileAccess + SnippetFileAccess + PresetFileAccess + Send + Sync + 'static,
    V: AssetCacheValues + Send + Sync + 'static,
{
    let start = Instant::now();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(Error::BuildRuntime)?;

    eprintln!("Started runtime");

    let mut watcher = Watcher::new(Duration::from_secs(1))?;

    cache.watch(&mut watcher)?;

    let state = Arc::new(RwLock::new(cache));
    runtime.spawn(watch_assets(state.clone(), watcher));

    eprintln!("Loaded assets");

    eprintln!("Server ready in {:.2}s", start.elapsed().as_secs_f32());

    Ok((runtime, state))
}

pub async fn watch_assets<F, V>(state: Arc<RwLock<AssetCache<F, V>>>, watcher: Watcher)
where
    F: AssetFileAccess + SnippetFileAccess + PresetFileAccess,
    V: AssetCacheValues,
{
    for res in watcher {
        or_print(
            (async || {
                let events = res?;

                let mut cache = state.write().await;

                let any_changed = cache
                    .update_from_watcher_event(&events)
                    .map_err(Error::LoadAssets)?;

                if any_changed {
                    eprintln!("Reloaded assets");
                }

                Ok(())
            })()
            .await,
        );
    }
}

fn or_print(res: Result<()>) {
    if let Err(err) = res {
        eprintln!("error in file watcher: {err}");
    }
}
