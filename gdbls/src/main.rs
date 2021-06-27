use std::error::Error;

use lsp_types::{
    notification, request, GotoDefinitionResponse, InitializeParams, OneOf, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind,
};

use lsp_server::{Connection, Message, RequestId, Response};

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
    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                eprintln!("got request: {:?}", req);

                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }

                if let Ok((id, params)) = cast_request::<request::GotoDefinition>(req) {
                    eprintln!("got GotoDefinition request #{}: {:?}", id, params);
                    let result = Some(GotoDefinitionResponse::Array(Vec::new()));
                    let result = serde_json::to_value(&result).unwrap();
                    let resp = Response {
                        id,
                        result: Some(result),
                        error: None,
                    };
                    connection.sender.send(Message::Response(resp))?;
                };
            }
            Message::Response(resp) => {
                eprintln!("got response: {:?}", resp);
            }
            Message::Notification(not) => {
                eprintln!("got notification: {:?}", not);

                if let Ok((id, params)) =
                    cast_notification::<notification::DidOpenTextDocument>(not)
                {
                    eprintln!("got DidOpenTextDocument notification #{}: {:?}", id, params);
                };
            }
        }
    }
    Ok(())
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
) -> Result<(RequestId, N::Params), lsp_server::Notification>
where
    N: notification::Notification,
    N::Params: serde::de::DeserializeOwned,
{
    notification.extract(N::METHOD)
}
