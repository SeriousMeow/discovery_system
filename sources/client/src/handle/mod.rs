use iroh::endpoint::{RecvStream, SendStream};
use iroh_tickets::endpoint::EndpointTicket;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

use crate::worker::commands;

#[derive(Clone)]
pub struct EmptyHandle {
    commands_sender: Sender<commands::Command>,
}

pub struct Handle {
    commands_sender: Sender<commands::Command>,
    data_sender: SendStream,
    data_reciever: RecvStream,
}

impl EmptyHandle {
    pub(crate) fn new(commands_sender: Sender<commands::Command>) -> Self {
        Self {
            commands_sender: commands_sender,
        }
    }

    pub async fn connect(self, ticket: EndpointTicket) -> Handle {
        use commands::connect::*;

        let (tx, rx) = oneshot::channel();

        let body: Body = Body { ticket: ticket };

        let command = Command::new(body, tx);

        self.commands_sender.send(command.into()).await.unwrap();

        let response = rx.await.unwrap();

        Handle {
            commands_sender: self.commands_sender,
            data_sender: response.tx,
            data_reciever: response.rx,
        }
    }
}

impl Handle {
    pub fn clone_empty(&self) -> EmptyHandle {
        EmptyHandle {
            commands_sender: self.commands_sender.clone(),
        }
    }
}

impl AsyncRead for Handle {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        AsyncRead::poll_read(
            std::pin::Pin::new(&mut self.get_mut().data_reciever),
            cx,
            buf,
        )
    }
}

impl AsyncWrite for Handle {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        AsyncWrite::poll_write(std::pin::Pin::new(&mut self.get_mut().data_sender), cx, buf)
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        AsyncWrite::poll_flush(std::pin::Pin::new(&mut self.get_mut().data_sender), cx)
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        AsyncWrite::poll_shutdown(std::pin::Pin::new(&mut self.get_mut().data_sender), cx)
    }
}
