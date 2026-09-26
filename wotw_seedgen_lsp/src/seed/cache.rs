use std::iter;

use rustc_hash::FxHashMap;
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, CompletionItemLabelDetails};
use wotw_seedgen_data::{
    assets::{
        AssetCacheValues, AssetFileAccess, ChangedAssets, DefaultFileAccess, LocData,
        PresetFileAccess, SnippetFileAccess, StateData, UberStateData, UberStateDataEntry,
        UberStateNameEntry,
    },
    parse::Source,
    UberIdentifier,
};
use wotw_seedgen_server_shared::ServerState;

pub type Cache = ServerState<DefaultFileAccess, CacheValues>;

pub struct CacheValues {
    pub loc_data: Result<LocData, String>,
    pub uber_state_data: Result<UberStateData, String>,
    pub uber_identifier_completion: UberIdentifierCompletion,
}

#[derive(Default)]
pub struct UberIdentifierCompletion {
    pub numeric: UberIdentifierNumericCompletion,
    pub name: UberIdentifierNameCompletion,
}

#[derive(Default)]
pub struct UberIdentifierNumericCompletion {
    pub groups: Vec<CompletionItem>,
    pub members: FxHashMap<i32, Vec<CompletionItem>>,
}

#[derive(Default)]
pub struct UberIdentifierNameCompletion {
    pub groups: Vec<CompletionItem>,
    pub members: FxHashMap<String, UberIdentifierNameMemberCompletion>,
}

#[derive(Default)]
pub struct UberIdentifierNameMemberCompletion {
    pub members: Vec<CompletionItem>,
    pub pickups: FxHashMap<String, Vec<CompletionItem>>,
}

impl AssetCacheValues for CacheValues {
    fn new<F>(file_access: &F) -> Self
    where
        F: AssetFileAccess + SnippetFileAccess + PresetFileAccess,
    {
        let loc_data = file_access.loc_data();
        let uber_state_data =
            uber_state_data(loc_data.as_ref().map_err(String::clone), file_access);
        let uber_identifier_completion = UberIdentifierCompletion::new(&uber_state_data);

        Self {
            loc_data,
            uber_state_data,
            uber_identifier_completion,
        }
    }

    fn loc_data(&self) -> Result<&LocData, String> {
        self.loc_data.as_ref().map_err(String::clone)
    }

    fn state_data(&self) -> Result<&StateData, String> {
        unimplemented!()
    }

    fn uber_state_data(&self) -> Result<&UberStateData, String> {
        self.uber_state_data.as_ref().map_err(String::clone)
    }

    fn paths(&self) -> Result<&Source, String> {
        unimplemented!()
    }

    fn snippet(&self, _identifier: &str) -> Result<&Source, String> {
        unimplemented!()
    }

    fn allow_read_file(&self) -> bool {
        unimplemented!()
    }

    fn available_snippets(&self) -> impl Iterator<Item = &String> {
        #[allow(unreachable_code)]
        iter::once(unimplemented!())
    }

    fn update<F>(&mut self, file_access: &F, changed: ChangedAssets)
    where
        F: AssetFileAccess + SnippetFileAccess + PresetFileAccess,
    {
        if changed.loc_data {
            self.loc_data = file_access.loc_data();
        }

        if changed.loc_data || changed.state_data || changed.uber_state_dump {
            self.uber_state_data = uber_state_data(self.loc_data(), file_access);
            self.uber_identifier_completion = UberIdentifierCompletion::new(&self.uber_state_data);
        }
    }
}

fn uber_state_data<F: AssetFileAccess>(
    loc_data: Result<&LocData, String>,
    file_access: &F,
) -> Result<UberStateData, String> {
    file_access.uber_state_data(loc_data?, &file_access.state_data()?)
}

impl UberIdentifierCompletion {
    fn new(uber_state_data: &Result<UberStateData, String>) -> Self {
        let Ok(uber_state_data) = uber_state_data else {
            return Self::default();
        };

        Self {
            numeric: UberIdentifierNumericCompletion::new(uber_state_data),
            name: UberIdentifierNameCompletion::new(uber_state_data),
        }
    }
}

