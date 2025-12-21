use std::iter;

use rustc_hash::FxHashMap;
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, CompletionItemLabelDetails};
use wotw_seedgen_data::{
    assets::{
        AssetCacheValues, AssetFileAccess, ChangedAssets, DefaultFileAccess, LocData,
        PresetFileAccess, SnippetFileAccess, StateData, UberStateAlias, UberStateData,
        UberStateDataEntry,
    },
    parse::Source,
    UberIdentifier,
};
use wotw_seedgen_server_shared::ServerState;

pub type Cache = ServerState<DefaultFileAccess, CacheValues>;

pub struct CacheValues {
    pub uber_state_data: UberStateData,
    pub uber_identifier_numeric_completion: Vec<CompletionItem>,
    pub uber_identifier_numeric_member_completion: FxHashMap<i32, Vec<CompletionItem>>,
    pub uber_identifier_name_completion: Vec<CompletionItem>,
    pub uber_identifier_name_member_completion: FxHashMap<String, Vec<CompletionItem>>,
}

impl AssetCacheValues for CacheValues {
    fn new<F>(file_access: &F) -> Result<Self, String>
    where
        F: AssetFileAccess + SnippetFileAccess + PresetFileAccess,
    {
        let uber_state_data = uber_state_data(file_access)?;
        let uber_identifier_numeric_completion =
            uber_identifier_numeric_completion(&uber_state_data);
        let uber_identifier_numeric_member_completion =
            uber_identifier_numeric_member_completion(&uber_state_data);
        let uber_identifier_name_completion = uber_identifier_name_completion(&uber_state_data);
        let uber_identifier_name_member_completion =
            uber_identifier_name_member_completion(&uber_state_data);

        Ok(Self {
            uber_state_data,
            uber_identifier_numeric_completion,
            uber_identifier_numeric_member_completion,
            uber_identifier_name_completion,
            uber_identifier_name_member_completion,
        })
    }

    fn loc_data(&self) -> &LocData {
        unimplemented!()
    }

    fn state_data(&self) -> &StateData {
        unimplemented!()
    }

    fn uber_state_data(&self) -> &UberStateData {
        &self.uber_state_data
    }

    fn areas(&self) -> &Source {
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

    fn update<F>(&mut self, file_access: &F, changed: ChangedAssets) -> Result<(), String>
    where
        F: AssetFileAccess + SnippetFileAccess + PresetFileAccess,
    {
        if changed.loc_data || changed.state_data || changed.uber_state_dump {
            self.uber_state_data = uber_state_data(file_access)?;
            self.uber_identifier_numeric_completion =
                uber_identifier_numeric_completion(&self.uber_state_data);
            self.uber_identifier_numeric_member_completion =
                uber_identifier_numeric_member_completion(&self.uber_state_data);
            self.uber_identifier_name_completion =
                uber_identifier_name_completion(&self.uber_state_data);
            self.uber_identifier_name_member_completion =
                uber_identifier_name_member_completion(&self.uber_state_data);
        }

        Ok(())
    }
}

fn uber_state_data<F: AssetFileAccess>(file_access: &F) -> Result<UberStateData, String> {
    let loc_data = file_access.loc_data()?;
    let state_data = file_access.state_data()?;
    file_access.uber_state_data(&loc_data, &state_data)
}

fn uber_identifier_numeric_completion(uber_state_data: &UberStateData) -> Vec<CompletionItem> {
    uber_state_data
        .id_lookup
        .iter()
        .map(|(id, data)| uber_identifier_numeric_completion_item(*id, data))
        .collect()
}

fn uber_identifier_numeric_member_completion(
    uber_state_data: &UberStateData,
) -> FxHashMap<i32, Vec<CompletionItem>> {
    let mut group_map = FxHashMap::<i32, Vec<CompletionItem>>::default();

    for (id, data) in &uber_state_data.id_lookup {
        group_map.entry(id.group).or_default().push(CompletionItem {
            insert_text: Some(id.member.to_string()),
            filter_text: Some(id.member.to_string()),
            ..uber_identifier_numeric_completion_item(*id, data)
        });
    }

    group_map
}

fn uber_identifier_name_completion(uber_state_data: &UberStateData) -> Vec<CompletionItem> {
    uber_state_data
        .name_lookup
        .iter()
        .flat_map(|(group, members)| {
            members.iter().flat_map(move |(member, aliases)| {
                let ambiguous = aliases.len() > 1;

                aliases.iter().map(move |alias| {
                    uber_identifier_name_completion_item(group, member, alias, ambiguous)
                })
            })
        })
        .collect()
}

fn uber_identifier_name_member_completion(
    uber_state_data: &UberStateData,
) -> FxHashMap<String, Vec<CompletionItem>> {
    uber_state_data
        .name_lookup
        .iter()
        .map(|(group, members)| {
            (
                group.clone(),
                members
                    .iter()
                    .flat_map(|(member, aliases)| {
                        let ambiguous = aliases.len() > 1;

                        aliases.iter().map(move |alias| CompletionItem {
                            insert_text: Some(member.clone()), // TODO edit in numbers on ambiguous names?
                            filter_text: Some(member.clone()),
                            ..uber_identifier_name_completion_item(group, member, alias, ambiguous)
                        })
                    })
                    .collect(),
            )
        })
        .collect()
}

fn uber_identifier_numeric_completion_item(
    id: UberIdentifier,
    data: &UberStateDataEntry,
) -> CompletionItem {
    CompletionItem {
        label: id.to_string(),
        label_details: Some(CompletionItemLabelDetails {
            description: Some(data.preferred_name().clone()),
            ..Default::default()
        }),
        kind: Some(CompletionItemKind::VALUE),
        ..Default::default()
    }
}

fn uber_identifier_name_completion_item(
    group: &str,
    member: &str,
    alias: &UberStateAlias,
    ambiguous: bool,
) -> CompletionItem {
    CompletionItem {
        label: format!("{group}.{member}"),
        label_details: Some(CompletionItemLabelDetails {
            description: Some(alias.to_string()),
            detail: ambiguous.then(|| "(ambiguous name)".to_string()),
        }),
        kind: Some(CompletionItemKind::VALUE),
        ..Default::default()
    }
}
