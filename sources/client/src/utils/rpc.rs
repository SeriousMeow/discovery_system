use tokio::sync::oneshot;

pub struct Command<Request, Response> {
    pub request: Request,
    pub response_tx: oneshot::Sender<Response>,
}

impl<Request, Response> Command<Request, Response> {
    pub fn new(requset: Request) -> (Self, oneshot::Receiver<Response>) {
        let (tx, rx) = oneshot::channel();

        let command = Command {
            request: requset,
            response_tx: tx,
        };

        (command, rx)
    }

    pub fn send_result(self, response: Response) {
        let _ = self.response_tx.send(response);
    }
}

pub trait Handler<Request, Response> {
    async fn handle(&mut self, command: Command<Request, Response>);
}
