use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
    time::Duration,
};
use serde_json::Value;
use tokio::sync::RwLock;

use async_trait::async_trait;
use mf_collab_client::{
    provider::WebsocketProvider,
    types::SyncEvent,
    utils::Utils,
    yrs::{
        sync::{awareness::Event, Awareness},
        types::{Change, EntryChange},
        Doc,
    },
    AwarenessRef,
};
use mf_core::{
    runtime::async_runtime::ForgeAsyncRuntime,
    error_utils,
    extension::Extension,
    history_manager::HistoryManager,
    types::{Content, Extensions, HistoryEntryWithMeta, RuntimeOptions},
    ForgeError, ForgeResult,
};
use mf_model::node_pool::NodePool;
use mf_state::{
    plugin::{Plugin, PluginSpec},
    resource::Resource,
    resource_table::ResourceId,
    transaction::Command,
    State, Transaction,
};

use crate::{plugins::collab::CollabStateField, types::EditorTrait};

pub struct CollabEditorOptions {
    pub editor_options: RuntimeOptions,
    pub server_url: String,
    pub room_name: String,
}
impl CollabEditorOptions {
    pub fn new(
        server_url: String,
        room_name: String,
    ) -> Self {
        Self {
            editor_options: RuntimeOptions::default(),
            server_url,
            room_name,
        }
    }
}

pub struct CollabEditor {
    /// 内部异步编辑器实例，处理底层编辑操作
    ///
    /// 负责状态管理、撤销/重做操作以及资源跟踪等基础功能
    /// 使用 Arc<RwLock> 支持并发访问
    editor: Arc<RwLock<ForgeAsyncRuntime>>,
    /// 协作编辑器提供者
    ///
    sync_manager: Arc<CollabSyncManager>,

    /// 编辑器配置选项
    ///
    /// 包含创建和运行编辑器所需的各项配置，如存储接口和规则加载器
    options: CollabEditorOptions,
}

#[async_trait]
impl EditorTrait for CollabEditor {
    async fn get_state(&self) -> Arc<State> {
        // 同样的问题，需要异步访问
        panic!("get_state 需要异步访问，请使用 get_state_async")
    }

    async fn doc(&self) -> Arc<NodePool> {
        // 同样的问题，需要异步访问
        panic!("doc 需要异步访问，请使用 doc_async")
    }

    async fn command(
        &mut self,
        command: Arc<dyn Command>,
    ) -> ForgeResult<()> {
        let mut editor = self.editor.write().await;
        editor.command(command).await
    }

    async fn command_with_meta(
        &mut self,
        command: Arc<dyn Command>,
        description: String,
        meta: serde_json::Value,
    ) -> ForgeResult<()> {
        let mut editor = self.editor.write().await;
        editor.command_with_meta(command, description, meta).await
    }

    async fn dispatch_flow(
        &mut self,
        transaction: Transaction,
    ) -> ForgeResult<()> {
        let mut editor = self.editor.write().await;
        editor.dispatch_flow(transaction).await
    }

