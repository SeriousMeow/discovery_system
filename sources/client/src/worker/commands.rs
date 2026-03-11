use derive_more::From;
use tokio::sync::oneshot;

pub struct CommandWithResponse<B, R> {
    pub body: B,
    pub response: oneshot::Sender<R>,
}

impl<B, R> CommandWithResponse<B, R> {
    pub fn new(body: B, tx: oneshot::Sender<R>) -> Self {
        Self {
            body: body,
            response: tx,
        }
    }
}

pub mod connect {
    use iroh::endpoint::{RecvStream, SendStream};
    use iroh_tickets::endpoint::EndpointTicket;

    pub type Command = super::CommandWithResponse<Body, Response>;

    pub struct Body {
        pub ticket: EndpointTicket,
    }

    pub(crate) struct Response {
        pub rx: RecvStream,
        pub tx: SendStream,
    }
}

#[derive(From)]
pub(crate) enum Command {
    Connect(connect::Command),
}
