use std::{collections::hash_map::Entry, mem, ops::Range};

use rustc_hash::FxHashMap;
use wotw_seedgen_parse::{Error, Source};

#[derive(Debug, Default)]
pub struct IdUse {
    pub boolean: IdUseMap,
    pub integer: IdUseMap,
    pub float: IdUseMap,
    pub string: IdUseMap,
    pub message: IdUseMap,
    pub box_trigger: IdUseMap,
    pub warp_icon: IdUseMap,
}

impl IdUse {
    pub fn finish_snippet(&mut self, identifier: &str) {
        let Self {
            boolean,
            integer,
            float,
            string,
            message,
            box_trigger,
            warp_icon,
        } = self;

        boolean.finish_snippet(identifier);
        integer.finish_snippet(identifier);
        float.finish_snippet(identifier);
        string.finish_snippet(identifier);
        message.finish_snippet(identifier);
        box_trigger.finish_snippet(identifier);
        warp_icon.finish_snippet(identifier);
    }

    pub fn finish(self, errors: &mut FxHashMap<String, (Source, Vec<Error>)>) {
        let Self {
            boolean,
            integer,
            float,
            string,
            message,
            box_trigger,
            warp_icon,
        } = self;

        boolean.finish(errors);
        integer.finish(errors);
        float.finish(errors);
        string.finish(errors);
        message.finish(errors);
        box_trigger.finish(errors);
        warp_icon.finish(errors);
    }
}

#[derive(Debug, Default)]
pub struct IdUseMap {
    inner: FxHashMap<String, IdUseStatus>,
}

impl IdUseMap {
    pub fn read(&mut self, id: String, span: Range<usize>) {
        match self.inner.entry(id) {
            Entry::Occupied(occupied) => match occupied.into_mut() {
                IdUseStatus::NeverWritten { read_spans } => read_spans.current.push(span),
                IdUseStatus::Written => {}
            },
            Entry::Vacant(vacant) => {
                vacant.insert(IdUseStatus::NeverWritten {
                    read_spans: SnippetIdReadSpans::new(span),
                });
            }
        }
    }

    pub fn write(&mut self, id: String) {
        self.inner.insert(id, IdUseStatus::Written);
    }

    fn finish_snippet(&mut self, identifier: &str) {
        for status in self.inner.values_mut() {
            match status {
                IdUseStatus::NeverWritten { read_spans } => {
                    read_spans.finish_snippet(identifier.to_string());
                }
                IdUseStatus::Written => {}
            }
        }
    }

    pub fn finish(self, errors: &mut FxHashMap<String, (Source, Vec<Error>)>) {
        for status in self.inner.into_values() {
            match status {
                IdUseStatus::NeverWritten { read_spans } => {
                    debug_assert!(read_spans.current.is_empty());

                    for (snippet, spans) in read_spans.finished {
                        errors.get_mut(&snippet).unwrap().1.extend(
                            spans.into_iter().map(|span| {
                                Error::warning("id never written to".to_string(), span)
                            }),
                        );
                    }
                }
                IdUseStatus::Written => {}
            }
        }
    }
}

#[derive(Debug)]
enum IdUseStatus {
    NeverWritten { read_spans: SnippetIdReadSpans },
    Written,
}

#[derive(Debug)]
struct SnippetIdReadSpans {
    current: Vec<Range<usize>>,
    finished: Vec<(String, Vec<Range<usize>>)>,
}

impl SnippetIdReadSpans {
    fn new(span: Range<usize>) -> Self {
        Self {
            current: vec![span],
            finished: Vec::new(),
        }
    }

    fn finish_snippet(&mut self, identifier: String) {
        self.finished
            .push((identifier, mem::take(&mut self.current)));
    }
}
