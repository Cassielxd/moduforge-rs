use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{ready, Stream};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use yrs::sync::Error;

/// 客户端 WebSocket Sink 包装器，类似于 WarpSink 但用于客户端
#[derive(Debug)]
pub struct ClientSink(
    pub SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
);

impl futures_util::Sink<Vec<u8>> for ClientSink {
    type Error = Error;

    fn poll_ready(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        let sink = unsafe { Pin::new_unchecked(&mut self.0) };
        let result = ready!(sink.poll_ready(cx));
        match result {
            Ok(_) => Poll::Ready(Ok(())),
            Err(e) => Poll::Ready(Err(Error::Other(Box::new(e)))),
        }
    }

    fn start_send(
        mut self: Pin<&mut Self>,
        item: Vec<u8>,
    ) -> Result<(), Self::Error> {
        let sink = unsafe { Pin::new_unchecked(&mut self.0) };
        let result = sink.start_send(Message::binary(item));
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::Other(Box::new(e))),
        }
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        let sink = unsafe { Pin::new_unchecked(&mut self.0) };
        let result = ready!(sink.poll_flush(cx));
        match result {
            Ok(_) => Poll::Ready(Ok(())),
            Err(e) => Poll::Ready(Err(Error::Other(Box::new(e)))),
        }
    }

    fn poll_close(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        let sink = unsafe { Pin::new_unchecked(&mut self.0) };
        let result = ready!(sink.poll_close(cx));
        match result {
            Ok(_) => Poll::Ready(Ok(())),
            Err(e) => Poll::Ready(Err(Error::Other(Box::new(e)))),
        }
    }
}

/// 客户端 WebSocket Stream 包装器，类似于 WarpStream 但用于客户端
#[derive(Debug)]
pub struct ClientStream(
    pub SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
);

impl Stream for ClientStream {
    type Item = Result<Vec<u8>, Error>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let stream = unsafe { Pin::new_unchecked(&mut self.0) };
        let result = ready!(stream.poll_next(cx));
        match result {
            None => Poll::Ready(None),
            Some(Ok(msg)) => {
                // 处理不同类型的 WebSocket 消息
                let bytes = match msg {
                    Message::Binary(data) => data,
                    Message::Text(text) => text.into_bytes(),
                    Message::Close(_) => {
                        return Poll::Ready(None);
                    },
                    Message::Ping(_) => {
                        // 忽略 ping/pong 消息，继续处理下一个
                        return self.poll_next(cx);
                    },
                    Message::Pong(_) => {
                        // 忽略 ping/pong 消息，继续处理下一个
                        return self.poll_next(cx);
                    },
                    _ => {
                        return self.poll_next(cx);
                    },
                };
                Poll::Ready(Some(Ok(bytes)))
            },
            Some(Err(e)) => Poll::Ready(Some(Err(Error::Other(Box::new(e))))),
        }
    }
}
