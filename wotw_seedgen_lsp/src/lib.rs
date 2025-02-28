mod convert;
mod folder_access;
mod semantic_tokens;

use std::fmt::Display;

use dashmap::{mapref::one::Ref, DashMap};
use folder_access::{url_to_path, FolderAccess};
use semantic_tokens::{semantic_tokens, semantic_tokens_legend};
use tower_lsp::{
    jsonrpc::{Error, Result},
    lsp_types::{
        Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams, DidOpenTextDocumentParams,
        DidSaveTextDocumentParams, InitializeParams, InitializeResult, MessageType, SemanticTokens,
        SemanticTokensFullOptions, SemanticTokensOptions, SemanticTokensParams,
        SemanticTokensResult, SemanticTokensServerCapabilities, ServerCapabilities,
        TextDocumentSyncCapability, TextDocumentSyncKind, Url,
    },
    Client, LanguageServer, LspService, Server,
};
use wotw_seedgen_seed_language::{
    ast::{self, Snippet},
    compile::Compiler,
};
use wotw_seedgen_static_assets::UBER_STATE_DATA;

struct Backend {
    client: Client,
    text_documents: DashMap<Url, String>,
}

impl Backend {
    fn new(client: Client) -> Self {
        Self {
            client,
            text_documents: Default::default(),
        }
    }

    fn get_text_document<'s>(&'s self, url: &Url) -> Result<Ref<'s, Url, String>> {
        self.text_documents
            .get(url)
            .ok_or(Error::invalid_params(format!(
                "unknown text document \"{url}\""
            )))
    }

    async fn consume_result<T, E: Display>(&self, result: std::result::Result<T, E>) -> Option<T> {
        match result {
            Ok(t) => Some(t),
            Err(err) => {
                self.client.log_message(MessageType::WARNING, err).await;
                None
            }
        }
    }

    async fn update_diagnostics(&self, url: Url) {
        let Some(path) = self.consume_result(url_to_path(&url)).await else {
            return;
        };
        let folder_access = FolderAccess::new(&path);
        let Some(identifier) = self
            .consume_result(
                path.file_stem()
                    .ok_or_else(|| format!("invalid url \"{url}\": not a file"))
                    .and_then(|identifier| {
                        identifier
                            .to_str()
                            .ok_or_else(|| format!("invalid url \"{url}\": invalid unicode"))
                    }),
            )
            .await
        else {
            return;
        };

        let errors = {
            let mut compiler = Compiler::new(
                &mut rand::thread_rng(),
                &folder_access,
                &*UBER_STATE_DATA,
                Default::default(),
                false,
            );
            // TODO currently we can only give diagnostics for saved files because we're not using the editors in-memory changes
            // Need to do changes in the language create to improve that
            compiler.compile_snippet(identifier).unwrap(); // TODO have to gracefully exit here, path might be outdated
            let (source, errors) = compiler
                .finish()
                .errors
                .into_iter()
                .find(|(source, _)| source.id[..source.id.len() - 6].ends_with(identifier))
                .unwrap();

            errors
                .into_iter()
                .map(|error| Diagnostic {
                    range: convert::range_to_lsp(error.span, &source.content),
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: error.kind.to_string(),
                    ..Default::default()
                })
                .collect()
        };

        self.client.publish_diagnostics(url, errors, None).await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: semantic_tokens_legend(),
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            ..Default::default()
                        },
                    ),
                ),
                ..Default::default()
            },
            server_info: None,
        })
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.text_documents
            .insert(params.text_document.uri.clone(), params.text_document.text);
        self.update_diagnostics(params.text_document.uri).await;
    }
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(mut text_document) = self.text_documents.get_mut(&params.text_document.uri) {
            for content_change in params.content_changes {
                match content_change
                    .range
                    .and_then(|range| convert::range_from_lsp(range, &text_document))
                {
                    None => *text_document.value_mut() = content_change.text,
                    Some(range) => text_document.replace_range(range, &content_change.text),
                }
            }
        }
    }
    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.update_diagnostics(params.text_document.uri).await;
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let source = self.get_text_document(&params.text_document.uri)?;

        let ast = ast::parse::<Snippet>(source.value());
        let data = semantic_tokens(source.value(), ast.parsed);

        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            data,
            ..Default::default()
        })))
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

pub fn start() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let stdin = tokio::io::stdin();
            let stdout = tokio::io::stdout();

            let (service, socket) = LspService::build(|client| Backend::new(client)).finish();

            Server::new(stdin, stdout, socket).serve(service).await;
        });
}