    async fn dispatch_flow_with_meta(
        &mut self,
        transaction: Transaction,
        description: String,
        meta: serde_json::Value,
    ) -> ForgeResult<()> {
        let mut editor = self.editor.write().await;
        editor.dispatch_flow_with_meta(transaction, description, meta).await
    }
}
impl CollabEditor {
    pub async fn create(options: CollabEditorOptions) -> ForgeResult<Self> {
        // 创建协作同步管理器
        let mut sync_manager = CollabSyncManager::new(&options).await?;
        // 订阅同步事件
        let mut event_rx =
            sync_manager.provider.subscribe_sync_events().unwrap();
        // 启动协作同步
        sync_manager.start().await;
        // 等待初始化完成
        // 设置超时 10 秒
        let timeout = tokio::time::timeout(Duration::from_secs(10), async {
            while let Ok(event) = event_rx.recv().await {
                if let SyncEvent::InitialSyncCompleted { has_data, .. } = event
                {
                    return Ok(has_data);
                }
            }
            Err(ForgeError::Internal {
                message: "服务器异常，请检查服务器是否启动".to_string(),
                location: None,
            })
        })
        .await
        .map_err(|e| ForgeError::Internal {
            message: "服务器异常，请检查服务器是否启动".to_string(),
            location: None,
        })?;
        // 创建协作扩展
        let mut ext = Extension::new();
        ext.add_plugin({
            Arc::new(Plugin::new(PluginSpec {
                state_field: Some(Arc::new(CollabStateField::new(
                    sync_manager.awareness.clone(),
                ))),
                key: ("collab".to_string(), "协作".to_string()),
                tr: None,
                priority: 0,
            }))
        });
        // 添加协作扩展
        let mut options = options;
        options.editor_options =
            options.editor_options.add_extension(Extensions::E(ext));
        // 创建编辑器
        let editor = match timeout {
            Ok(true) => {
                //有数据 反向同步数据 并创建 编辑器
                let awareness_lock = sync_manager.awareness.read().await;
                //把 远程的 文档 转换成 树
                match Utils::apply_yrs_to_tree(awareness_lock.doc()) {
                    Ok(tree) => {
                        println!("有数据 反向同步数据 并创建 编辑器");
                        //转换成功
                        let pool =
                            NodePool::new(Arc::new(tree)).as_ref().clone();
                        let options = options
                            .editor_options
                            .clone()
                            .set_content(Content::NodePool(pool));
                        ForgeAsyncRuntime::create(options).await?
                    },
                    Err(_) => {
                        println!(
                            "转换失败 创建一个本地编辑器 并同步数据到远程1"
                        );
                        //转换失败 创建一个本地编辑器 并同步数据到远程
                        let ed = ForgeAsyncRuntime::create(
                            options.editor_options.clone(),
                        )
                        .await?;
                        sync_manager.sync_to_remote(&ed).await?;
                        ed
                    },
                }
            },

            Ok(false) => {
                println!("没有数据 创建一个本地编辑器 并同步数据到远程2");
                //没有数据 创建一个本地编辑器 并同步数据到远程
                let ed =
                    ForgeAsyncRuntime::create(options.editor_options.clone())
                        .await?;
                sync_manager.sync_to_remote(&ed).await?;
                ed
            },
            _ => {
                return Err(ForgeError::Other(anyhow::anyhow!(
                    "服务器异常，请检查服务器是否启动"
                )));
            },
        };

        let editor_arc = Arc::new(RwLock::new(editor));

        let mut collab_editor = Self {
            editor: editor_arc.clone(),
            options,
            sync_manager: Arc::new(sync_manager),
        };

        // 启动同步管理器，传入编辑器引用
        if let Err(e) = Arc::get_mut(&mut collab_editor.sync_manager)
            .unwrap()
            .start_with_editor(editor_arc)
            .await
        {
            eprintln!("启动同步管理器失败: {}", e);
        }

        Ok(collab_editor)
    }

    pub fn get_options(&self) -> &CollabEditorOptions {
        &self.options
    }

    pub fn get_resource<T: Resource>(
        &self,
        rid: ResourceId,
    ) -> Option<Arc<T>> {
        // 注意：这里需要异步访问 editor，但为了保持接口兼容性，暂时 panic
        panic!("get_resource 需要异步访问，请使用 get_resource_async")
    }

    /// 异步获取资源
    pub async fn get_resource_async<T: Resource>(
        &self,
        rid: ResourceId,
    ) -> Option<Arc<T>> {
        // 获取编辑器状态
        let editor = self.editor.read().await;
        let state = editor.get_state();

        // 获取资源管理器
        let resource_manager = state.resource_manager();

        // 从资源表中获取指定类型和ID的资源
        resource_manager.resource_table.get::<T>(rid)
    }

    /// 异步获取编辑器的只读访问
    pub async fn get_editor(
        &self
    ) -> tokio::sync::RwLockReadGuard<'_, ForgeAsyncRuntime> {
        self.editor.read().await
    }

    /// 异步获取编辑器的可写访问
    pub async fn get_editor_mut(
        &self
    ) -> tokio::sync::RwLockWriteGuard<'_, ForgeAsyncRuntime> {
        self.editor.write().await
    }

    /// 异步获取历史管理器
    pub async fn get_history_manager_async(
        &self
    ) -> &HistoryManager<HistoryEntryWithMeta> {
        // 注意：这里仍然有生命周期问题，需要重新设计
        // 暂时返回一个错误提示
        panic!("需要重新设计异步访问模式")
    }

    /// 异步获取状态
    pub async fn get_state_async(&self) -> Arc<State> {
        let editor = self.editor.read().await;
        editor.get_state().clone()
    }

    /// 异步获取文档
    pub async fn doc_async(&self) -> Arc<NodePool> {
        let editor = self.editor.read().await;
        editor.doc()
    }
}

