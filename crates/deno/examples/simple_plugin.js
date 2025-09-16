// ModuForge Deno 插件示例
// 这个插件实现了基本的事务处理功能

/**
 * 事务追加处理
 * @param {Object} args - 包含 transactionCount, oldStateVersion, newStateVersion
 * @returns {Object|null} - 返回新事务数据或 null
 */
function appendTransaction(args) {
    console.log('📝 appendTransaction called:', {
        transactionCount: args.transactionCount,
        oldVersion: args.oldStateVersion,
        newVersion: args.newStateVersion
    });

    // 只在有多个事务时才追加新事务
    if (args.transactionCount > 1) {
        const transactionId = ModuForge.Transaction.new();

        // 添加元数据
        ModuForge.Transaction.setMeta(transactionId, 'batchSize', args.transactionCount);
        ModuForge.Transaction.setMeta(transactionId, 'timestamp', Date.now());
        ModuForge.Transaction.setMeta(transactionId, 'plugin', 'simple-plugin');

        console.log('✅ Created batch transaction:', transactionId);
        return { transactionId };
    }

    return null;
}

/**
 * 事务过滤
 * @param {Object} args - 包含 transactionId, stateVersion
 * @returns {boolean} - 是否允许事务执行
 */
function filterTransaction(args) {
    console.log('🔍 filterTransaction called:', args);

    // 示例：拒绝在特定状态版本下的事务
    if (args.stateVersion % 10 === 0) {
        console.log('❌ Transaction rejected at milestone version:', args.stateVersion);
        return false;
    }

    console.log('✅ Transaction allowed');
    return true;
}

/**
 * 自定义插件方法：验证文档结构
 * @param {Object} args - 验证参数
 * @returns {Object} - 验证结果
 */
function validateDocument(args) {
    try {
        const docId = ModuForge.State.getDoc();
        const schema = JSON.parse(ModuForge.State.getSchema());

        console.log('🔍 Validating document:', docId);
        console.log('📄 Schema info:', {
            name: schema.name,
            nodeTypes: schema.nodeTypes.length,
            markTypes: schema.markTypes.length
        });

        return {
            valid: true,
            docId: docId,
            schemaName: schema.name,
            timestamp: Date.now()
        };
    } catch (error) {
        console.error('❌ Document validation failed:', error);
        return {
            valid: false,
            error: error.message,
            timestamp: Date.now()
        };
    }
}

/**
 * 自定义插件方法：获取节点统计信息
 * @param {Object} args - 包含要统计的节点范围
 * @returns {Object} - 统计结果
 */
function getNodeStats(args) {
    const startNodeId = args.startNodeId || 1;
    const endNodeId = args.endNodeId || 10;

    let foundNodes = 0;
    let totalChildren = 0;

    for (let nodeId = startNodeId; nodeId <= endNodeId; nodeId++) {
        if (ModuForge.Node.findById(nodeId)) {
            foundNodes++;

            // 获取子节点
            try {
                const children = JSON.parse(ModuForge.Node.getChildren(nodeId));
                totalChildren += children.length;
            } catch (e) {
                // 忽略解析错误
            }
        }
    }

    console.log('📊 Node statistics:', { foundNodes, totalChildren });

    return {
        range: { start: startNodeId, end: endNodeId },
        foundNodes,
        totalChildren,
        averageChildren: foundNodes > 0 ? totalChildren / foundNodes : 0,
        timestamp: Date.now()
    };
}

// 插件初始化
console.log('🚀 Simple ModuForge Deno plugin loaded successfully');
console.log('📋 Available methods: appendTransaction, filterTransaction, validateDocument, getNodeStats');

// 导出插件信息（用于调试）
globalThis.pluginInfo = {
    name: 'simple-plugin',
    version: '1.0.0',
    methods: ['appendTransaction', 'filterTransaction', 'validateDocument', 'getNodeStats'],
    loadedAt: Date.now()
};