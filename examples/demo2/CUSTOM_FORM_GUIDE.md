# 自定义表单实现指南

## 概述

CostTable 组件现在支持多种自定义表单的实现方式，以满足不同子模块的需求。本指南将详细说明如何实现和使用自定义表单。

## 1. 树形数据默认展开

### 配置选项

```vue
<CostTable
  :data="treeData"
  :columns="columns"
  :default-expand-all="true"
  :default-expand-level="2"
/>
```

### 属性说明

- `defaultExpandAll`: Boolean - 是否默认展开所有节点
- `defaultExpandLevel`: Number - 默认展开到第几层（-1 表示不限制）

### 使用示例

```javascript
// 展开所有节点
<CostTable :default-expand-all="true" />

// 只展开前两层
<CostTable :default-expand-level="2" />

// 不自动展开（默认行为）
<CostTable :default-expand-level="-1" />
```

## 2. 自定义表单实现方式

### 方式一：自定义组件属性

```vue
<CostTable
  :custom-form-component="MyCustomForm"
  :form-props="{ tableType: 'estimate', extraData: someData }"
/>
```

### 方式二：插槽方式

```vue
<CostTable>
  <template #form="{ record, data, tableType, onSubmit, onCancel }">
    <MyCustomForm
      :record="record"
      :data="data"
      :table-type="tableType"
      @submit="onSubmit"
      @cancel="onCancel"
    />
  </template>
</CostTable>
```

### 方式三：内联组件定义

```javascript
const EstimateFormComponent = {
  props: ['value', 'record', 'tableType'],
  emits: ['update:value', 'submit', 'cancel'],
  template: `
    <div class="custom-form">
      <!-- 表单内容 -->
    </div>
  `,
  setup(props, { emit }) {
    // 组件逻辑
    return {
      // 返回的数据和方法
    }
  }
}
```

## 3. 自定义表单组件规范

### 必需的 Props

```javascript
props: {
  value: Object,        // 表单数据
  record: Object,       // 当前编辑的记录
  tableType: String,    // 表格类型（budget, estimate, settlement, measures）
}
```

### 必需的 Emits

```javascript
emits: [
  'update:value',  // 更新表单数据
  'submit',        // 提交表单
  'cancel'         // 取消编辑
]
```

### 推荐的表单结构

```vue
<template>
  <div class="custom-form">
    <a-form layout="vertical">
      <!-- 基础信息 -->
      <a-row :gutter="16">
        <a-col :span="12">
          <a-form-item label="项目编码" :rules="[{ required: true }]">
            <a-input v-model:value="formData.code" />
          </a-form-item>
        </a-col>
        <a-col :span="12">
          <a-form-item label="项目名称" :rules="[{ required: true }]">
            <a-input v-model:value="formData.name" />
          </a-form-item>
        </a-col>
      </a-row>
      
      <!-- 数量和价格 -->
      <a-row :gutter="16">
        <a-col :span="8">
          <a-form-item label="数量">
            <a-input-number v-model:value="formData.quantity" :min="0" />
          </a-form-item>
        </a-col>
        <a-col :span="8">
          <a-form-item label="单价">
            <a-input-number v-model:value="formData.unitPrice" :min="0" :precision="2" />
          </a-form-item>
        </a-col>
        <a-col :span="8">
          <a-form-item label="金额">
            <a-input-number v-model:value="computedAmount" :precision="2" disabled />
          </a-form-item>
        </a-col>
      </a-row>
      
      <!-- 根据 tableType 显示不同字段 -->
      <template v-if="tableType === 'estimate'">
        <!-- 概算特有字段 -->
      </template>
      
      <template v-if="tableType === 'budget'">
        <!-- 预算特有字段 -->
      </template>
    </a-form>
  </div>
</template>
```

## 4. 不同子模块的表单实现

### 概算模块 (rough-estimate)

```javascript
const EstimateFormComponent = {
  props: ['value', 'record', 'tableType'],
  template: `
    <div class="estimate-form">
      <!-- 概算特有的表单字段 -->
      <a-form-item label="概算依据">
        <a-select v-model:value="formData.estimateBasis">
          <a-select-option value="quota">定额标准</a-select-option>
          <a-select-option value="market">市场价格</a-select-option>
        </a-select>
      </a-form-item>
    </div>
  `
}
```

### 预算模块 (budget)

```javascript
const BudgetFormComponent = {
  props: ['value', 'record', 'tableType'],
  template: `
    <div class="budget-form">
      <!-- 预算特有的表单字段 -->
      <a-form-item label="预算科目">
        <a-cascader v-model:value="formData.budgetCategory" :options="budgetOptions" />
      </a-form-item>
    </div>
  `
}
```

### 结算模块 (settlement)

```javascript
const SettlementFormComponent = {
  props: ['value', 'record', 'tableType'],
  template: `
    <div class="settlement-form">
      <!-- 结算特有的表单字段 -->
      <a-form-item label="实际完成量">
        <a-input-number v-model:value="formData.actualQuantity" />
      </a-form-item>
    </div>
  `
}
```

## 5. 最佳实践

### 避免过多插槽

虽然插槽提供了灵活性，但过多的插槽会导致：
- 组件接口复杂化
- 维护困难
- 性能问题

**推荐方案：**
1. 优先使用 `customFormComponent` 属性
2. 为常见场景提供预设组件
3. 只在特殊情况下使用插槽

### 表单组件复用

```javascript
// 创建基础表单组件
const BaseFormComponent = {
  props: ['value', 'record', 'tableType', 'extraFields'],
  template: `
    <div class="base-form">
      <!-- 通用字段 -->
      <CommonFields v-model="formData" />
      
      <!-- 扩展字段 -->
      <component 
        v-if="extraFields" 
        :is="extraFields" 
        v-model="formData" 
        :table-type="tableType"
      />
    </div>
  `
}

// 在不同模块中使用
<CostTable 
  :custom-form-component="BaseFormComponent"
  :form-props="{ extraFields: EstimateExtraFields }"
/>
```

### 表单验证

```javascript
const FormComponent = {
  setup(props, { emit }) {
    const formRef = ref()
    const rules = {
      code: [{ required: true, message: '请输入项目编码' }],
      name: [{ required: true, message: '请输入项目名称' }],
      quantity: [{ required: true, type: 'number', min: 0 }]
    }
    
    const handleSubmit = async () => {
      try {
        await formRef.value.validate()
        emit('submit', formData.value)
      } catch (error) {
        console.error('表单验证失败:', error)
      }
    }
    
    return { formRef, rules, handleSubmit }
  }
}
```

## 6. 性能优化

### 懒加载表单组件

```javascript
const LazyFormComponent = defineAsyncComponent(() => 
  import('./components/EstimateForm.vue')
)
```

### 表单数据缓存

```javascript
const useFormCache = () => {
  const cache = new Map()
  
  const getCachedForm = (recordId) => cache.get(recordId)
  const setCachedForm = (recordId, data) => cache.set(recordId, data)
  
  return { getCachedForm, setCachedForm }
}
```

## 7. 总结

通过以上方案，可以灵活地为不同子模块提供定制化的表单，同时保持代码的可维护性和性能。

**关键要点：**
1. 优先使用组件属性而非插槽
2. 建立表单组件规范
3. 合理复用基础组件
4. 注意性能优化
5. 保持接口简洁