use mf_collab_client::yrs::{DeepObservable, Subscription};
use tokio::sync::mpsc;

#[derive(Debug, Clone, PartialEq)]
pub enum ChangeType {
    /// Determines a change that resulted in adding a consecutive number of new elements:
    /// - For [Array] it's a range of inserted elements.
    /// - For [XmlElement] it's a range of inserted child XML nodes.
    Added(Vec<Value>),
    Removed(u32),
    Retain(u32),
}
#[derive(Debug, Clone, PartialEq)]
pub enum EntryChangeType {
    /// Informs about a new value inserted under specified entry.
    Inserted(Value),

    /// Informs about a change of old value (1st field) to a new one (2nd field) under
    /// a corresponding entry.
    Updated(Value, Value),

    /// Informs about a removal of a corresponding entry - contains a removed value.
    Removed(Value),
}

/// 同步事件类型
#[derive(Debug, Clone)]
pub enum SyncEventType {
    /// 来自本地的事务
    ArrayChange(Vec<Value>, Vec<ChangeType>),
    /// yrs 深度变化事件
    MapChange(Vec<Value>, Vec<EntryChangeType>), // 简化为字节数组，避免复杂的类型问题
}

/// 协作同步管理器
pub struct CollabSyncManager {
    provider: WebsocketProvider,
    awareness: AwarenessRef,
    /// 事件发送器，用于处理同步事件
    event_sender: Option<mpsc::UnboundedSender<Vec<SyncEventType>>>,
}

impl CollabSyncManager {
    /// 创建协作同步管理器
    ///
    /// 初始化协作同步管理器，包括创建文档和意识
    ///
    /// # 参数
    ///
    /// * `options` - 协作编辑器选项
    pub async fn new(options: &CollabEditorOptions) -> ForgeResult<Self> {
        let doc = Doc::new();
        let awareness: AwarenessRef =
            Arc::new(tokio::sync::RwLock::new(Awareness::new(doc)));

        Ok(Self {
            provider: WebsocketProvider::new(
                options.server_url.to_string(),
                options.room_name.to_string(),
                awareness.clone(),
            )
            .await,
            awareness,
            event_sender: None,
        })
    }

