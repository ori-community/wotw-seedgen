use std::{
    fs,
    iter::FilterMap,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    sync::mpsc,
    time::Duration,
};

use itertools::Itertools;
use notify_debouncer_full::{
    notify::{self, EventKind, INotifyWatcher, RecursiveMode},
    DebouncedEvent, Debouncer, NoCache,
};
use thiserror::Error;

pub struct Watcher {
    sender: WatcherSender,
    receiver: WatcherReceiver,
}

type DebouncedEventResult = Result<Vec<DebouncedEvent>, Vec<notify::Error>>;

type WatcherSender = Debouncer<INotifyWatcher, NoCache>;
type WatcherReceiver = FilterMap<
    mpsc::IntoIter<DebouncedEventResult>,
    fn(DebouncedEventResult) -> Option<Result<Vec<DebouncedEvent>, WatcherError>>,
>;

impl Watcher {
    pub fn new(timeout: Duration) -> Result<Self, WatcherError> {
        let (tx, rx) = mpsc::channel();

        Ok(Self {
            sender: notify_debouncer_full::new_debouncer(timeout, None, tx)
                .map_err(WatcherError::Build)?,
            receiver: rx.into_iter().filter_map(|res| match res {
                Ok(events) => events
                    .iter()
                    .any(|event| {
                        matches!(
                            event.event.kind,
                            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                        )
                    })
                    .then_some(Ok(events)),
                Err(err) => Some(Err(WatcherError::Event(err))),
            }),
        })
    }

    pub fn watch(
        &mut self,
        path: impl AsRef<Path>,
        recursive_mode: RecursiveMode,
    ) -> Result<(), WatcherError> {
        let _ = fs::create_dir_all(&path);

        self.sender
            .watch(&path, recursive_mode)
            .map_err(|err| WatcherError::Watch(err, path.as_ref().to_path_buf()))
    }
}

impl Deref for Watcher {
    type Target = WatcherSender;

    fn deref(&self) -> &Self::Target {
        &self.sender
    }
}

impl DerefMut for Watcher {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sender
    }
}

impl Iterator for Watcher {
    type Item = Result<Vec<DebouncedEvent>, WatcherError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.receiver.next()
    }
}

#[derive(Debug, Error)]
pub enum WatcherError {
    #[error("failed to configure file watcher: {error}", error = .0)]
    Build(notify::Error),
    #[error("failed to watch \"{path}\": {error}", error = .0, path = .1.display())]
    Watch(notify::Error, PathBuf),
    #[error("file watcher error: {errors}", errors = .0.iter().format(", "))]
    Event(Vec<notify::Error>),
}
