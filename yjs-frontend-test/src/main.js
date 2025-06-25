import * as Y from 'yjs'

// 状态管理
class AppState {
    constructor() {
        this.ws = null
        this.ydoc = new Y.Doc()
        this.room = null
        this.isConnected = false
        this.localUpdates = [] // 记录本地和服务器的 Yjs 更新
        this.logs = []
        
        this.initYjs()
        this.initUI()
        
        // 初始化时更新一次显示
        this.updateDataDisplay()
    }

    // 初始化 Yjs 文档
    initYjs() {
        // 监听 Yjs 文档变化
        this.ydoc.on('update', (update, origin) => {
            // 如果更新不是来自服务器，就将其发送到服务器
            if (origin !== 'server') {
                this.sendYjsUpdate(update)
            }

            // --- 以下是日志和显示更新逻辑 ---

            const updateHex = Array.from(update).map(b => b.toString(16).padStart(2, '0')).join(' ');
            const parsedJson = this.parseYjsUpdateToJson(updateHex);
            
            const updateInfo = {
                timestamp: new Date().toISOString(),
                size: update.length,
                parsedUpdate: parsedJson,
                source: origin === 'server' ? 'server' : 'local', // 标记更新来源
                type: 'yjs_update'
            }
            
            // 记录更新
            this.localUpdates.push(updateInfo)
            if (this.localUpdates.length > 20) {
                this.localUpdates.shift(); // 保持数组大小
            }
            
            this.log(`📝 Yjs Doc Update (${updateInfo.source})`, { 
                updateSize: update.length,
                parsedJson: parsedJson
            })
            
            // 更新数据显示
            this.updateDataDisplay()
        })

        this.ydoc.on('destroy', () => {
            this.log('🗑️ Yjs 文档被销毁')
        })
    }
    
    // 获取更新摘要
    getUpdateSummary(update) {
        const bytes = Array.from(update);
        // 简单的更新类型检测
        if (bytes.length > 4) {
            const firstBytes = bytes.slice(0, 4);
            if (firstBytes.includes(0x01)) return 'array_operation';
            if (firstBytes.includes(0x02)) return 'map_operation';
            if (firstBytes.includes(0x03)) return 'text_operation';
        }
        return `binary_data_${bytes.length}bytes`;
    }
    
    // 将十六进制字符串转换为Uint8Array
    hexToUint8Array(hexString) {
        const hex = hexString.replace(/\s+/g, ''); // 移除空格
        const bytes = new Uint8Array(hex.length / 2);
        for (let i = 0; i < hex.length; i += 2) {
            bytes[i / 2] = parseInt(hex.substr(i, 2), 16);
        }
        return bytes;
    }
    
    // 解析Yjs更新为JSON格式
    parseYjsUpdateToJson(updateHex) {
        try {
            // 将十六进制转换为字节数组
            const updateBytes = this.hexToUint8Array(updateHex);
            
            // 创建临时文档来解析更新
            const tempDoc = new Y.Doc();
            
            // 获取更新前的状态
            const beforeState = {
                nodes: tempDoc.getMap('nodes').toJSON(),
                attributes: tempDoc.getMap('attributes').toJSON(),
            }
            
            // 应用更新
            Y.applyUpdate(tempDoc, updateBytes);
            
            // 获取更新后的状态
            const afterState = {
                nodes: tempDoc.getMap('nodes').toJSON(),
                attributes: tempDoc.getMap('attributes').toJSON(),
                add_node: null, // 我们将在这里存放解析出的特定数据
            }

            // 专门检查由 add_node_step 创建的数据
            const addNodeMap = tempDoc.getMap('add_node');
            if (addNodeMap && addNodeMap.size > 0) {
                const addNodeData = addNodeMap.toJSON();
                console.log("addNodeData", addNodeData);
                // 后端将 'child' 字段存储为JSON字符串，所以我们需要在这里解析它
                if (addNodeData.child && typeof addNodeData.child === 'string') {
                    try {
                        addNodeData.child = JSON.parse(addNodeData.child);
                    } catch (e) {
                        console.error("解析 add_node 'child' 数据失败:", e);
                    }
                }
                afterState.add_node = addNodeData;
            }
            
            // 分析通用变化
            const changes = this.detectChanges(beforeState.nodes, beforeState.attributes, afterState.nodes, afterState.attributes);
            
            return {
                success: true,
                updateSize: updateBytes.length,
                beforeState: beforeState,
                afterState: afterState,
                changes: changes,
                parseTime: new Date().toISOString()
            };
        } catch (error) {
            return {
                success: false,
                error: error.message,
                updateSize: updateHex.length / 2,
                parseTime: new Date().toISOString()
            };
        }
    }
    
