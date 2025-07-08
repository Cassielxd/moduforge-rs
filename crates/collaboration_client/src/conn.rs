use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{ready, Stream, StreamExt};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use yrs::sync::{
     Error, Protocol,
};

use yrs_warp::conn::Connection;
use yrs_warp::AwarenessRef;

/// 客户端 WebSocket Sink 包装器，类似于 WarpSink 但用于客户端
#[derive(Debug)]
pub struct ClientSink(
    SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
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
    SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
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
                    Message::Binary(data) => {
                        data
                    },
                    Message::Text(text) => {
                        text.into_bytes()
                    },
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
                    Message::Frame(_) => {
                        return self.poll_next(cx);
                    },
                };
                Poll::Ready(Some(Ok(bytes)))
            },
            Some(Err(e)) => Poll::Ready(Some(Err(Error::Other(Box::new(e))))),
        }
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub struct ClientConn(pub Connection<ClientSink, ClientStream>);

impl ClientConn {
    /// 连接到指定的服务器地址并创建新的客户端连接
    pub async fn connect(
        url: &str,
        awareness: AwarenessRef,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let (ws_stream, _) = connect_async(url).await?;
        let (sink, stream) = ws_stream.split();
        let conn =
            Connection::new(awareness, ClientSink(sink), ClientStream(stream));
        Ok(ClientConn(conn))
    }

    /// 连接到指定的服务器地址并使用自定义协议创建新的客户端连接
    pub async fn connect_with_protocol<P: Protocol + Send + Sync + 'static>(
        url: &str,
        awareness: AwarenessRef,
        protocol: P,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let (ws_stream, _) = connect_async(url).await?;
        let (sink, stream) = ws_stream.split();
        let conn = Connection::with_protocol(
            awareness,
            ClientSink(sink),
            ClientStream(stream),
            protocol,
        );
        Ok(ClientConn(conn))
    }

    /// 获取 awareness 的引用
    pub fn awareness(&self) -> &AwarenessRef {
        self.0.awareness()
    }
}

impl core::future::Future for ClientConn {
    type Output = Result<(), Error>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        match Pin::new(&mut self.0).poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(Err(Error::Other(e.into()))),
            Poll::Ready(Ok(_)) => Poll::Ready(Ok(())),
        }
    }
}
