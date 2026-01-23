pub mod cache;

mod definition;

use tower_lsp::{
    jsonrpc::Result,
    lsp_types::{
        DidChangeTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
        GotoDefinitionParams, GotoDefinitionResponse, InitializeParams, InitializeResult, OneOf,
        ServerCapabilities, ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
    },
    LanguageServer,
};
use wotw_seedgen_data::{
    assets,
    logic_language::{
        ast::{self, Areas},
        output::Graph,
    },
    parse::ParseResult,
};

use crate::{
    backend::Backend,
    logic::{cache::Cache, definition::goto_definition},
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
                definition_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "wotw_seedgen_logic_lsp".to_string(),
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

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let (source, index) = self.goto_definition_base(params).await?;

        let ast = ast::Areas::parse(source.value());

        Ok(goto_definition(
            ast.parsed,
            index,
            source.key(),
            source.value(),
        ))
    }
}

impl Backend<Cache> {
    async fn update_diagnostics(&self, uri: Url) {
        self.update_diagnostics_with(uri, async |path| {
            let source = self.consume_result(assets::read_to_string(&path)).await?;

            let ParseResult { parsed, mut errors } = Areas::parse(&source);

            if let Some(areas) = parsed {
                let mut result = {
                    let cache = self.cache.read().await;

                    Graph::compile(areas, cache.loc_data.clone(), cache.state_data.clone(), &[])
                };

                errors.append(&mut result.errors);
            }

            Some((source, errors))
        })
        .await;
    }
}
