#![allow(dead_code)]
use futures_util::sink::SinkExt;
use futures_util::StreamExt;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::{Arc, Weak};
use std::task::{Context, Poll};
use tokio::spawn;
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;
use yrs::encoding::read::Cursor;
use yrs::sync::Awareness;
use yrs::sync::{
    DefaultProtocol, Error, Message, MessageReader, Protocol, SyncMessage,
};
use yrs::updates::decoder::{Decode, DecoderV1};
use yrs::updates::encoder::{Encode, Encoder, EncoderV1};
use yrs::Update;
use std::time::Instant;

/// 链接处理，通过消息流实现 Yjs/Yrs 意识和更新交换协议。
///
/// 这个连接实现了 Future 模式，可以被等待，以便调用者识别底层 websocket 连接是否已优雅地完成或突然结束。
#[derive(Debug)]
pub struct Connection<Sink, Stream> {
    processing_loop: JoinHandle<Result<(), Error>>,
    awareness: Arc<RwLock<Awareness>>,
    inbox: Arc<Mutex<Sink>>,
    sync_tracker: Arc<RwLock<SyncTracker>>, // 新增同步跟踪器
    _stream: PhantomData<Stream>,
}

impl<Sink, Stream, E> Connection<Sink, Stream>
where
    Sink: SinkExt<Vec<u8>, Error = E> + Send + Sync + Unpin + 'static,
    E: Into<Error> + Send + Sync,
{
    pub async fn send(
        &self,
        msg: Vec<u8>,
    ) -> Result<(), Error> {
        let mut inbox = self.inbox.lock().await;
        match inbox.send(msg).await {
            Ok(_) => Ok(()),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn close(self) -> Result<(), E> {
        let mut inbox = self.inbox.lock().await;
        inbox.close().await
    }

    pub fn sink(&self) -> Weak<Mutex<Sink>> {
        Arc::downgrade(&self.inbox)
    }
}

impl<Sink, Stream, E> Connection<Sink, Stream>
where
    Stream:
        StreamExt<Item = Result<Vec<u8>, E>> + Send + Sync + Unpin + 'static,
    Sink: SinkExt<Vec<u8>, Error = E> + Send + Sync + Unpin + 'static,
    E: Into<Error> + Send + Sync,
{
    /// 创建带同步检测的连接
    pub fn new_with_sync_detection(
        awareness: Arc<RwLock<Awareness>>,
        sink: Sink,
        stream: Stream,
        event_sender: Option<SyncEventSender>,
    ) -> Self {
        let sync_tracker =
            Arc::new(RwLock::new(SyncTracker::new(event_sender)));
        Self::with_protocol_and_sync(
            awareness,
            sink,
            stream,
            DefaultProtocol,
            sync_tracker,
        )
    }
    /// 创建带协议和同步检测的连接
    pub fn with_protocol_and_sync<P>(
        awareness: Arc<RwLock<Awareness>>,
        sink: Sink,
        mut stream: Stream,
        protocol: P,
        sync_tracker: Arc<RwLock<SyncTracker>>,
    ) -> Self
    where
        P: Protocol + Send + Sync + 'static,
    {
        let sink = Arc::new(Mutex::new(sink));
        let inbox = sink.clone();
        let loop_sink = Arc::downgrade(&sink);
        let loop_awareness = Arc::downgrade(&awareness);
        let loop_sync_tracker = Arc::downgrade(&sync_tracker);

        let processing_loop: JoinHandle<Result<(), Error>> =
            spawn(async move {
                // 发送 SyncStep1
                let payload = {
                    let awareness = loop_awareness.upgrade().unwrap();
                    let mut encoder = EncoderV1::new();
                    let awareness = awareness.read().await;
                    protocol.start(&awareness, &mut encoder)?;
                    encoder.to_vec()
                };

                if !payload.is_empty() {
                    // 🔥 标记 Step1 已发送
                    if let Some(tracker) = loop_sync_tracker.upgrade() {
                        tracker.read().await.on_step1_sent();
                    }

                    if let Some(sink) = loop_sink.upgrade() {
                        let mut s = sink.lock().await;
                        if let Err(e) = s.send(payload).await {
                            return Err(e.into());
                        }
                    } else {
                        return Ok(());
                    }
                }

                // 消息处理循环
                while let Some(input) = stream.next().await {
                    match input {
                        Ok(data) => {
                            if let Some(mut sink) = loop_sink.upgrade() {
                                if let Some(awareness) =
                                    loop_awareness.upgrade()
                                {
                                    if let Some(sync_tracker) =
                                        loop_sync_tracker.upgrade()
                                    {
                                        match Self::process_with_sync_detection(
                                            &protocol,
                                            &awareness,
                                            &mut sink,
                                            &sync_tracker,
                                            data,
                                        )
                                        .await
                                        {
                                            Ok(()) => { /* continue */ },
                                            Err(e) => return Err(e),
                                        }
                                    }
                                } else {
                                    return Ok(());
                                }
                            } else {
                                return Ok(());
                            }
                        },
                        Err(e) => return Err(e.into()),
                    }
                }

                Ok(())
            });

        Connection {
            processing_loop,
            awareness,
            inbox,
            sync_tracker,
            _stream: PhantomData,
        }
    }
    /// 带同步检测的消息处理
    async fn process_with_sync_detection<P: Protocol>(
        protocol: &P,
        awareness: &Arc<RwLock<Awareness>>,
        sink: &mut Arc<Mutex<Sink>>,
        sync_tracker: &Arc<RwLock<SyncTracker>>,
        input: Vec<u8>,
    ) -> Result<(), Error> {
        let mut decoder = DecoderV1::new(Cursor::new(&input));
        let reader = MessageReader::new(&mut decoder);

        for r in reader {
            let msg = r?;

            // 🔥 在处理消息前检测同步状态
            Self::track_sync_message(&msg, sync_tracker).await;

            if let Some(reply) = handle_msg(protocol, awareness, msg).await? {
                let mut sender = sink.lock().await;
                if let Err(e) = sender.send(reply.encode_v1()).await {
                    tracing::error!("连接发送回复失败");
                    return Err(e.into());
                }
            }
        }

        Ok(())
    }
    /// 跟踪同步消息
    async fn track_sync_message(
        msg: &Message,
        sync_tracker: &Arc<RwLock<SyncTracker>>,
    ) {
        if let Message::Sync(sync_msg) = msg {
            match sync_msg {
                SyncMessage::SyncStep2(_) => {
                    // 🎉 收到 Step2，首次同步完成！
                    let mut tracker = sync_tracker.write().await;
                    tracker.on_step2_received();
                },
                SyncMessage::Update(_) => {
                    // 收到数据更新
                    let tracker = sync_tracker.read().await;
                    tracker.on_update_received();
                },
                _ => {},
            }
        }
    }

    /// 获取同步跟踪器
    pub fn sync_tracker(&self) -> &Arc<RwLock<SyncTracker>> {
        &self.sync_tracker
    }

    /// 等待初始同步完成
    pub async fn wait_for_initial_sync(
        &self,
        timeout_ms: u64,
    ) -> bool {
        let start_time = Instant::now();
        let timeout_duration = tokio::time::Duration::from_millis(timeout_ms);

        loop {
            {
                let tracker = self.sync_tracker.read().await;
                if tracker.is_initial_sync_completed() {
                    return true;
                }
            }

            if start_time.elapsed() >= timeout_duration {
                break;
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }

        false
    }

    /// 获取当前协议同步状态
    pub async fn get_protocol_sync_state(&self) -> ProtocolSyncState {
        self.sync_tracker.read().await.get_protocol_state()
    }
    /// Returns an underlying [Awareness] structure, that contains client state of that connection.
    pub fn awareness(&self) -> &Arc<RwLock<Awareness>> {
        &self.awareness
    }
}

impl<Sink, Stream> Unpin for Connection<Sink, Stream> {}

impl<Sink, Stream> Future for Connection<Sink, Stream> {
    type Output = Result<(), Error>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        match Pin::new(&mut self.processing_loop).poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(Err(Error::Other(e.into()))),
            Poll::Ready(Ok(r)) => Poll::Ready(r),
        }
    }
}

pub async fn handle_msg<P: Protocol>(
    protocol: &P,
    a: &Arc<RwLock<Awareness>>,
    msg: Message,
) -> Result<Option<Message>, Error> {
    match msg {
        Message::Sync(msg) => match msg {
            SyncMessage::SyncStep1(sv) => {
                let awareness = a.read().await;
                protocol.handle_sync_step1(&awareness, sv)
            },
            SyncMessage::SyncStep2(update) => {
                let mut awareness = a.write().await;
                protocol.handle_sync_step2(
                    &mut awareness,
                    Update::decode_v1(&update)?,
                )
            },
            SyncMessage::Update(update) => {
                let mut awareness = a.write().await;
                protocol
                    .handle_update(&mut awareness, Update::decode_v1(&update)?)
            },
        },
        Message::Auth(reason) => {
            let awareness = a.read().await;
            protocol.handle_auth(&awareness, reason)
        },
        Message::AwarenessQuery => {
            let awareness = a.read().await;
            protocol.handle_awareness_query(&awareness)
        },
        Message::Awareness(update) => {
            let mut awareness = a.write().await;
            protocol.handle_awareness_update(&mut awareness, update)
        },
        Message::Custom(tag, data) => {
            let mut awareness = a.write().await;
            protocol.missing_handle(&mut awareness, tag, data)
        },
    }
}

use crate::types::{ConnectionError, ProtocolSyncState, SyncEvent, SyncEventSender};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

/// 同步状态跟踪器
#[derive(Debug)]
pub struct SyncTracker {
    protocol_state: AtomicU8, // 0=NotStarted, 1=Step1Sent, 2=Step2Received, 3=Updating
    has_data: AtomicBool,
    start_time: Option<Instant>,
    step2_time: Option<Instant>,
    event_sender: Option<SyncEventSender>,
}

impl SyncTracker {
    pub fn new(event_sender: Option<SyncEventSender>) -> Self {
        Self {
            protocol_state: AtomicU8::new(0),
            has_data: AtomicBool::new(false),
            start_time: Some(Instant::now()),
            step2_time: None,
            event_sender,
        }
    }
    pub fn on_step1_sent(&self) {
        let prev = self.protocol_state.swap(1, Ordering::Relaxed);
        if prev == 0 {
            tracing::debug!("📡 协议: SyncStep1 已发送");
            self.emit_event(SyncEvent::ProtocolStateChanged(
                ProtocolSyncState::Step1Sent,
            ));
        }
    }
    pub fn on_step2_received(&mut self) -> bool {
        let prev = self.protocol_state.swap(2, Ordering::Relaxed);
        if prev == 1 {
            // Step1 -> Step2，首次同步完成！
            self.step2_time = Some(Instant::now());

            let elapsed_ms = if let (Some(start), Some(step2)) =
                (self.start_time, self.step2_time)
            {
                step2.duration_since(start).as_millis() as u64
            } else {
                0
            };

            let has_data = self.has_data.load(Ordering::Relaxed);

            tracing::info!(
                "🎉 协议同步完成: Step1->Step2, 耗时 {}ms, 有数据: {}",
                elapsed_ms,
                has_data
            );

            self.emit_event(SyncEvent::ProtocolStateChanged(
                ProtocolSyncState::Step2Received,
            ));
            self.emit_event(SyncEvent::InitialSyncCompleted {
                has_data,
                elapsed_ms,
            });

            return true; // 首次同步完成
        }
        false
    }
    pub fn on_update_received(&self) {
        let prev_state = self.protocol_state.load(Ordering::Relaxed);

        // 标记有数据
        self.has_data.store(true, Ordering::Relaxed);

        // 如果还在Step2状态，切换到Updating
        if prev_state == 2 {
            self.protocol_state.store(3, Ordering::Relaxed);
            self.emit_event(SyncEvent::ProtocolStateChanged(
                ProtocolSyncState::Updating,
            ));
        }

        self.emit_event(SyncEvent::DataReceived);
    }

    pub fn is_initial_sync_completed(&self) -> bool {
        self.protocol_state.load(Ordering::Relaxed) >= 2
    }

    pub fn get_protocol_state(&self) -> ProtocolSyncState {
        match self.protocol_state.load(Ordering::Relaxed) {
            0 => ProtocolSyncState::NotStarted,
            1 => ProtocolSyncState::Step1Sent,
            2 => ProtocolSyncState::Step2Received,
            3 => ProtocolSyncState::Updating,
            _ => ProtocolSyncState::NotStarted,
        }
    }
    fn emit_event(
        &self,
        event: SyncEvent,
    ) {
        if let Some(sender) = &self.event_sender {
            let _ = sender.send(event);
        }
    }
    /// 重置同步状态（用于重连）
    pub fn reset(&mut self) {
        self.protocol_state.store(0, Ordering::Relaxed);
        self.has_data.store(false, Ordering::Relaxed);
        self.start_time = Some(Instant::now());
        self.step2_time = None;
    }

    /// 标记连接失败
    pub fn on_connection_failed(
        &self,
        error: &ConnectionError,
    ) {
        tracing::error!("🔌 连接失败: {}", error);
        self.emit_event(SyncEvent::ConnectionFailed(error.clone()));
    }
}
