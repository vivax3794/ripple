use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::mpsc;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use crate::css::Css;

#[derive(Debug)]
struct Backend {
    client: Arc<Client>,
    documents: DashMap<Url, mpsc::Sender<Vec<TextDocumentContentChangeEvent>>>,
}

impl Backend {
    fn new(client: Client) -> Self {
        Self {
            client: Arc::new(client),
            documents: DashMap::new(),
        }
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
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let TextDocumentItem {
            uri,
            language_id,
            text,
            ..
        } = params.text_document;

        if language_id == "css" {
            let parser =
                ripple_parser::construct_css_parser().expect("Failed to load css grammar.");
            match ripple_parser::Document::parse(text, parser) {
                Ok(document) => {
                    let (processor, channel) = DocumentProcessor::<Css>::new(uri.clone(), document);
                    let client_copy = Arc::clone(&self.client);
                    tokio::spawn(processor.do_loop(client_copy));

                    self.documents.insert(uri, channel);
                }
                Err(err) => {
                    let range = Range::new(Position::new(0, 0), Position::new(0, 0));
                    let diag = Diagnostic {
                        range,
                        severity: Some(DiagnosticSeverity::ERROR),
                        message: format!("{err}"),
                        ..Default::default()
                    };
                    self.client.publish_diagnostics(uri, vec![diag], None).await;
                }
            }
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let Some(channel) = self.documents.get(&params.text_document.uri) else {
            return;
        };

        channel.send(params.content_changes).await.unwrap();
    }
}

pub trait Language: Default {
    fn diagnostics(&self, document: &ripple_parser::Document) -> Vec<Diagnostic>;
}

struct DocumentProcessor<L: Language> {
    url: Url,
    document: ripple_parser::Document,
    edits: mpsc::Receiver<Vec<TextDocumentContentChangeEvent>>,
    lang: L,
}

impl<L: Language> DocumentProcessor<L> {
    fn new(
        url: Url,
        document: ripple_parser::Document,
    ) -> (Self, mpsc::Sender<Vec<TextDocumentContentChangeEvent>>) {
        let (tx, rx) = mpsc::channel(10);
        (
            Self {
                url,
                document,
                edits: rx,
                lang: L::default(),
            },
            tx,
        )
    }

    /// Update the TS spans, and the docuemnt text based on the edit.
    fn apply_edit(&mut self, edit: TextDocumentContentChangeEvent) {
        if let Some(range) = edit.range {
            let edit = ripple_parser::Edit {
                start_line: range.start.line as usize,
                start_collumn: range.start.character as usize,
                end_line: range.end.line as usize,
                end_collumn: range.end.character as usize,
                new_text: edit.text,
            };
            self.document.apply_edit(edit);
        } else {
            self.document.replace_document(edit.text);
        }
    }

    async fn do_loop(mut self, client: Arc<Client>) {
        let mut edits = Vec::new();
        loop {
            client
                .publish_diagnostics(
                    self.url.clone(),
                    self.lang.diagnostics(&self.document),
                    None,
                )
                .await;

            // Read all pending edits
            // If there are no edits in the queue this will wait for at least one
            // (i.e we dont have to worry about doing extra work when nothing changed)
            let amount = self.edits.recv_many(&mut edits, usize::MAX).await;
            // According to docs for (non-zero limit) this means the channel is closed
            if amount == 0 {
                break;
            }

            // Each edit from the editor needs to be applied to the full text of the document and
            // the TS tree spans.
            for edit_calls in edits.drain(..) {
                for edit in edit_calls {
                    self.apply_edit(edit);
                }
            }
            // but the actualy reparsing only needs to run for the combined changes
            self.document.reparse();
        }
    }
}

pub fn node_range(node: &ripple_parser::tree_sitter::Node) -> Range {
    let start = node.start_position();
    let end = node.end_position();

    let start = Position {
        line: start.row as u32,
        character: start.column as u32,
    };
    let end = Position {
        line: end.row as u32,
        character: end.column as u32,
    };

    Range { start, end }
}

#[tokio::main]
pub async fn lsp_main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