    /// 启动同步管理器，设置监听器和事件处理
    pub async fn start_with_editor(
        &mut self,
        editor: Arc<RwLock<ForgeAsyncRuntime>>,
    ) -> ForgeResult<()> {
        // 连接到服务器
        self.provider.connect().await;

        // 创建事件通道
        let (sender, mut receiver) = mpsc::unbounded_channel();
        self.event_sender = Some(sender.clone());

        // 设置 yrs 深度监听器
        self.setup_yrs_listener(sender.clone()).await?;

        // 启动事件处理循环
        let awareness_clone = self.awareness.clone();
        let editor_clone = editor.clone();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                if let Err(e) = Self::handle_sync_event(
                    event,
                    editor_clone.clone(),
                    awareness_clone.clone(),
                )
                .await
                {
                    eprintln!("处理同步事件时出错: {}", e);
                }
            }
        });

        println!("协作同步管理器已启动");
        Ok(())
    }

    /// 设置 yrs nodes 监听器
    /// 专门监听 nodes 相关的变化，而不是整个文档
    async fn setup_yrs_listener(
        &mut self,
        sender: mpsc::UnboundedSender<Vec<SyncEventType>>,
    ) -> ForgeResult<()> {
        let awareness_lock = self.awareness.read().await;
        let doc = awareness_lock.doc();

        // 获取或创建 nodes 映射
        let nodes_map = doc.get_or_insert_map("nodes");

        // 使用 observe_deep 监听 nodes 映射的深度变化
        let sender_clone = sender.clone();
        self.provider.subscription(nodes_map.observe_deep(move |_txn, events| {
            let sender = sender_clone.clone();
            let mut event_vec = Vec::new();
            // 检查是否有事件（使用 iter() 方法）
            let event_count = events.iter().count();
            for event in events.iter() {
                //path 转换成数组
                match event {
                    mf_collab_client::yrs::types::Event::Array(array_event) => {
                        let path = array_event.path().iter().map(|path| serde_json::to_value(path).unwrap()).collect::<Vec<serde_json::Value>>();
                        array_event.delta(_txn).iter().for_each(|delta| {
                            // 将 delta 转换为 ChangeType
                            let change_type = match delta {
                                Change::Added(values) => ChangeType::Added(values.iter()
                                .filter_map(|value| {
                                    if let mf_collab_client::yrs::Value::Any(ref any) = value {
                                        Utils::yrs_any_to_json_value(any)
                                    } else {
                                        None
                                    }
                                })
                                .collect()),
                                Change::Removed(count) => ChangeType::Removed(*count),
                                Change::Retain(count) => ChangeType::Retain(*count),
                            };
                            event_vec.push(SyncEventType::ArrayChange(path.clone(), vec![change_type]));
                        });
                    }
                    mf_collab_client::yrs::types::Event::Map(map_event) => {
                        map_event.keys(_txn).iter().for_each(|(key, value)| {
                            let path = vec![serde_json::to_value(key).unwrap()];
                            let change_type = match value {
                                EntryChange::Inserted(value) => {
                                    if let mf_collab_client::yrs::Value::Any(ref any) = value {
                                        EntryChangeType::Inserted(Utils::yrs_any_to_json_value(any).unwrap())
                                    } else {
                                        // 处理非 Any 类型（可选：用默认值或跳过）
                                        return;
                                    }
                                }
                                EntryChange::Updated(value, value1) => {
                                    if let (mf_collab_client::yrs::Value::Any(ref any0), mf_collab_client::yrs::Value::Any(ref any1)) = (value, value1) {
                                        EntryChangeType::Updated(
                                            Utils::yrs_any_to_json_value(any0).unwrap(),
                                            Utils::yrs_any_to_json_value(any1).unwrap()
                                        )
                                    } else {
                                        return;
                                    }
                                }
                                EntryChange::Removed(value) => {
                                    if let mf_collab_client::yrs::Value::Any(ref any) = value {
                                        EntryChangeType::Removed(Utils::yrs_any_to_json_value(any).unwrap())
                                    } else {
                                        return;
                                    }
                                }
                            };
                            event_vec.push(SyncEventType::MapChange(path, vec![change_type]));
                        });
                    }
                    _ => {
                    }
                }
            }
            if event_vec.len() > 0 {
                println!("检测到 nodes 深度变化: {} 个事件", event_count);

                // 简化事件处理，只发送事件计数
                if let Err(e) = sender.send(event_vec) {
                    eprintln!("发送 nodes 深度变化事件失败: {}", e);
                }
            }
        }));

        println!("yrs nodes 监听器已设置");
        Ok(())
    }

    /// 处理同步事件
    async fn handle_sync_event(
        events: Vec<SyncEventType>,
        _editor: Arc<RwLock<ForgeAsyncRuntime>>,
        _awareness: AwarenessRef,
    ) -> ForgeResult<()> {
        for event in events {
            match event {
                SyncEventType::ArrayChange(_path, changes) => {
                    // path 数组第一个对应的 是 节点id
                    for change in changes {
                        match change {
                            ChangeType::Added(values) => {
                                println!("处理 Added 事件: {:?}", values);
                                // 转换成mark
                            },
                            ChangeType::Removed(index) => {
                                println!("处理 Removed 事件: {:?}", index);
                            },
                            ChangeType::Retain(index) => {
                                println!("处理 Retain 事件: {:?}", index);
                            },
                        }
                    }
                },
                SyncEventType::MapChange(path, changes) => {
                    for change in changes {
                        match change {
                            EntryChangeType::Inserted(value) => {
                                println!("处理 Inserted 事件: {:?}", value);
                            },
                            EntryChangeType::Updated(value, value1) => {
                                println!("处理 Updated 事件: {:?}", value);
                            },
                            EntryChangeType::Removed(value) => {
                                println!("处理 Removed 事件: {:?}", value);
                            },
                        }
                    }
                },
            }
        }

        Ok(())
    }

    /// 同步数据到远程
    ///
    /// 将编辑器中的数据同步到远程服务器
    ///
    /// # 参数
    ///
    /// * `editor` - 编辑器实例
    pub async fn sync_to_remote(
        &self,
        editor: &ForgeAsyncRuntime,
    ) -> ForgeResult<()> {
        let doc = editor.get_state().doc();
        let tree = doc.get_inner().as_ref();
        Utils::apply_tree_to_yrs(self.awareness.clone(), tree).await?;
        Ok(())
    }

    pub async fn start(&mut self) {
        self.provider.connect().await;
    }
}