    // 检测变化
    detectChanges(beforeNodes, beforeAttrs, afterNodes, afterAttrs) {
        const changes = [];
        
        // 检测节点变化
        const beforeNodeKeys = Object.keys(beforeNodes);
        const afterNodeKeys = Object.keys(afterNodes);

        if (beforeNodeKeys.length !== afterNodeKeys.length) {
            changes.push({
                type: 'nodes_count_changed',
                from: beforeNodeKeys.length,
                to: afterNodeKeys.length
            });
        }
        
        // 检测属性变化
        const beforeKeys = Object.keys(beforeAttrs);
        const afterKeys = Object.keys(afterAttrs);
        
        if (beforeKeys.length !== afterKeys.length) {
            changes.push({
                type: 'attributes_count_changed',
                from: beforeKeys.length,
                to: afterKeys.length
            });
        }
        
        // 检测具体属性变化
        afterKeys.forEach(key => {
            if (beforeAttrs[key] !== afterAttrs[key]) {
                changes.push({
                    type: 'attribute_changed',
                    key: key,
                    from: beforeAttrs[key],
                    to: afterAttrs[key]
                });
            }
        });
        
        return changes;
    }

    // 初始化 UI 事件
    initUI() {
        // 连接按钮
        document.getElementById('connect-btn').onclick = () => this.connect()
        document.getElementById('disconnect-btn').onclick = () => this.disconnect()
        
        // 数据操作按钮
        document.getElementById('add-node-btn').onclick = () => this.addNode()
        document.getElementById('update-attr-btn').onclick = () => this.updateAttribute()
        document.getElementById('clear-data-btn').onclick = () => this.clearData()
        document.getElementById('request-sync-btn').onclick = () => this.requestSync()
        document.getElementById('test-update-btn').onclick = () => this.testLocalUpdate()
        
        // 更新状态显示
        this.updateUI()
    }

    // 连接 WebSocket
    connect() {
        const roomId = document.getElementById('room-input').value.trim()
        if (!roomId) {
            alert('请输入房间ID')
            return
        }

        this.room = roomId
        this.log(`🔌 正在连接房间: ${roomId}`)

        // 连接到 Rust 后端 WebSocket 服务器
        this.ws = new WebSocket('ws://localhost:8080')
        
        this.ws.onopen = () => {
            this.log('✅ WebSocket 连接成功')
            this.updateWSStatus(true)
            
            // 1. 加入房间
            this.sendMessage({
                JoinRoom: { room_id: this.room }
            });

            // 2. 加入房间后，立即请求同步
            const stateVector = Y.encodeStateVector(this.ydoc);
            this.sendMessage({
                YrsSyncRequest: {
                    room_id: this.room,
                    state_vector: Array.from(stateVector)
                }
            });

            // 3. 设置观察者来监听细粒度的变化
            this.setupObservers();
        }

        this.ws.onmessage = (event) => {
            try {
                if (event.data instanceof ArrayBuffer) {
                    // 处理二进制数据 (Yjs updates)
                    this.handleBinaryMessage(new Uint8Array(event.data))
                } else if (event.data instanceof Blob) {
                    // 处理 Blob 数据，转换为 ArrayBuffer
                    event.data.arrayBuffer().then(buffer => {
                        this.handleBinaryMessage(new Uint8Array(buffer))
                    })
                } else if (typeof event.data === 'string') {
                    // 检查是否是 JSON 格式
                    const data = event.data.trim()
                    if (data.startsWith('{') || data.startsWith('[')) {
                        // 处理 JSON 消息
                        const message = JSON.parse(data)
                        this.handleMessage(message)
                    } else {
                        // 处理纯文本消息
                        this.log('📨 收到文本消息: ' + data)
                    }
                } else {
                    this.log('❓ 未知消息类型', { type: typeof event.data, data: event.data })
                }
            } catch (error) {
                this.log('❌ 消息处理错误', { error: error.message, data: event.data })
            }
        }

        this.ws.onclose = () => {
            this.log('❌ WebSocket 连接关闭')
            this.updateWSStatus(false)
            this.isConnected = false
            this.updateUI()
        }

        this.ws.onerror = (error) => {
            this.log('❌ WebSocket 错误', error)
        }
    }

