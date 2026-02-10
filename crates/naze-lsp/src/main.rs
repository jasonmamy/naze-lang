//! Naze Language Server Protocol implementation.
//!
//! Provides IDE features for .naze files:
//! - Real-time diagnostics (parse and type errors)
//! - Autocompletion
//! - Hover documentation
//! - Go-to-definition

mod capabilities;

use std::sync::Arc;

use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use capabilities::{
    get_code_actions, get_completions, get_definition, get_diagnostics, get_document_symbols,
    get_hover, get_references, get_rename_edits, prepare_rename,
};

/// Document state tracked by the server.
#[derive(Debug, Clone)]
pub struct DocumentState {
    pub content: String,
    pub version: i32,
}

/// The Naze language server.
pub struct NazeLsp {
    client: Client,
    documents: Arc<DashMap<Url, DocumentState>>,
}

impl NazeLsp {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(DashMap::new()),
        }
    }

    /// Validate a document and publish diagnostics.
    async fn validate_document(&self, uri: &Url) {
        let content = match self.documents.get(uri) {
            Some(doc) => doc.content.clone(),
            None => return,
        };

        let diagnostics = get_diagnostics(&content, uri.path());
        self.client
            .publish_diagnostics(uri.clone(), diagnostics, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for NazeLsp {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![
                        ":".to_string(),
                        " ".to_string(),
                        "{".to_string(),
                    ]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                rename_provider: Some(OneOf::Right(RenameOptions {
                    prepare_provider: Some(true),
                    work_done_progress_options: Default::default(),
                })),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "naze-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Naze language server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let content = params.text_document.text;
        let version = params.text_document.version;

        self.documents
            .insert(uri.clone(), DocumentState { content, version });

        self.validate_document(&uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        // We use full sync, so there's only one change containing the full text
        if let Some(change) = params.content_changes.into_iter().next() {
            self.documents.insert(
                uri.clone(),
                DocumentState {
                    content: change.text,
                    version,
                },
            );

            self.validate_document(&uri).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.remove(&params.text_document.uri);
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let content = match self.documents.get(uri) {
            Some(doc) => doc.content.clone(),
            None => return Ok(None),
        };

        let items = get_completions(&content, position);
        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let content = match self.documents.get(uri) {
            Some(doc) => doc.content.clone(),
            None => return Ok(None),
        };

        Ok(get_hover(&content, position))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let content = match self.documents.get(uri) {
            Some(doc) => doc.content.clone(),
            None => return Ok(None),
        };

        Ok(get_definition(&content, uri.path(), position, uri))
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let include_declaration = params.context.include_declaration;

        let content = match self.documents.get(uri) {
            Some(doc) => doc.content.clone(),
            None => return Ok(None),
        };

        let locations = get_references(&content, uri.path(), position, uri, include_declaration);
        if locations.is_empty() {
            Ok(None)
        } else {
            Ok(Some(locations))
        }
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = &params.text_document.uri;

        let content = match self.documents.get(uri) {
            Some(doc) => doc.content.clone(),
            None => return Ok(None),
        };

        let symbols = get_document_symbols(&content, uri.path());
        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = &params.text_document.uri;
        let range = params.range;

        let content = match self.documents.get(uri) {
            Some(doc) => doc.content.clone(),
            None => return Ok(None),
        };

        let actions = get_code_actions(&content, uri.path(), range, &params.context.diagnostics);
        if actions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(actions))
        }
    }

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        let uri = &params.text_document.uri;
        let position = params.position;

        let content = match self.documents.get(uri) {
            Some(doc) => doc.content.clone(),
            None => return Ok(None),
        };

        Ok(prepare_rename(&content, position))
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let new_name = &params.new_name;

        let content = match self.documents.get(uri) {
            Some(doc) => doc.content.clone(),
            None => return Ok(None),
        };

        Ok(get_rename_edits(&content, position, new_name, uri))
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(NazeLsp::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
