use std::{
    fs::File,
    marker::PhantomData,
    ops::{Deref, DerefMut, Range},
    path::PathBuf,
};

use derivative::Derivative;
use log::warn;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::{
    assets::{file_create, file_err},
    seed_language::compile::{GlobalCompilerData, FREE_MEMORY_START},
};

impl GlobalCompilerData<'_, '_> {
    pub fn read_boolean_id(&mut self, id: String, span: Range<usize>) -> usize {
        if let Some(lint_data) = &mut self.lint_data {
            lint_data.id_use.boolean.read(id.clone(), span);
        }

        self.id_resolver.ids.boolean.id(id)
    }

    pub fn write_boolean_id(&mut self, id: String) -> usize {
        if let Some(lint_data) = &mut self.lint_data {
            lint_data.id_use.boolean.write(id.clone());
        }

        self.id_resolver.ids.boolean.id(id)
    }

    pub fn read_integer_id(&mut self, id: String, span: Range<usize>) -> usize {
        if let Some(lint_data) = &mut self.lint_data {
            lint_data.id_use.integer.read(id.clone(), span);
        }

        self.id_resolver.ids.integer.id(id)
    }

    pub fn write_integer_id(&mut self, id: String) -> usize {
        if let Some(lint_data) = &mut self.lint_data {
            lint_data.id_use.integer.write(id.clone());
        }

        self.id_resolver.ids.integer.id(id)
    }

    pub fn read_float_id(&mut self, id: String, span: Range<usize>) -> usize {
        if let Some(lint_data) = &mut self.lint_data {
            lint_data.id_use.float.read(id.clone(), span);
        }

        self.id_resolver.ids.float.id(id)
    }

    pub fn write_float_id(&mut self, id: String) -> usize {
        if let Some(lint_data) = &mut self.lint_data {
            lint_data.id_use.float.write(id.clone());
        }

        self.id_resolver.ids.float.id(id)
    }

    pub fn read_string_id(&mut self, id: String, span: Range<usize>) -> usize {
        if let Some(lint_data) = &mut self.lint_data {
            lint_data.id_use.string.read(id.clone(), span);
        }

        self.id_resolver.ids.string.id(id)
    }

    pub fn write_string_id(&mut self, id: String) -> usize {
        if let Some(lint_data) = &mut self.lint_data {
            lint_data.id_use.string.write(id.clone());
        }

        self.id_resolver.ids.string.id(id)
    }

    pub fn read_message_id(&mut self, id: String, span: Range<usize>) -> usize {
        if let Some(lint_data) = &mut self.lint_data {
            lint_data.id_use.message.read(id.clone(), span);
        }

        self.id_resolver.ids.message.id(id)
    }

    pub fn write_message_id(&mut self, id: String) -> usize {
        if let Some(lint_data) = &mut self.lint_data {
            lint_data.id_use.message.write(id.clone());
        }

        self.id_resolver.ids.message.id(id)
    }

    pub fn read_box_trigger_id(&mut self, id: String, span: Range<usize>) -> usize {
        if let Some(lint_data) = &mut self.lint_data {
            lint_data.id_use.box_trigger.read(id.clone(), span);
        }

        self.id_resolver.ids.box_trigger.id(id)
    }

    pub fn write_box_trigger_id(&mut self, id: String) -> usize {
        if let Some(lint_data) = &mut self.lint_data {
            lint_data.id_use.box_trigger.write(id.clone());
        }

        self.id_resolver.ids.box_trigger.id(id)
    }

    pub fn read_warp_icon_id(&mut self, id: String, span: Range<usize>) -> usize {
        if let Some(lint_data) = &mut self.lint_data {
            lint_data.id_use.warp_icon.read(id.clone(), span);
        }

        self.id_resolver.ids.warp_icon.id(id)
    }

    pub fn write_warp_icon_id(&mut self, id: String) -> usize {
        if let Some(lint_data) = &mut self.lint_data {
            lint_data.id_use.warp_icon.write(id.clone());
        }

        self.id_resolver.ids.warp_icon.id(id)
    }
}

/// String -> usize resolver for various ids which can use a lockfile to persist choices between compilations.
///
/// If used, the lockfile will be written on drop.
#[derive(Debug)]
pub struct IdResolver {
    lockfile: Option<PathBuf>,
    ids: Ids,
}

impl IdResolver {
    /// Reads ids from the lockfile, or uses default fallbacks
    pub fn new(lockfile: Option<PathBuf>) -> Self {
        match lockfile {
            None => Self {
                lockfile: None,
                ids: Ids::default(),
            },
            Some(lockfile_path) => {
                let ids = File::open(&lockfile_path)
                    .ok()
                    .and_then(|lockfile| match serde_json::from_reader(&lockfile) {
                        Ok(ids) => Some(ids),
                        Err(err) => {
                            warn!(
                                "regenerating ids after failing to parse lockfile \"{lockfile_path}\": {err}",
                                lockfile_path = lockfile_path.display()
                            );
                            None
                        }
                    })
                    .unwrap_or_default();

                Self {
                    lockfile: Some(lockfile_path),
                    ids,
                }
            }
        }
    }
}

impl Deref for IdResolver {
    type Target = Ids;

