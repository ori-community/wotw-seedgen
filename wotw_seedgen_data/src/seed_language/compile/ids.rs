use std::{
    fs::{self, File},
    marker::PhantomData,
    ops::{ControlFlow, Deref, DerefMut, Range},
    path::{Path, PathBuf},
};

use derivative::Derivative;
use log::warn;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use wotw_seedgen_log_capture::{LogCapture, NO_LOG_CAPTURE};
use wotw_seedgen_parse::{Error, Span};

use crate::{
    assets::{file_create, file_err},
    seed_language::compile::{
        GlobalCompilerData, FREE_BOOLEANS_START, FREE_FLOATS_START, FREE_INTEGERS_START,
        FREE_STRINGS_START,
    },
};

impl GlobalCompilerData<'_, '_, '_> {
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

    pub fn read_position_trigger_id(&mut self, id: String, span: Range<usize>) -> usize {
        if let Some(lint_data) = &mut self.lint_data {
            lint_data.id_use.position_trigger.read(id.clone(), span);
        }

        self.id_resolver.ids.position_trigger.id(id)
    }

    pub fn write_position_trigger_id(&mut self, id: String) -> usize {
        if let Some(lint_data) = &mut self.lint_data {
            lint_data.id_use.position_trigger.write(id.clone());
        }

        self.id_resolver.ids.position_trigger.id(id)
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
pub struct IdResolver<'log> {
    lockfile: Option<PathBuf>,
    ids: Ids,
    log_capture: &'log LogCapture,
}

impl<'log> IdResolver<'log> {
    pub fn new() -> Self {
        Self {
            lockfile: None,
            ids: Ids::default(),
            log_capture: &NO_LOG_CAPTURE,
        }
    }

    /// Reads ids from the lockfile, or uses default fallbacks
    pub fn from_lockfile(path: PathBuf, log_capture: &'log LogCapture) -> Self {
        let ids = Ids::from_lockfile(&path, log_capture).unwrap_or_default();

        Self {
            lockfile: Some(path),
            ids,
            log_capture,
        }
    }
}

impl Deref for IdResolver<'_> {
    type Target = Ids;

    fn deref(&self) -> &Self::Target {
        &self.ids
    }
}

impl DerefMut for IdResolver<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ids
    }
}

