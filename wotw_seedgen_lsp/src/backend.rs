use dashmap::{
    mapref::one::{Ref, RefMut},
    DashMap,
};
use std::{fmt::Display, path::PathBuf};
use tower_lsp::{
    jsonrpc::Result,
    lsp_types::{
        CompletionParams, DidChangeTextDocumentParams, DidOpenTextDocumentParams,
        DidSaveTextDocumentParams, GotoDefinitionParams, HoverParams, MessageType,
        SemanticTokensParams, SignatureHelpParams, TextDocumentContentChangeEvent,
        TextDocumentItem, TextDocumentPositionParams, Url,
    },
    Client,
};
use wotw_seedgen_data::parse::Error;

use crate::{convert, error};

pub struct Backend<C> {
    pub client: Client,
    pub cache: C,
    text_documents: DashMap<Url, String>,
}

pub type TextDocument<'a> = Ref<'a, Url, String>;
pub type TextDocumentMut<'a> = RefMut<'a, Url, String>;
pub type TextDocumentPosition<'a> = (TextDocument<'a>, usize);

impl<C> Backend<C> {
    pub fn new(client: Client, cache: C) -> Self {
        Self {
            client,
            cache,
            text_documents: Default::default(),
        }
    }

    pub fn insert_text_document(&self, uri: Url, text: String) {
        self.text_documents.insert(uri, text);
    }

    pub fn get_text_document<'s>(&'s self, url: &Url) -> Result<TextDocument<'s>> {
        self.text_documents
            .get(url)
            .ok_or(error::unknown_text_document(url))
    }

    pub fn get_text_document_mut<'s>(&'s self, url: &Url) -> Result<TextDocumentMut<'s>> {
        self.text_documents
            .get_mut(url)
            .ok_or(error::unknown_text_document(url))
    }

    pub fn get_text_document_position(
        &self,
        text_document_position: TextDocumentPositionParams,
    ) -> Result<TextDocumentPosition<'_>> {
        let TextDocumentPositionParams {
            text_document,
            position,
        } = text_document_position;

        let source = self.get_text_document(&text_document.uri)?;
        let position = convert::position_from_lsp(position, source.value())?;

        Ok((source, position))
    }

    pub async fn update_text_document(
        &self,
        uri: &Url,
        changes: Vec<TextDocumentContentChangeEvent>,
    ) {
        let mut text_document = match self.get_text_document_mut(uri) {
            Ok(text_document) => text_document,
            Err(err) => {
                self.warn(err).await;
                return;
            }
        };

        for content_change in changes {
            match content_change.range {
                None => *text_document.value_mut() = content_change.text,
                Some(range) => {
                    let Some(range) = self
                        .consume_result(convert::range_from_lsp(range, &text_document))
                        .await
                    else {
                        continue;
                    };

                    text_document.replace_range(range, &content_change.text)
                }
            }
        }
    }

    pub fn clear_text_documents(&self) {
        self.text_documents.clear();
    }

    pub async fn error<M: Display>(&self, message: M) {
        self.client.log_message(MessageType::ERROR, message).await;
    }

    pub async fn warn<M: Display>(&self, message: M) {
        self.client.log_message(MessageType::WARNING, message).await;
    }

    pub async fn log<M: Display>(&self, message: M) {
        self.client.log_message(MessageType::INFO, message).await;
    }

    pub async fn consume_result<T, E: Display>(
        &self,
        result: std::result::Result<T, E>,
    ) -> Option<T> {
        match result {
            Ok(t) => Some(t),
            Err(err) => {
                self.error(err).await;
                None
            }
        }
    }

    pub async fn initialize_base(&self) {
        self.log("received initialize 👋").await;
    }

    pub async fn shutdown_base(&self) {
        self.log("received shutdown 😵").await;

        self.clear_text_documents();
    }

    pub async fn did_open_base(&self, params: DidOpenTextDocumentParams) -> Url {
        let TextDocumentItem { uri, text, .. } = params.text_document;

        self.log(format!("received textDocument/didOpen for \"{uri}\""))
            .await;

        self.insert_text_document(uri.clone(), text);

        uri
    }

    pub async fn did_change_base(&self, params: DidChangeTextDocumentParams) -> Url {
        let uri = params.text_document.uri;

        self.log(format!("received textDocument/didChange for \"{uri}\""))
            .await;

        self.update_text_document(&uri, params.content_changes)
            .await;

        uri
    }

    pub async fn did_save_base(&self, params: DidSaveTextDocumentParams) -> Url {
        let uri = params.text_document.uri;

        self.log(format!("received textDocument/didSave for \"{uri}\""))
            .await;

        uri
    }

    pub async fn goto_definition_base(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<TextDocumentPosition<'_>> {
        self.log("received textDocument/definition").await;

        self.get_text_document_position(params.text_document_position_params)
    }

    pub async fn hover_base(&self, params: HoverParams) -> Result<TextDocumentPosition<'_>> {
        self.log("received textDocument/hover").await;

        self.get_text_document_position(params.text_document_position_params)
    }

    pub async fn semantic_tokens_full_base(
        &self,
        params: SemanticTokensParams,
    ) -> Result<TextDocument<'_>> {
        let uri = params.text_document.uri;

        self.log(format!(
            "received textDocument/semanticTokens/full for \"{uri}\""
        ))
        .await;

        self.get_text_document(&uri)
    }

    pub async fn completion_base(
        &self,
        params: CompletionParams,
    ) -> Result<TextDocumentPosition<'_>> {
        self.log("received textDocument/completion").await;

        self.get_text_document_position(params.text_document_position)
    }

    pub async fn signature_help_base(
        &self,
        params: SignatureHelpParams,
    ) -> Result<TextDocumentPosition<'_>> {
        self.log("received textDocument/signatureHelp").await;

        self.get_text_document_position(params.text_document_position_params)
    }

    pub async fn update_diagnostics_with<F>(&self, uri: Url, f: F)
    where
        F: AsyncFnOnce(PathBuf) -> Option<(String, Vec<Error>)>,
    {
        let Some(path) = self.consume_result(convert::path_from_lsp(&uri)).await else {
            return;
        };

        let Some((source, errors)) = f(path).await else {
            return;
        };

        let errors = errors
            .into_iter()
            .map(|error| convert::error_to_lsp(error, &source))
            .collect();

        self.client.publish_diagnostics(uri, errors, None).await;
    }

    pub async fn execute_command_base(&self, command: &str) {
        self.log(format!(
            "received workspace/execute_command for \"{command}\""
        ))
        .await;
    }
}
