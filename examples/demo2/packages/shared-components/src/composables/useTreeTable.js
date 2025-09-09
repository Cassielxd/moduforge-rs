/**
 * 树形表格功能组合式函数
 * @description 提供树形数据展开/折叠、层级管理等功能
 */
import { computed } from 'vue'

export function useTreeTable(expandedRowKeys, treeProps = {}) {
  const {
    children = 'children',
    hasChildren: hasChildrenField = 'hasChildren'
  } = treeProps

  // 获取行的唯一标识
  const getRowKey = (record) => {
    return record.id || record.key || record.sequenceNbr
  }

  // 展开行
  const expandRow = (rowKey) => {
    if (!expandedRowKeys.value.includes(rowKey)) {
      expandedRowKeys.value.push(rowKey)
    }
  }

  // 折叠行
  const collapseRow = (rowKey) => {
    const index = expandedRowKeys.value.indexOf(rowKey)
    if (index > -1) {
      expandedRowKeys.value.splice(index, 1)
    }
  }

  // 切换展开/折叠状态
  const toggleExpand = (record) => {
    const rowKey = getRowKey(record)
    if (isRowExpanded(rowKey)) {
      collapseRow(rowKey)
    } else {
      expandRow(rowKey)
    }
  }

  // 判断行是否展开
  const isRowExpanded = (rowKey) => {
    return expandedRowKeys.value.includes(rowKey)
  }

  // 判断是否有子节点
  const hasChildren = (record) => {
    // 优先使用 hasChildren 字段
    if (record[hasChildrenField] !== undefined) {
      return record[hasChildrenField]
    }
    
    // 检查 children 数组
    const childrenData = record[children]
    return Array.isArray(childrenData) && childrenData.length > 0
  }

  // 获取树形层级
  const getTreeLevel = (record, allData, level = 0) => {
    // 如果记录中已经有层级信息
    if (record.level !== undefined) {
      return record.level
    }
    
    // 递归查找父级
    for (const item of allData) {
      const childrenData = item[children] || []
      if (childrenData.includes(record)) {
        return getTreeLevel(item, allData, level) + 1
      }
      
      // 递归查找更深层级
      const deepLevel = findInChildren(record, childrenData, level + 1)
      if (deepLevel !== -1) {
        return deepLevel
      }
    }
    
    return level
  }

  // 在子节点中查找
  const findInChildren = (target, childrenData, level) => {
    for (const child of childrenData) {
      if (child === target) {
        return level
      }
      
      const grandChildren = child[children] || []
      if (grandChildren.length > 0) {
        const deepLevel = findInChildren(target, grandChildren, level + 1)
        if (deepLevel !== -1) {
          return deepLevel
        }
      }
    }
    return -1
  }

  // 展开所有节点
  const expandAll = (data) => {
    const allKeys = []
    
    const collectKeys = (items) => {
      items.forEach(item => {
        if (hasChildren(item)) {
          allKeys.push(getRowKey(item))
          const childrenData = item[children] || []
          collectKeys(childrenData)
        }
      })
    }
    
    collectKeys(data)
    expandedRowKeys.value = allKeys
  }

  // 折叠所有节点
  const collapseAll = () => {
    expandedRowKeys.value = []
  }

  // 展开到指定层级
  const expandToLevel = (data, targetLevel) => {
    const keysToExpand = []
    
    const collectKeysToLevel = (items, currentLevel = 0) => {
      items.forEach(item => {
        if (hasChildren(item) && currentLevel < targetLevel) {
          keysToExpand.push(getRowKey(item))
          const childrenData = item[children] || []
          collectKeysToLevel(childrenData, currentLevel + 1)
        }
      })
    }
    
    collectKeysToLevel(data)
    expandedRowKeys.value = keysToExpand
  }

  // 获取所有后代节点
  const getDescendants = (record) => {
    const descendants = []
    const childrenData = record[children] || []
    
    childrenData.forEach(child => {
      descendants.push(child)
      descendants.push(...getDescendants(child))
    })
    
    return descendants
  }

  // 获取所有祖先节点
  const getAncestors = (record, allData) => {
    const ancestors = []
    
    const findParent = (target, items, currentAncestors = []) => {
      for (const item of items) {
        const childrenData = item[children] || []
        
        if (childrenData.includes(target)) {
          ancestors.unshift(...currentAncestors, item)
          return true
        }
        
        if (childrenData.length > 0) {
          if (findParent(target, childrenData, [...currentAncestors, item])) {
            return true
          }
        }
      }
      return false
    }
    
    findParent(record, allData)
    return ancestors
  }

  // 获取兄弟节点
  const getSiblings = (record, allData) => {
    const ancestors = getAncestors(record, allData)
    
    if (ancestors.length === 0) {
      // 根级节点
      return allData.filter(item => item !== record)
    }
    
    const parent = ancestors[ancestors.length - 1]
    const childrenData = parent[children] || []
    return childrenData.filter(item => item !== record)
  }

  // 判断是否为叶子节点
  const isLeafNode = (record) => {
    return !hasChildren(record)
  }

  // 判断是否为根节点
  const isRootNode = (record, allData) => {
    return getAncestors(record, allData).length === 0
  }

  // 获取下一个兄弟节点
  const getNextSibling = (record, allData) => {
    const siblings = getSiblings(record, allData)
    const ancestors = getAncestors(record, allData)
    
    let parentChildren
    if (ancestors.length === 0) {
      parentChildren = allData
    } else {
      const parent = ancestors[ancestors.length - 1]
      parentChildren = parent[children] || []
    }
    
    const currentIndex = parentChildren.indexOf(record)
    return currentIndex < parentChildren.length - 1 ? parentChildren[currentIndex + 1] : null
  }

  // 获取上一个兄弟节点
  const getPrevSibling = (record, allData) => {
    const ancestors = getAncestors(record, allData)
    
    let parentChildren
    if (ancestors.length === 0) {
      parentChildren = allData
    } else {
      const parent = ancestors[ancestors.length - 1]
      parentChildren = parent[children] || []
    }
    
    const currentIndex = parentChildren.indexOf(record)
    return currentIndex > 0 ? parentChildren[currentIndex - 1] : null
  }

  // 计算展开的节点数量
  const expandedCount = computed(() => expandedRowKeys.value.length)

  // 判断是否有展开的节点
  const hasExpandedNodes = computed(() => expandedRowKeys.value.length > 0)

  return {
    // 基本操作
    expandRow,
    collapseRow,
    toggleExpand,
    
    // 批量操作
    expandAll,
    collapseAll,
    expandToLevel,
    
    // 查询方法
    isRowExpanded,
    hasChildren,
    isLeafNode,
    isRootNode,
    getTreeLevel,
    
    // 关系查询
    getDescendants,
    getAncestors,
    getSiblings,
    getNextSibling,
    getPrevSibling,
    
    // 计算属性
    expandedCount,
    hasExpandedNodes,
    
    // 工具方法
    getRowKey
  }
}