    // 断开连接
    disconnect() {
        if (this.ws) {
            if (this.room) {
                this.sendMessage({
                    LeaveRoom: { room_id: this.room }
                })
            }
            this.ws.close()
            this.ws = null
        }
        this.room = null
        this.isConnected = false
        this.updateWSStatus(false)
        this.updateUI()
    }

    // 发送 JSON 消息
    sendMessage(message) {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify(message))
            this.log('📤 发送消息', message)
        }
    }

    // 发送 Yjs 更新
    sendYjsUpdate(update) {
        if (this.ws && this.ws.readyState === WebSocket.OPEN && this.room) {
            // 注意：Yjs的二进制更新需要被包装在一个JSON消息中发送
            const message = {
                "YrsUpdate": {
                    "room_id": this.room,
                    // 将 Uint8Array 转换为普通的数字数组以便JSON序列化
                    "update": Array.from(update) 
                }
            };
            this.sendMessage(message);
        }
    }

    // 处理服务器消息
    handleMessage(message) {
        this.log('📥 收到消息', message)

        if (message.YrsUpdate && message.YrsUpdate.update) {
            this.log('Received YrsUpdate over JSON, handling as binary...');
            // 将数字数组转换回 Uint8Array
            const updateBytes = new Uint8Array(message.YrsUpdate.update);
            this.handleBinaryMessage(updateBytes);
        } else if (message.Error) {
            this.log('❌ 服务器错误: ' + message.Error.message)
        } else if (message.Notification) {
            this.log('📢 服务器通知: ' + message.Notification.message)
        } else if (message.Pong) {
            this.log('🏓 收到 Pong')
        } else {
            this.log('📨 其他消息 (可能已过时)', message)
        }
    }

    // 处理二进制消息 (Yjs updates)
    handleBinaryMessage(data) {
        this.log('📥 正在处理二进制更新', { size: data.length });
        
        // 记录服务器更新
        const updateHex = Array.from(data).map(b => b.toString(16).padStart(2, '0')).join(' ');
        const parsedJson = this.parseYjsUpdateToJson(updateHex);
        const updateInfo = {
            timestamp: new Date().toISOString(),
            size: data.length,
            parsedUpdate: parsedJson,
            source: 'server', // 服务器推送的更新
            type: 'yjs_update'
        };
        this.localUpdates.push(updateInfo)
        if (this.localUpdates.length > 20) {
            this.localUpdates.shift();
        }

        // 应用来自服务器的更新，并标记来源为 'server'
        // 'update' 事件监听器会因此知道这个更新来自服务器，从而不会把它再发回去
        Y.applyUpdate(this.ydoc, data, 'server');
        
        // 收到第一个有效更新后，我们认为已连接并同步
        if (!this.isConnected) {
            this.isConnected = true;
            this.log('✅ 同步完成');
            this.updateUI();
        }

        this.updateYjsStatus(true)
    }

    // 设置Yjs观察者，以实现细粒度感知
    setupObservers() {
        const nodesMap = this.ydoc.getMap('nodes');
        nodesMap.observe(event => {
            event.changes.keys.forEach((change, key) => {
                if (change.action === 'add') {
                    this.log(`[Observer] ✨ 节点已添加: ${key}`);
                    
                    // 为新节点的属性也设置一个观察者
                    const newNode = nodesMap.get(key);
                    if (newNode) {
                        const attrsMap = newNode.get('attrs');
                        attrsMap.observe(attrEvent => {
                            attrEvent.changes.keys.forEach((attrChange, attrKey) => {
                                this.log(`[Observer] 🔧 节点 ${key} 的属性 '${attrKey}' 已更新`);
                            });
                        });
                    }
                } else if (change.action === 'delete') {
                    this.log(`[Observer] 🗑️ 节点已删除: ${key}`);
                }
            });
        });
        this.log('🔬 Yjs 观察者已设置');
    }

    // 添加节点 (模拟操作)
    addNode() {
        const nodesMap = this.ydoc.getMap('nodes');
        const nodeId = 'node_' + Date.now();
    
        const newNodeMap = new Y.Map();
        newNodeMap.set('type', 'text-client');
        
        const attrsMap = new Y.Map();
        attrsMap.set('created_at', new Date().toISOString());
        attrsMap.set('source', 'client');
        newNodeMap.set('attrs', attrsMap);
        
        const contentArr = new Y.Array();
        newNodeMap.set('content', contentArr);
    
        nodesMap.set(nodeId, newNodeMap);
    
        this.log('➕ 添加节点', { id: nodeId, data: newNodeMap.toJSON() });
        // ydoc 'update' event will automatically send the changes to the server.
    }

    // 更新属性 (模拟操作)
    updateAttribute() {
        const nodesMap = this.ydoc.getMap('nodes');
        const nodeKeys = Array.from(nodesMap.keys());
        
        if (nodeKeys.length === 0) {
            this.log('⚠️ 没有节点可更新属性');
            return;
        }
    
        // Pick the last node to update for demonstration
        const lastNodeKey = nodeKeys[nodeKeys.length - 1];
        const nodeToUpdate = nodesMap.get(lastNodeKey);
    
        if (nodeToUpdate) {
            const attrsMap = nodeToUpdate.get('attrs');
            const key = 'last_update_client';
            const value = new Date().toISOString();
            attrsMap.set(key, value);
            this.log(`🔄 更新节点 ${lastNodeKey} 的属性`, { key, value });
        } else {
            this.log(`⚠️ 未找到节点 ${lastNodeKey}`);
        }
    }

    // 清空数据
    clearData() {
        // 清空 Yjs 文档会触发 'update' 事件，自动同步到服务器
        this.ydoc.getMap('nodes').clear();
        this.ydoc.getMap('attributes').clear();
        
        // 清空本地日志
        this.localUpdates = [];
        this.updateDataDisplay();
        
        this.log('🗑️ 清空所有数据');
    }

    // 请求重新同步
    requestSync() {
        this.log('🔄 请求重新同步...');
        // 通过重新连接来获取最新状态
        this.disconnect();
        // 短暂延迟以确保旧连接已关闭
        setTimeout(() => this.connect(), 100);
    }

    // 发送心跳
    sendPing() {
        this.sendMessage({ Ping: {} })
    }
    
    // 测试本地更新（不依赖服务器连接）
    testLocalUpdate() {
        this.log('🧪 开始测试本地更新')
        
        // 模拟添加一个测试节点
        const nodesMap = this.ydoc.getMap('nodes');
        const nodeId = 'test_' + Date.now();
        
        const newNodeMap = new Y.Map();
        newNodeMap.set('type', 'test-client');
        
        const attrsMap = new Y.Map();
        attrsMap.set('created_at', new Date().toISOString());
        attrsMap.set('source', 'local_test');
        
        newNodeMap.set('attrs', attrsMap);
        const contentArr = new Y.Array();
        newNodeMap.set('content', contentArr);
    
        nodesMap.set(nodeId, newNodeMap);
        
        // 模拟更新属性
        attrsMap.set('test_timestamp', new Date().toISOString());
        attrsMap.set('test_counter', (attrsMap.get('test_counter') || 0) + 1);
        
        this.log('✅ 测试数据已添加', { node: newNodeMap.toJSON() });
        
        // 强制更新显示
        this.updateDataDisplay()
    }

    // 更新 UI 状态
    updateUI() {
        const connectBtn = document.getElementById('connect-btn')
        const disconnectBtn = document.getElementById('disconnect-btn')
        const addNodeBtn = document.getElementById('add-node-btn')
        const updateAttrBtn = document.getElementById('update-attr-btn')
        const requestSyncBtn = document.getElementById('request-sync-btn')
        const roomInput = document.getElementById('room-input')

        const isWsConnected = this.ws && this.ws.readyState === WebSocket.OPEN

        connectBtn.disabled = isWsConnected
        disconnectBtn.disabled = !isWsConnected
        addNodeBtn.disabled = !this.isConnected
        updateAttrBtn.disabled = !this.isConnected
        requestSyncBtn.disabled = !isWsConnected
        roomInput.disabled = isWsConnected
    }

    // 更新 WebSocket 状态指示器
    updateWSStatus(connected) {
        const indicator = document.getElementById('ws-status')
        if (connected) {
            indicator.classList.add('connected')
        } else {
            indicator.classList.remove('connected')
        }
    }

    // 更新 Yjs 状态指示器
    updateYjsStatus(synced) {
        const indicator = document.getElementById('yjs-status')
        if (synced) {
            indicator.classList.add('connected')
        } else {
            indicator.classList.remove('connected')
        }
    }

    // 格式化显示数据
    formatDataForDisplay(nodes, attributes) {
        let text = `更新时间: ${new Date().toLocaleString()}\n\n`;
        text += `--- Document State ---\n`;
        // 使用更具可读性的方式来展示整个文档状态
        text += JSON.stringify(nodes, null, 2);
        text += `\n\n`;
        return text;
    }
    
    // 格式化显示增量更新
    formatUpdatesForDisplay(updates) {
        let text = `Yjs 更新记录: count=${updates.length}\n`;
        
        // Yjs 更新字符串 (显示JSON内容)
        if (updates.length > 0) {
            const updateStrings = updates.map(update => {
                const parsed = update.parsedUpdate;
                let summary = `${update.source}:${update.size}b`;
                
                if (parsed && parsed.success) {
                    if (parsed.changes && parsed.changes.length > 0) {
                        const changeTypes = parsed.changes.map(c => c.type).join(',');
                        summary += ` changes:(${changeTypes})`;
                    }
                }
                
                return summary;
            });
            text += `History: ${updateStrings.join(' | ')}\n\n`;
            
            // 显示最新的更新详情 (JSON格式)
            if (updates.length > 0) {
                const latestUpdate = updates[updates.length - 1];
                text += `最新更新详情 (${latestUpdate.source}):\n`;
                if (latestUpdate.parsedUpdate) {
                    text += `解析后JSON: ${JSON.stringify(latestUpdate.parsedUpdate, null, 2)}\n`;
                }
            }
        }
        
        if (updates.length === 0) {
            text += `暂无更新`;
        }
        
        return text;
    }

    // 更新数据显示
    updateDataDisplay() {
        try {
            // 获取当前 Yjs 文档的最新状态
            const nodes = this.ydoc.getMap('nodes').toJSON() // 直接获取整个nodes map
            const attributes = this.ydoc.getMap('attributes').toJSON()
            
            // 格式化显示数据快照
            const snapshotElement = document.getElementById('snapshot-data')
            if (snapshotElement) {
                snapshotElement.textContent = this.formatDataForDisplay(nodes, attributes)
            }

            // 格式化显示增量更新
            const patchesElement = document.getElementById('patches-data')
            if (patchesElement) {
                patchesElement.textContent = this.formatUpdatesForDisplay(this.localUpdates)
            }
            
            // 更新统计信息
            this.updateStatsDisplay(Object.keys(nodes).length, Object.keys(attributes).length)
            
        } catch (error) {
            console.error('❌ updateDataDisplay 出错:', error)
            this.log('❌ 数据显示更新失败: ' + error.message)
        }
    }
    
    // 更新统计信息显示
    updateStatsDisplay(nodeCount, attrCount) {
        // 如果页面有统计显示元素，更新它们
        const statsElements = {
            nodes: document.getElementById('node-count'),
            attrs: document.getElementById('attr-count')
        }
        
        if (statsElements.nodes) {
            statsElements.nodes.textContent = nodeCount
        }
        if (statsElements.attrs) {
            statsElements.attrs.textContent = attrCount
        }
    }

    // 添加日志
    log(message, data = null) {
        const timestamp = new Date().toLocaleTimeString()
        const logEntry = `[${timestamp}] ${message}`
        
        if (data) {
            console.log(logEntry, data)
            this.logs.push({ timestamp, message, data })
        } else {
            console.log(logEntry)
            this.logs.push({ timestamp, message })
        }

        // 更新日志显示 (只显示最近的 20 条)
        const recentLogs = this.logs.slice(-20)
        const logElement = document.getElementById('log-data')
        logElement.textContent = recentLogs
            .map(log => {
                const dataStr = log.data ? ` - ${JSON.stringify(log.data)}` : ''
                return `[${log.timestamp}] ${log.message}${dataStr}`
            })
            .join('\n')
        
        // 自动滚动到底部
        logElement.scrollTop = logElement.scrollHeight
    }
}

// 启动应用
const app = new AppState()

// 定期发送心跳 (每 30 秒)
setInterval(() => {
    if (app.ws && app.ws.readyState === WebSocket.OPEN) {
        app.sendPing()
    }
}, 30000)

// 将 app 暴露到全局，便于调试
window.app = app

// 添加全局调试函数
window.parseHex = (hexString) => {
    return app.parseYjsUpdateToJson(hexString);
}

console.log('🚀 ModuForge Yjs 测试客户端已启动')
console.log('可以通过 window.app 访问应用实例')
console.log('可以通过 window.parseHex("十六进制字符串") 手动解析更新') 