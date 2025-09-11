/*
 * 概算模块操作按钮配置
 * 参考原框架 operate.js 的数据结构
 */
import { ref } from 'vue'

const operateList = ref([
  // 新建概算
  {
    label: '新建',
    name: 'create',
    levelType: [1, 2, 3],
    windows: ['childPage', 'parentPage'],
    iconType: 'icon-cs-charu',
    iconStyle: {
      width: '28px',
      position: 'relative',
      left: '-4px',
    },
    components: ['estimate'],
    showProjectType: ['estimate'],
  },
  
  // 编辑概算
  {
    label: '移除',
    name: 'edit',
    levelType: [1, 2, 3],
    windows: ['childPage', 'parentPage'],
    iconType: 'icon-cs-shanchu', // 使用删除图标，名称改为移除
    components: ['estimate'],
    showProjectType: ['estimate'],
  },
  
  // 删除概算
  {
    label: '删除',
    name: 'delete',
    levelType: [1, 2, 3],
    windows: ['childPage', 'parentPage'],
    iconType: 'icon-cs-shanchu',
    components: ['estimate'],
    showProjectType: ['estimate'],
  },

  // 保存
  {
    label: '保存',
    name: 'save',
    windows: ['childPage', 'parentPage'],
    levelType: [1, 2, 3],
    iconType: 'icon-cs-baocun',
    components: ['estimate'],
  },

  // 导出表格 - 使用弹窗
  {
    label: '导出报表',
    name: 'export-table',
    windows: ['childPage', 'parentPage'],
    levelType: [1, 2, 3],
    showProjectType: ['estimate'],
    iconType: 'icon-cs-daochubaobiao',
    components: ['estimate'],
    useModal: true, // 标记使用弹窗
  },

  // 数据导入 - 新增弹窗操作
  {
    label: '数据导入',
    name: 'import-data',
    windows: ['childPage', 'parentPage'],
    levelType: [1, 2, 3],
    showProjectType: ['estimate'],
    iconType: 'icon-cs-daoru',
    components: ['estimate'],
    useModal: true,
  },

  // 统一应用
  {
    label: '统一应用',
    name: 'batch-operation',
    windows: ['parentPage'],
    type: 'menuList',
    menuList: [
      'batch-approve',    // 批量审批
      'batch-export',     // 批量导出
      'batch-delete',     // 批量删除
    ],
    levelType: [1, 2, 3],
    infoDec: '统一应用批量处理概算项目',
    iconType: 'icon-cs-tongyiyingyong', // 使用统一应用图标
    components: ['estimate'],
    iconStyle: {
      width: '28px',
      position: 'relative',
    },
    hidden: true,
  },

  // 项目自检
  {
    label: '项目自检',
    name: 'batch-approve',
    windows: ['parentPage'],
    levelType: [3],
    iconType: 'icon-xiangmuzijian', // 使用项目自检图标
    components: ['estimate'],
    showProjectType: ['estimate'],
  },

  // 批量导出
  {
    label: '批量导出',
    name: 'batch-export',
    windows: ['parentPage'],
    levelType: [3],
    iconType: 'icon-cs-daochubaobiao',
    components: ['estimate'],
    showProjectType: ['estimate'],
  },

  // 批量删除
  {
    label: '批量删除',
    name: 'batch-delete',
    windows: ['parentPage'],
    levelType: [3],
    iconType: 'icon-cs-shanchu',
    components: ['estimate'],
    showProjectType: ['estimate'],
  },

  // 项目检查
  {
    label: '项目检查',
    name: 'approval-status',
    type: 'selectRadio',
    windows: ['childPage', 'parentPage'],
    levelType: [3],
    value: 'all',
    options: [
      {
        name: '全部',
        kind: 'all',
        isValid: true,
      },
      {
        name: '草稿',
        kind: 'draft',
        isValid: true,
      },
      {
        name: '审核中',
        kind: 'reviewing',
        isValid: true,
      },
      {
        name: '已批准',
        kind: 'approved',
        isValid: true,
      },
      {
        name: '已拒绝',
        kind: 'rejected',
        isValid: true,
      },
    ],
    iconType: 'icon-xiangmuzijian', // 使用项目自检图标
    components: ['estimate'],
    showProjectType: ['estimate'],
  },

  // 项目类型过滤
  {
    label: '项目类型',
    name: 'project-type-filter',
    type: 'selectCheck',
    options: [
      {
        name: '建筑工程',
        kind: 'building',
        isValid: true,
      },
      {
        name: '基础设施',
        kind: 'infrastructure',
        isValid: true,
      },
      {
        name: '装修工程',
        kind: 'renovation',
        isValid: true,
      },
      {
        name: '安装工程',
        kind: 'installation',
        isValid: true,
      },
    ],
    windows: ['childPage', 'parentPage'],
    levelType: [1, 2, 3],
    iconType: 'icon-guolv',
    components: ['estimate'],
    showProjectType: ['estimate'],
    iconStyle: {
      fontSize: '19px',
      padding: '3px',
    },
  },

  // 导入数据
  {
    label: '导入数据',
    name: 'import-data',
    windows: ['childPage', 'parentPage'],
    levelType: [1, 2, 3],
    iconType: 'icon-daoruexcel',
    showProjectType: ['estimate'],
    components: ['estimate'],
    iconStyle: {
      fontSize: '16px',
      padding: '5px',
    },
  },

  // 保存
  {
    label: '保存数据',
    name: 'refresh',
    windows: ['childPage', 'parentPage'],
    levelType: [1, 2, 3],
    iconType: 'icon-cs-baocun', // 使用保存图标
    components: ['estimate'],
    infoDec: '保存概算数据',
  },

  // 查看详情
  {
    label: '查看详情',
    name: 'view-detail',
    windows: ['childPage', 'parentPage'],
    levelType: [3],
    iconType: 'icon-chakanguanlian',
    components: ['estimate'],
    showProjectType: ['estimate'],
  },

  // 复用组价
  {
    label: '复用组价',
    name: 'copy-project',
    windows: ['childPage', 'parentPage'],
    levelType: [3],
    iconType: 'icon-fuyongzujia', // 使用复用组价图标
    components: ['estimate'],
    showProjectType: ['estimate'],
  },

  // 载入模板
  {
    label: '载入模板',
    name: 'project-template',
    windows: ['parentPage'],
    type: 'select',
    options: [
      {
        name: '保存为模板',
        kind: 'save-template',
        isValid: true,
      },
      {
        name: '从模板创建',
        kind: 'create-from-template',
        isValid: true,
      },
      {
        name: '模板管理',
        kind: 'template-manage',
        isValid: true,
      },
    ],
    levelType: [1, 2, 3],
    iconType: 'icon-cs-zairumoban', // 使用载入模板图标
    components: ['estimate'],
    showProjectType: ['estimate'],
  },

  // 计税方式
  {
    label: '计税方式',
    name: 'tax-settings',
    windows: ['parentPage'],
    levelType: [1],
    iconType: 'icon-cs-jishuifangshi', // 使用计税方式图标
    components: ['estimate'],
    infoDec: '概算计税方式设置',
    useModal: true,
  },

  // 数据分析 - 弹窗操作
  {
    label: '数据分析',
    name: 'data-analysis',
    windows: ['parentPage'],
    levelType: [1, 2, 3],
    iconType: 'icon-cs-fenxi',
    components: ['estimate'],
    useModal: true,
  },

  // 模板管理 - 弹窗操作
  {
    label: '模板管理',
    name: 'template-manage',
    windows: ['parentPage'],
    levelType: [1, 2, 3],
    iconType: 'icon-cs-moban',
    components: ['estimate'],
    useModal: true,
  }
])

