use std::collections::HashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
// Removed unused imports

mod advanced;
mod completion;
mod hover;
mod parser;
mod validation;

use advanced::{AdvancedFeatures, PerformanceMetrics};
use completion::CompletionProvider;
use hover::HoverProvider;
use parser::AssParser;
use std::time::Instant;
use validation::ValidationProvider;

pub struct AssLanguageServer {
    client: Client,
    parser: AssParser,
    completion: CompletionProvider,
    hover: HoverProvider,
    validation: ValidationProvider,
    document_map: tokio::sync::RwLock<HashMap<Url, String>>,
    advanced_features: tokio::sync::RwLock<HashMap<String, AdvancedFeatures>>,
}

impl AssLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            parser: AssParser::new(),
            completion: CompletionProvider::new(),
            hover: HoverProvider::new(),
            validation: ValidationProvider::new(),
            document_map: tokio::sync::RwLock::new(HashMap::new()),
            advanced_features: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    async fn on_change(&self, uri: Url, text: String) {
        let start_time = Instant::now();
        let mut document_map = self.document_map.write().await;
        document_map.insert(uri.clone(), text.clone());
        drop(document_map);

        // Performance tracking
        let parse_start = Instant::now();
        let parsed = self.parser.parse(&text);
        let parse_time = parse_start.elapsed();

        let validation_start = Instant::now();
        let mut diagnostics = self.validation.validate(&parsed);
        let validation_time = validation_start.elapsed();

        // Advanced features
        let file_path = uri.to_string();
        let mut advanced_map = self.advanced_features.write().await;
        let advanced = advanced_map
            .entry(file_path.clone())
            .or_insert_with(|| AdvancedFeatures::new(file_path.clone()));

        // Advanced validation
        let style_warnings = advanced.analyze_style_inheritance(&text);
        let timing_warnings = advanced.detect_timing_overlaps(&text);
        let advanced_warnings = advanced.validate_advanced(&text);

        // Log timing summary
        let timing_summary = advanced.get_timing_summary();
        if !timing_summary.is_empty() && timing_summary != "No timing overlaps detected" {
            self.client
                .log_message(MessageType::INFO, timing_summary)
                .await;
        }

        // Add advanced warnings as diagnostics
        for warning in style_warnings
            .iter()
            .chain(timing_warnings.iter())
            .chain(advanced_warnings.iter())
        {
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position::new(0, 0),
                    end: Position::new(0, 0),
                },
                severity: Some(DiagnosticSeverity::WARNING),
                code: None,
                code_description: None,
                source: Some("ass-lsp-advanced".to_string()),
                message: warning.clone(),
                related_information: None,
                tags: None,
                data: None,
            });
        }

        // Record performance metrics
        let total_time = start_time.elapsed();
        let metrics = PerformanceMetrics {
            parse_time,
            validation_time,
            completion_time: std::time::Duration::default(),
            total_time,
            file_size: text.len(),
            lines_count: text.lines().count(),
        };
        advanced.record_performance_metrics(metrics);

        // Log performance suggestions
        let suggestions = advanced.get_performance_suggestions();
        for suggestion in suggestions {
            self.client.log_message(MessageType::INFO, suggestion).await;
        }

        drop(advanced_map);

        // Send diagnostics to client
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for AssLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![
                        "\\".to_string(),
                        "{".to_string(),
                        ",".to_string(),
                        ":".to_string(),
                    ]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    completion_item: None,
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("ass-lsp".to_string()),
                        inter_file_dependencies: false,
                        workspace_diagnostics: false,
                        work_done_progress_options: Default::default(),
                    },
                )),
                document_formatting_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                ..ServerCapabilities::default()
            },
            server_info: Some(ServerInfo {
                name: "ass-lsp".to_string(),
                version: Some("0.1.0".to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "ASS Language Server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file opened!")
            .await;
        self.on_change(params.text_document.uri, params.text_document.text)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.into_iter().next() {
            self.on_change(params.text_document.uri, change.text).await;
        }
    }

    async fn did_save(&self, _: DidSaveTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file saved!")
            .await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let mut document_map = self.document_map.write().await;
        document_map.remove(&params.text_document.uri);
        self.client
            .log_message(MessageType::INFO, "file closed!")
            .await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let document_map = self.document_map.read().await;
        if let Some(text) = document_map.get(uri) {
            let completions = self.completion.provide_completions(text, position);
            return Ok(Some(CompletionResponse::Array(completions)));
        }

        Ok(None)
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let document_map = self.document_map.read().await;
        if let Some(text) = document_map.get(uri) {
            return Ok(self.hover.provide_hover(text, position));
        }

        Ok(None)
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = &params.text_document.uri;

        let document_map = self.document_map.read().await;
        if let Some(text) = document_map.get(uri) {
            let formatted = self.parser.format(text);
            if formatted != *text {
                return Ok(Some(vec![TextEdit {
                    range: Range {
                        start: Position::new(0, 0),
                        end: Position::new(u32::MAX, 0),
                    },
                    new_text: formatted,
                }]));
            }
        }

        Ok(None)
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = &params.text_document.uri;

        let document_map = self.document_map.read().await;
        if let Some(text) = document_map.get(uri) {
            let symbols = self.parser.extract_symbols(text);
            return Ok(Some(DocumentSymbolResponse::Nested(symbols)));
        }

        Ok(None)
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| AssLanguageServer::new(client)).finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