impl UberIdentifierNumericCompletion {
    fn new(uber_state_data: &UberStateData) -> Self {
        let mut numeric = Self::default();

        for (id, data) in &uber_state_data.id_lookup {
            numeric.groups.push(numeric_completion_item(*id, data));

            numeric
                .members
                .entry(id.group)
                .or_default()
                .push(CompletionItem {
                    insert_text: Some(id.member.to_string()),
                    filter_text: Some(id.member.to_string()),
                    ..numeric_completion_item(*id, data)
                });
        }

        numeric
    }
}

impl UberIdentifierNameCompletion {
    fn new(uber_state_data: &UberStateData) -> Self {
        let mut name = Self::default();

        name.groups.reserve(uber_state_data.name_lookup.len());
        name.members.reserve(uber_state_data.name_lookup.len());

        for (group, members) in &uber_state_data.name_lookup {
            let mut member_completions = UberIdentifierNameMemberCompletion::default();

            for (member, entry) in members {
                match entry {
                    UberStateNameEntry::Vanilla(uber_identifiers) => {
                        let ambiguous = uber_identifiers.len() > 1;

                        name.groups.reserve(uber_identifiers.len());
                        member_completions.members.reserve(uber_identifiers.len());

                        for uber_identifier in uber_identifiers {
                            push_name_member_completion(
                                CompletionItem {
                                    label: format!("{group}.{member}"),
                                    label_details: Some(CompletionItemLabelDetails {
                                        description: Some(uber_identifier.to_string()),
                                        detail: ambiguous.then(|| "(ambiguous name)".to_string()),
                                    }),
                                    kind: Some(CompletionItemKind::VALUE),
                                    ..Default::default()
                                },
                                member,
                                &mut name,
                                &mut member_completions,
                            );
                        }
                    }
                    UberStateNameEntry::Rando(rando_group) => {
                        if let Some(alias) = &rando_group.root_member {
                            push_name_member_completion(
                                simple_completion_item(
                                    format!("{group}.{member}"),
                                    alias.to_string(),
                                ),
                                member,
                                &mut name,
                                &mut member_completions,
                            );
                        }

                        name.groups.reserve(rando_group.members.len());
                        member_completions
                            .members
                            .reserve(rando_group.members.len());

                        let mut pickup_completions = Vec::with_capacity(rando_group.members.len());

                        for (pickup, alias) in &rando_group.members {
                            push_name_pickup_completion(
                                simple_completion_item(
                                    format!("{group}.{member}.{pickup}"),
                                    alias.to_string(),
                                ),
                                member,
                                pickup,
                                &mut name,
                                &mut member_completions,
                                &mut pickup_completions,
                            );
                        }

                        member_completions
                            .pickups
                            .insert(member.clone(), pickup_completions);
                    }
                }
            }

            name.members.insert(group.clone(), member_completions);
        }

        name
    }
}

fn numeric_completion_item(id: UberIdentifier, data: &UberStateDataEntry) -> CompletionItem {
    simple_completion_item(id.to_string(), data.preferred_name().clone())
}

fn simple_completion_item(label: String, description: String) -> CompletionItem {
    CompletionItem {
        label,
        label_details: Some(CompletionItemLabelDetails {
            description: Some(description),
            ..Default::default()
        }),
        kind: Some(CompletionItemKind::VALUE),
        ..Default::default()
    }
}

fn push_name_member_completion(
    full_completion: CompletionItem,
    member: &str,
    name: &mut UberIdentifierNameCompletion,
    member_completions: &mut UberIdentifierNameMemberCompletion,
) {
    member_completions.members.push(CompletionItem {
        insert_text: Some(member.to_string()), // TODO edit in numbers on ambiguous names?
        filter_text: Some(member.to_string()),
        ..full_completion.clone()
    });

    name.groups.push(full_completion);
}

fn push_name_pickup_completion(
    full_completion: CompletionItem,
    member: &String,
    pickup: &String,
    name: &mut UberIdentifierNameCompletion,
    member_completions: &mut UberIdentifierNameMemberCompletion,
    pickup_completions: &mut Vec<CompletionItem>,
) {
    pickup_completions.push(CompletionItem {
        insert_text: Some(pickup.clone()),
        filter_text: Some(pickup.clone()),
        ..full_completion.clone()
    });

    member_completions.members.push(CompletionItem {
        insert_text: Some(format!("{member}.{pickup}")),
        filter_text: Some(format!("{member}.{pickup}")),
        ..full_completion.clone()
    });

    name.groups.push(full_completion);
}