impl Drop for IdResolver<'_> {
    fn drop(&mut self) {
        if let Some(lockfile_path) = &self.lockfile {
            if self.ids == Ids::default() {
                let _ = fs::remove_file(lockfile_path);
            } else {
                match file_create(lockfile_path) {
                    Ok(lockfile) => {
                        self.ids.purge_unused();

                        if let Err(err) = serde_json::to_writer(lockfile, &self.ids) {
                            warn!(logger: self.log_capture, "{}", file_err("write id lockfile", lockfile_path, err));
                        }
                    }
                    Err(err) => {
                        warn!(logger: self.log_capture, "{err}");
                    }
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ids {
    #[serde(skip_serializing_if = "IdMap::is_empty", default)]
    boolean: IdMap<FREE_BOOLEANS_START>,
    #[serde(skip_serializing_if = "IdMap::is_empty", default)]
    integer: IdMap<FREE_INTEGERS_START>,
    #[serde(skip_serializing_if = "IdMap::is_empty", default)]
    float: IdMap<FREE_FLOATS_START>,
    #[serde(skip_serializing_if = "IdMap::is_empty", default)]
    string: IdMap<FREE_STRINGS_START>,
    #[serde(skip_serializing_if = "IdMap::is_empty", default)]
    pub boolean_state: IdMap<0, 100>,
    #[serde(skip_serializing_if = "IdMap::is_empty", default)]
    pub integer_state: IdMap<0, 100>,
    #[serde(skip_serializing_if = "IdMap::is_empty", default)]
    pub float_state: IdMap<0, 25>,
    #[serde(skip_serializing_if = "IdMap::is_empty", default)]
    message: IdMap<0>,
    #[serde(skip_serializing_if = "IdMap::is_empty", default)]
    position_trigger: IdMap<0>,
    // never empty
    // 9000 is the first rando-reserved wheel (aside from root)
    pub wheel: IdMap<0, 9000, IdMapWheel>,
    // warp icon 0 is reserved for the spawn warp
    #[serde(skip_serializing_if = "IdMap::is_empty", default)]
    warp_icon: IdMap<1>,
}

impl Ids {
    fn from_lockfile(path: &Path, log_capture: &LogCapture) -> Option<Self> {
        let lockfile = File::open(path).ok()?;

        let mut previous: Ids = match serde_json::from_reader(&lockfile) {
            Ok(previous) => previous,
            Err(err) => {
                warn!(
                    logger: log_capture,
                    "regenerating ids after failing to parse lockfile \"{path}\": {err}",
                    path = path.display(),
                );

                return None;
            }
        };

        previous.validate(log_capture);

        Some(previous)
    }

    fn validate(&mut self, log_capture: &LogCapture) {
        let Self {
            boolean,
            integer,
            float,
            string,
            boolean_state,
            integer_state,
            float_state,
            message,
            position_trigger,
            wheel,
            warp_icon,
        } = self;

        boolean.validate_offset(log_capture, "boolean");
        integer.validate_offset(log_capture, "integer");
        float.validate_offset(log_capture, "float");
        string.validate_offset(log_capture, "string");
        boolean_state.validate_limit(log_capture, "boolean state");
        integer_state.validate_limit(log_capture, "integer state");
        float_state.validate_limit(log_capture, "float state");
        message.validate_offset(log_capture, "message");
        position_trigger.validate_offset(log_capture, "position trigger");
        wheel.validate_limit(log_capture, "wheel");
        warp_icon.validate_offset(log_capture, "warp icon");
    }

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
            position_trigger,
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
        position_trigger.purge_unused();
        wheel.purge_unused();
        warp_icon.purge_unused();
    }
}

#[derive(Derivative, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derivative(Debug(bound = ""), PartialEq(bound = ""), Eq(bound = ""))]
pub struct IdMap<const OFFSET: usize, const LIMIT: usize = { usize::MAX }, S = IdMapEmpty> {
    /// Ids which have become unused and may be reassigned
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    gaps: Vec<usize>,
    /// Assigned ids
    #[serde(skip_serializing_if = "FxHashMap::is_empty", default)]
    ids: FxHashMap<String, Id>,
    #[serde(skip)]
    special: PhantomData<S>,
}

impl<const OFFSET: usize, S> IdMap<OFFSET, { usize::MAX }, S> {
    pub fn id(&mut self, id: String) -> usize {
        self.id_impl(id, |next| ControlFlow::Continue(next + OFFSET))
    }

    fn validate_offset(&mut self, log_capture: &LogCapture, id_kind: &str) {
        self.gaps.retain(|gap| *gap >= OFFSET);
        self.ids.retain(|id, value| {
            let retain = value.value >= OFFSET;

            if !retain {
                warn!(logger: log_capture, "reassigning {id_kind} id \"{id}\" since the previous value has been reserved");
            }

            retain
        });
    }
}

impl<const LIMIT: usize, SD> IdMap<0, LIMIT, SD> {
    pub fn try_id<S: Span>(&mut self, id: String, errors: &mut Vec<Error>, span: S) -> usize {
        self.id_impl(id, move |next|
            if next >= LIMIT {
                errors.push(Error::error(
                    format!(
                        "Only {LIMIT} instances of this type are available (What on earth are you doing?)"
                    ),
                    span.span(),
                ));
                ControlFlow::Break(LIMIT - 1)
            } else {
                ControlFlow::Continue(next)
            })
    }

    fn validate_limit(&mut self, log_capture: &LogCapture, id_kind: &str) {
        self.gaps.retain(|gap| *gap < LIMIT);
        self.ids.retain(|id, value| {
            let retain = value.value < LIMIT;

            if !retain {
                warn!(logger: log_capture, "reassigning {id_kind} id \"{id}\" since the previous value falls out of the available limit");
            }

            retain
        });
    }
}

impl<const OFFSET: usize, const LIMIT: usize, S> IdMap<OFFSET, LIMIT, S> {
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    fn id_impl<F>(&mut self, id: String, new: F) -> usize
    where
        F: FnOnce(usize) -> ControlFlow<usize, usize>,
    {
        match self.ids.get_mut(&id) {
            None => {
                let value = match self.gaps.pop() {
                    None => match new(self.ids.len()) {
                        ControlFlow::Continue(next) => next,
                        ControlFlow::Break(dummy) => return dummy,
                    },
                    Some(gap) => gap,
                };

                self.ids.insert(id, Id::new(value));

                value
            }
            Some(id) => {
                id.used = true;
                id.value
            }
        }
    }

    /// Purges ids which have not been confirmed used since restoring from the lockfile
    fn purge_unused(&mut self)
    where
        S: IdMapDefault,
    {
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

impl<const OFFSET: usize, const LIMIT: usize, S> Default for IdMap<OFFSET, LIMIT, S>
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

impl PartialEq for Id {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for Id {}