/**
 * 根据名称更新操作项
 */
export const updateOperateByName = (name, callback) => {
  const info = operateList.value.find(item => item.name === name)
  if (info) {
    callback(info)
  }
}

/**
 * 菜单聚合处理
 */
export const menuPolymerizeHandler = () => {
  const clientWidth = document.body.clientWidth
  const menuListOperate = operateList.value.filter(item => item.type === 'menuList')
  
  const isMenuPolymerize = name => {
    const map = {
      'batch-operation': 1200, // 批量操作在1200px以下聚合
    }
    return clientWidth < (map[name] || 1200)
  }
  
  for (let operate of menuListOperate) {
    operate.hidden = !isMenuPolymerize(operate.name)
    operate.menuInfoList = getMenuList(operate)
  }
  
  for (let operate of menuListOperate) {
    operate.menuInfoList.forEach(item => {
      const info = operateList.value.find(sub => sub.name === item.name)
      if (info) info.isPolymerizeShow = isMenuPolymerize(operate.name)
    })
  }
}

/**
 * 根据menuList生成对应的操作按钮列表
 */
const getMenuList = item => {
  const list = []
  for (let menu of item.menuList || []) {
    if (typeof menu === 'string') {
      const operate = operateList.value.find(opt => opt.name === menu)
      if (operate) list.push(operate)
    } else {
      list.push(menu)
    }
  }
  return list
}

export default operateList