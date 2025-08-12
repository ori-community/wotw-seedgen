mod completion;
mod convert;
mod error;
mod folder_access;
mod semantic_tokens;

use std::fmt::Display;

use completion::Completion;
use dashmap::{
    mapref::one::{Ref, RefMut},
    DashMap,
};
use folder_access::{url_to_path, FolderAccess};
use semantic_tokens::{semantic_tokens, semantic_tokens_legend};
use tower_lsp::{
    jsonrpc::Result,
    lsp_types::{
        CompletionOptions, CompletionParams, CompletionResponse, Diagnostic, DiagnosticSeverity,
        DidChangeTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
        InitializeParams, InitializeResult, MessageType, ParameterInformation, ParameterLabel,
        SemanticTokens, SemanticTokensFullOptions, SemanticTokensOptions, SemanticTokensParams,
        SemanticTokensResult, SemanticTokensServerCapabilities, ServerCapabilities, ServerInfo,
        SignatureHelp, SignatureHelpOptions, SignatureHelpParams, SignatureInformation,
        TextDocumentItem, TextDocumentPositionParams, TextDocumentSyncCapability,
        TextDocumentSyncKind, Url,
    },
    Client, LanguageServer, LspService, Server,
};
use wotw_seedgen_seed_language::{
    ast,
    compile::{Compiler, FunctionIdentifier},
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
            .ok_or(error::unknown_text_document(url))
    }

    fn get_text_document_mut<'s>(&'s self, url: &Url) -> Result<RefMut<'s, Url, String>> {
        self.text_documents
            .get_mut(url)
            .ok_or(error::unknown_text_document(url))
    }

    fn get_text_document_position<'s>(
        &'s self,
        text_document_position: TextDocumentPositionParams,
    ) -> Result<(Ref<'s, Url, String>, usize)> {
        let TextDocumentPositionParams {
            text_document,
            position,
        } = text_document_position;
        let source = self.get_text_document(&text_document.uri)?;
        let position = convert::position_from_lsp(position, source.value())?;

        Ok((source, position))
    }

    async fn error<M: Display>(&self, message: M) {
        self.client.log_message(MessageType::ERROR, message).await;
    }

    async fn warn<M: Display>(&self, message: M) {
        self.client.log_message(MessageType::WARNING, message).await;
    }

    async fn log<M: Display>(&self, message: M) {
        self.client.log_message(MessageType::INFO, message).await;
    }

    async fn consume_result<T, E: Display>(&self, result: std::result::Result<T, E>) -> Option<T> {
        match result {
            Ok(t) => Some(t),
            Err(err) => {
                self.error(err).await;
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
                &UBER_STATE_DATA,
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
        self.log("received initialize 👋").await;

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(
                        ('0'..='9')
                            .chain(['|', '.', ':', '!', '#'])
                            .map(|c| c.to_string())
                            .collect(),
                    ),
                    ..Default::default()
                }),
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec!['('.to_string()]),
                    ..Default::default()
                }),
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
            server_info: Some(ServerInfo {
                name: "wotw_seedgen_lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn shutdown(&self) -> Result<()> {
        self.log("received shutdown 😵").await;

        self.text_documents.clear();

        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let TextDocumentItem { uri, text, .. } = params.text_document;

        self.log(format!("received textDocument/didOpen for \"{uri}\""))
            .await;

        self.text_documents.insert(uri.clone(), text);
        self.update_diagnostics(uri).await;
    }
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;

        self.log(format!("received textDocument/didChange for \"{uri}\""))
            .await;

        let mut text_document = match self.get_text_document_mut(&uri) {
            Ok(text_document) => text_document,
            Err(err) => {
                self.warn(err).await;
                return;
            }
        };

        for content_change in params.content_changes {
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
    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;

        self.log(format!("received textDocument/didSave for \"{uri}\""))
            .await;

        self.update_diagnostics(uri).await;
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri;

        self.log(format!(
            "received textDocument/semanticTokens/full for \"{uri}\""
        ))
        .await;

        let source = self.get_text_document(&uri)?;

        let ast = ast::parse::<ast::Snippet>(source.value());
        let data = semantic_tokens(source.value(), ast.parsed);

        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            data,
            ..Default::default()
        })))
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        self.log("received textDocument/completion").await;

        let (source, index) = self.get_text_document_position(params.text_document_position)?;

        let ast = ast::parse::<ast::Snippet>(source.value());

        // index is the cursor position, we want to offer completions for whatever was typed before.
        let completion = ast.completion(index - 1);

        Ok(completion.map(CompletionResponse::Array))
    }

    async fn signature_help(&self, params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        self.log("received textDocument/signatureHelp").await;

        let (source, index) =
            self.get_text_document_position(params.text_document_position_params)?;

        // index is after the trigger character '(', we want to find the identifier immediately before.
        let source = &source[..index - 1];
        let start = source
            .rfind(|c: char| c.is_ascii_whitespace())
            .map_or(0, |index| index + 1);
        let identifier = &source[start..];

        let help = identifier
            .parse::<FunctionIdentifier>()
            .ok()
            .map(|identifier| {
                let signature = identifier.signature();

                SignatureHelp {
                    signatures: vec![SignatureInformation {
                        label: format!("{identifier}{signature}"),
                        documentation: None,
                        parameters: Some(
                            signature
                                .args
                                .into_iter()
                                .map(|arg| ParameterInformation {
                                    label: ParameterLabel::Simple(arg.to_string()),
                                    documentation: None,
                                })
                                .collect(),
                        ),
                        active_parameter: None,
                    }],
                    active_signature: None,
                    active_parameter: None,
                }
            });

        Ok(help)
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

            let (service, socket) = LspService::build(Backend::new).finish();

            Server::new(stdin, stdout, socket).serve(service).await;
        });
}
