use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use itertools::Itertools;
use notify_debouncer_full::{
    notify::{self, EventKind, RecommendedWatcher, RecursiveMode},
    DebounceEventResult, DebouncedEvent, Debouncer, RecommendedCache,
};
use thiserror::Error;

pub struct Watcher {
    sender: Debouncer<RecommendedWatcher, RecommendedCache>,
}

pub type WatcherResult<T> = Result<T, WatcherError>;
pub type WatcherMessage = WatcherResult<Vec<DebouncedEvent>>;

#[derive(Debug, Error)]
pub enum WatcherError {
    #[error("failed to configure file watcher: {error}", error = .0)]
    Build(notify::Error),
    #[error("failed to watch \"{path}\": {error}", error = .0, path = .1.display())]
    Watch(notify::Error, PathBuf),
    #[error("file watcher error: {errors}", errors = .0.iter().format(", "))]
    Event(Vec<notify::Error>),
}

impl Watcher {
    pub fn new<S>(timeout: Duration, mut sender: S) -> WatcherResult<Self>
    where
        S: FnMut(WatcherMessage) + Send + 'static,
    {
        let event_handler = move |event| {
            if let Some(event) = filter_event(event) {
                (sender)(event);
            }
        };

        Ok(Self {
            sender: notify_debouncer_full::new_debouncer(timeout, None, event_handler)
                .map_err(WatcherError::Build)?,
        })
    }

    pub fn watch(
        &mut self,
        path: impl AsRef<Path>,
        recursive_mode: RecursiveMode,
    ) -> WatcherResult<()> {
        if fs::create_dir_all(&path).is_ok() {
            self.sender
                .watch(&path, recursive_mode)
                .map_err(|err| WatcherError::Watch(err, path.as_ref().to_path_buf()))
        } else {
            Ok(())
        }
    }
}

fn filter_event(event: DebounceEventResult) -> Option<WatcherMessage> {
    match event {
        Ok(mut events) => {
            events.retain(|event| {
                matches!(
                    event.event.kind,
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                )
            });

            (!events.is_empty()).then_some(Ok(events))
        }
        Err(err) => Some(Err(WatcherError::Event(err))),
    }
}
