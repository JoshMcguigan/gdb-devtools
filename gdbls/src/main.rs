use language_model::{FilePosition, Semantics};

use std::{env, error::Error, fs, path::PathBuf};

use lsp_server::{Connection, Message, RequestId, Response};
use lsp_types::{
    notification, request, GotoDefinitionResponse, InitializeParams, OneOf, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind,
};

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    eprintln!("starting generic LSP server");

    let (connection, io_threads) = Connection::stdio();

    let server_capabilities = {
        let mut cap = ServerCapabilities::default();
        cap.definition_provider = Some(OneOf::Left(true));

        cap.text_document_sync = Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::Full));

        serde_json::to_value(&cap).unwrap()
    };

    let initialization_params = connection.initialize(server_capabilities)?;
    main_loop(&connection, initialization_params)?;
    io_threads.join()?;

    eprintln!("shutting down server");
    Ok(())
}

fn main_loop(
    connection: &Connection,
    params: serde_json::Value,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    let _params: InitializeParams = serde_json::from_value(params).unwrap();
    eprintln!("starting main loop");

    let mut semantics = Semantics::new(env::current_dir()?);

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                eprintln!("got request: {:?}", req);

                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }

                if let Ok((id, params)) = cast_request::<request::GotoDefinition>(req) {
                    eprintln!("got GotoDefinition request #{}: {:?}", id, params);
                    let result = match semantics.find_definition(FilePosition {
                        file: &params
                            .text_document_position_params
                            .text_document
                            .uri
                            .to_file_path()
                            .unwrap(),
                        line: params.text_document_position_params.position.line as usize,
                        column: params.text_document_position_params.position.character as usize,
                    }) {
                        Some(definition_position) => {
                            let pos = lsp_types::Position {
                                line: definition_position.line as u32,
                                character: definition_position.column as u32,
                            };
                            // We are using an empty range here to indicate a specific
                            // location.
                            let range = lsp_types::Range {
                                start: pos,
                                end: pos,
                            };
                            let result =
                                Some(GotoDefinitionResponse::from(lsp_types::Location::new(
                                    lsp_types::Url::from_file_path(definition_position.file)
                                        .unwrap(),
                                    range,
                                )));
                            Some(serde_json::to_value(&result).unwrap())
                        }
                        None => None,
                    };
                    // TODO we must always return either a result or an error
                    //
                    // If we don't find the definition, is that supposed to be
                    // represented as an empty result or as an error?
                    let resp = Response {
                        id,
                        result,
                        error: None,
                    };
                    connection.sender.send(Message::Response(resp))?;
                };
            }
            Message::Response(resp) => {
                eprintln!("got response: {:?}", resp);
            }
            Message::Notification(notification) => {
                eprintln!("got notification: {:#?}", notification);

                let notification =
                    match cast_notification::<notification::DidOpenTextDocument>(notification) {
                        Ok(params) => {
                            eprintln!("got DidOpenTextDocument notification: {:?}", params);
                            recursively_set_file_text(
                                &mut semantics,
                                // This unwrap fails if using file URIs which are not
                                // file: scheme.
                                params.text_document.uri.to_file_path().unwrap(),
                                params.text_document.text,
                            );
                            continue;
                        }
                        Err(notification) => notification,
                    };
                let _notification =
                    match cast_notification::<notification::DidChangeTextDocument>(notification) {
                        Ok(mut params) => {
                            eprintln!("got DidChangeTextDocument notification: {:?}", params);
                            recursively_set_file_text(
                                &mut semantics,
                                // This unwrap fails if using file URIs which are not
                                // file: scheme.
                                params.text_document.uri.to_file_path().unwrap(),
                                // We are assuming here that the client is sending the
                                // full file, as this is how we initialize our config.
                                params.content_changes.pop().unwrap().text,
                            );
                            continue;
                        }
                        Err(notification) => notification,
                    };
            }
        }
    }
    Ok(())
}

fn recursively_set_file_text(semantics: &mut Semantics, path: PathBuf, text: String) {
    let unresolved_paths = semantics.set_file_text(path, text);

    for path in unresolved_paths.into_iter() {
        if let Ok(text) = fs::read_to_string(&path) {
            recursively_set_file_text(semantics, path, text);
        }
    }
}

fn cast_request<R>(req: lsp_server::Request) -> Result<(RequestId, R::Params), lsp_server::Request>
where
    R: request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}

fn cast_notification<N>(
    notification: lsp_server::Notification,
) -> Result<N::Params, lsp_server::Notification>
where
    N: notification::Notification,
    N::Params: serde::de::DeserializeOwned,
{
    notification.extract(N::METHOD)
}
