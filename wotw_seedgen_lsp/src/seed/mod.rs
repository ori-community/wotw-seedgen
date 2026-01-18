pub mod cache;
mod completion;
mod hover;
mod semantic_tokens;

use crate::{
    backend::Backend,
    seed::{cache::Cache, hover::hover},
};
use completion::Completion;
use semantic_tokens::{semantic_tokens, semantic_tokens_legend};
use tower_lsp::{
    jsonrpc::Result,
    lsp_types::{
        CompletionOptions, CompletionParams, CompletionResponse, DidChangeTextDocumentParams,
        DidOpenTextDocumentParams, DidSaveTextDocumentParams, Hover, HoverParams,
        HoverProviderCapability, InitializeParams, InitializeResult, ParameterInformation,
        ParameterLabel, SemanticTokens, SemanticTokensFullOptions, SemanticTokensOptions,
        SemanticTokensParams, SemanticTokensResult, SemanticTokensServerCapabilities,
        ServerCapabilities, ServerInfo, SignatureHelp, SignatureHelpOptions, SignatureHelpParams,
        SignatureInformation, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
    },
    LanguageServer,
};
use wotw_seedgen_data::{
    assets::{AssetCacheValues, PlandoFileAccess},
    seed_language::{
        ast,
        compile::{Compiler, FunctionIdentifier},
    },
};

#[tower_lsp::async_trait]
impl LanguageServer for Backend<Cache> {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        self.initialize_base().await;

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
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "wotw_seedgen_seed_lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn shutdown(&self) -> Result<()> {
        self.shutdown_base().await;

        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = self.did_open_base(params).await;

        self.update_diagnostics(uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.did_change_base(params).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = self.did_save_base(params).await;

        self.update_diagnostics(uri).await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let (source, index) = self.hover_base(params).await?;

        let ast = ast::Snippet::parse(source.value());
        let cache = self.cache.read().await;

        Ok(hover(
            source.value(),
            ast.parsed,
            index,
            cache.uber_state_data(),
        ))
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let source = self.semantic_tokens_full_base(params).await?;

        let ast = ast::Snippet::parse(source.value());
        let data = semantic_tokens(source.value(), ast.parsed);

        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            data,
            ..Default::default()
        })))
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let (source, index) = self.completion_base(params).await?;

        let ast = ast::Snippet::parse(source.value());

        let cache = self.cache.read().await;

        // index is the cursor position, we want to offer completions for whatever was typed before.
        let completion = ast.parsed.completion(index - 1, &cache.values);

        Ok(completion.map(CompletionResponse::Array))
    }

    async fn signature_help(&self, params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        let (source, index) = self.signature_help_base(params).await?;

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

impl Backend<Cache> {
    async fn update_diagnostics(&self, uri: Url) {
        self.update_diagnostics_with(uri, async |path| {
            let identifier = self
                .consume_result(
                    path.file_stem()
                        .ok_or_else(|| format!("invalid path \"{}\": not a file", path.display()))
                        .and_then(|identifier| {
                            identifier.to_str().ok_or_else(|| {
                                format!("invalid path \"{}\": invalid unicode", path.display())
                            })
                        }),
                )
                .await?;

            let root = self
                .consume_result(
                    path.parent()
                        .ok_or_else(|| format!("invalid path \"{}\": no parent", path.display())),
                )
                .await?;

            let snippet_access = PlandoFileAccess::new(root);

            let result = {
                let cache = self.cache.read().await;

                let mut compiler = Compiler::new(
                    &mut rand::thread_rng(),
                    &snippet_access,
                    &cache.uber_state_data,
                    Default::default(),
                    false,
                );

                // TODO currently we can only give diagnostics for saved files because we're not using the editors in-memory changes
                // Need to do changes in the language create to improve that
                compiler.compile_snippet(identifier).unwrap(); // TODO have to gracefully exit here, path might be outdated
                compiler.finish()
            };

            let (source, errors) = result
                .errors
                .into_iter()
                .find(|(source, _)| source.id[..source.id.len() - 6].ends_with(identifier))
                .unwrap();

            Some((source.content, errors))
        })
        .await;
    }
}