    fn deref(&self) -> &Self::Target {
        &self.ids
    }
}

impl DerefMut for IdResolver {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ids
    }
}

impl Drop for IdResolver {
    fn drop(&mut self) {
        if let Some(lockfile_path) = &self.lockfile {
            match file_create(lockfile_path) {
                Ok(lockfile) => {
                    self.ids.purge_unused();

                    if let Err(err) = serde_json::to_writer(lockfile, &self.ids) {
                        warn!("{}", file_err("write id lockfile", lockfile_path, err));
                    }
                }
                Err(err) => {
                    warn!("{err}");
                }
            }
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Ids {
    #[serde(skip_serializing_if = "IdMap::is_empty", default)]
    boolean: IdMap<FREE_MEMORY_START>,
    #[serde(skip_serializing_if = "IdMap::is_empty", default)]
    integer: IdMap<FREE_MEMORY_START>,
    #[serde(skip_serializing_if = "IdMap::is_empty", default)]
    float: IdMap<FREE_MEMORY_START>,
    #[serde(skip_serializing_if = "IdMap::is_empty", default)]
    string: IdMap<FREE_MEMORY_START>,
    #[serde(skip_serializing_if = "IdMap::is_empty", default)]
    pub boolean_state: IdMap<0>,
    #[serde(skip_serializing_if = "IdMap::is_empty", default)]
    pub integer_state: IdMap<0>,
    #[serde(skip_serializing_if = "IdMap::is_empty", default)]
    pub float_state: IdMap<0>,
    #[serde(skip_serializing_if = "IdMap::is_empty", default)]
    message: IdMap<0>,
    #[serde(skip_serializing_if = "IdMap::is_empty", default)]
    box_trigger: IdMap<0>,
    // never empty
    pub wheel: IdMap<0, IdMapWheel>,
    #[serde(skip_serializing_if = "IdMap::is_empty", default)]
    warp_icon: IdMap<0>,
}

impl Ids {
    fn purge_unused(&mut self) {
        let Self {
            boolean,
            integer,
            float,
            string,
            boolean_state,
            integer_state,
            float_state,
            message,
            box_trigger,
            wheel,
            warp_icon,
        } = self;

        boolean.purge_unused();
        integer.purge_unused();
        float.purge_unused();
        string.purge_unused();
        boolean_state.purge_unused();
        integer_state.purge_unused();
        float_state.purge_unused();
        message.purge_unused();
        box_trigger.purge_unused();
        wheel.purge_unused();
        warp_icon.purge_unused();
    }
}

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Debug(bound = ""))]
pub struct IdMap<const OFFSET: usize, S = IdMapEmpty> {
    /// Ids which have become unused and may be reassigned
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    gaps: Vec<usize>,
    /// Assigned ids
    #[serde(skip_serializing_if = "FxHashMap::is_empty", default)]
    ids: FxHashMap<String, Id>,
    #[serde(skip)]
    special: PhantomData<S>,
}

impl<const OFFSET: usize, S> IdMap<OFFSET, S>
where
    S: IdMapDefault,
{
    pub fn id(&mut self, id: String) -> usize {
        match self.ids.get_mut(&id) {
            None => {
                let value = self.gaps.pop().unwrap_or_else(|| self.ids.len() + OFFSET);
                self.ids.insert(id, Id::new(value));
                value
            }
            Some(id) => {
                id.used = true;
                id.value
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    /// Purges ids which have not been confirmed used since restoring from the lockfile
    fn purge_unused(&mut self) {
        let mut max = 0;

        self.ids.retain(|_, id| {
            if S::never_purge(id.value) {
                return true;
            }

            if id.used {
                max = usize::max(max, id.value);
            } else {
                self.gaps.push(id.value);
            }

            id.used
        });

        self.gaps.retain(|id| *id < max);
    }
}

impl<const OFFSET: usize, S> Default for IdMap<OFFSET, S>
where
    S: IdMapDefault,
{
    fn default() -> Self {
        Self {
            gaps: vec![],
            ids: S::default(),
            special: PhantomData,
        }
    }
}

/// Allows specifying default values for [`IdMap`]
pub trait IdMapDefault {
    fn default() -> FxHashMap<String, Id>;

    fn never_purge(id: usize) -> bool;
}

/// [`IdMap`] without default values
pub struct IdMapEmpty;

impl IdMapDefault for IdMapEmpty {
    fn default() -> FxHashMap<String, Id> {
        FxHashMap::default()
    }

    fn never_purge(_id: usize) -> bool {
        false
    }
}

/// Default values for the wheel [`IdMap`], which always contains the `root` wheel
pub struct IdMapWheel;

impl IdMapDefault for IdMapWheel {
    fn default() -> FxHashMap<String, Id> {
        FxHashMap::from_iter([("root".to_string(), Id::new(0))])
    }

    fn never_purge(id: usize) -> bool {
        id == 0
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Id {
    value: usize,
    /// Whether the id is confirmed used. Will only be initialized to false when reading from the lockfile.
    #[serde(skip)]
    used: bool,
}

impl Id {
    pub fn new(value: usize) -> Self {
        Self { value, used: true }
    }
}
