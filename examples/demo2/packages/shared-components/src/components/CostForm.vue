<template>
  <div class="cost-form-container">
    <div class="form-header">
      <h2>{{ formTitle }}</h2>
      <p>{{ formDescription }}</p>
    </div>

    <a-form
      ref="formRef"
      :model="formData"
      :rules="formRules"
      layout="vertical"
      @finish="handleSubmit"
      @finishFailed="handleSubmitFailed"
    >
      <!-- 基本信息 -->
      <a-card title="基本信息" class="form-section">
        <a-row :gutter="16">
          <a-col :span="12">
            <a-form-item label="项目名称" name="name">
              <a-input 
                v-model:value="formData.name" 
                placeholder="请输入项目名称"
                :disabled="!editable"
              />
            </a-form-item>
          </a-col>
          <a-col :span="12">
            <a-form-item label="项目类型" name="type">
              <a-select 
                v-model:value="formData.type" 
                placeholder="请选择项目类型"
                :disabled="!editable"
              >
                <a-select-option value="building">建筑工程</a-select-option>
                <a-select-option value="infrastructure">基础设施</a-select-option>
                <a-select-option value="renovation">装修工程</a-select-option>
                <a-select-option value="landscape">园林绿化</a-select-option>
                <a-select-option value="municipal">市政工程</a-select-option>
              </a-select>
            </a-form-item>
          </a-col>
        </a-row>

        <a-row :gutter="16">
          <a-col :span="8">
            <a-form-item label="预算金额" name="amount">
              <a-input-number
                v-model:value="formData.amount"
                :min="0"
                :precision="2"
                :formatter="value => `¥ ${value}`.replace(/\B(?=(\d{3})+(?!\d))/g, ',')"
                :parser="value => value.replace(/¥\s?|(,*)/g, '')"
                style="width: 100%"
                placeholder="请输入预算金额"
                :disabled="!editable"
              />
            </a-form-item>
          </a-col>
          <a-col :span="8">
            <a-form-item label="开始日期" name="startDate">
              <a-date-picker 
                v-model:value="formData.startDate" 
                style="width: 100%"
                :disabled="!editable"
              />
            </a-form-item>
          </a-col>
          <a-col :span="8">
            <a-form-item label="结束日期" name="endDate">
              <a-date-picker 
                v-model:value="formData.endDate" 
                style="width: 100%"
                :disabled="!editable"
              />
            </a-form-item>
          </a-col>
        </a-row>

        <a-row :gutter="16">
          <a-col :span="12">
            <a-form-item label="负责人" name="manager">
              <a-input 
                v-model:value="formData.manager" 
                placeholder="请输入负责人"
                :disabled="!editable"
              />
            </a-form-item>
          </a-col>
          <a-col :span="12">
            <a-form-item label="状态" name="status">
              <a-select 
                v-model:value="formData.status" 
                placeholder="请选择状态"
                :disabled="!editable"
              >
                <a-select-option value="draft">草稿</a-select-option>
                <a-select-option value="reviewing">审核中</a-select-option>
                <a-select-option value="approved">已批准</a-select-option>
                <a-select-option value="rejected">已拒绝</a-select-option>
              </a-select>
            </a-form-item>
          </a-col>
        </a-row>

        <a-form-item label="项目描述" name="description">
          <a-textarea
            v-model:value="formData.description"
            :rows="4"
            placeholder="请输入项目详细描述"
            :disabled="!editable"
          />
        </a-form-item>
      </a-card>

      <!-- 费用明细 -->
      <a-card title="费用明细" class="form-section">
        <div class="cost-items-header">
          <a-space>
            <a-button 
              type="dashed" 
              @click="addCostItem"
              :disabled="!editable"
            >
              <template #icon><PlusOutlined /></template>
              添加费用项目
            </a-button>
            <a-button 
              @click="importCostItems"
              :disabled="!editable"
            >
              <template #icon><ImportOutlined /></template>
              导入模板
            </a-button>
          </a-space>
        </div>

        <a-table
          :columns="costColumns"
          :data-source="formData.costItems"
          :pagination="false"
          size="small"
          bordered
          class="cost-items-table"
        >
          <template #bodyCell="{ column, record, index }">
            <template v-if="column.key === 'name'">
              <a-input 
                v-model:value="record.name" 
                placeholder="费用项目"
                :disabled="!editable"
                @change="calculateTotal"
              />
            </template>
            <template v-else-if="column.key === 'unit'">
              <a-input 
                v-model:value="record.unit" 
                placeholder="单位"
                :disabled="!editable"
              />
            </template>
            <template v-else-if="column.key === 'quantity'">
              <a-input-number
                v-model:value="record.quantity"
                :min="0"
                :precision="2"
                style="width: 100%"
                placeholder="数量"
                :disabled="!editable"
                @change="calculateTotal"
              />
            </template>
            <template v-else-if="column.key === 'unitPrice'">
              <a-input-number
                v-model:value="record.unitPrice"
                :min="0"
                :precision="2"
                style="width: 100%"
                placeholder="单价"
                :disabled="!editable"
                @change="calculateTotal"
              />
            </template>
            <template v-else-if="column.key === 'amount'">
              <span class="amount-display">
                ¥{{ formatAmount(record.quantity * record.unitPrice) }}
              </span>
            </template>
            <template v-else-if="column.key === 'action'">
              <a-button 
                type="link" 
                danger 
                size="small"
                @click="removeCostItem(index)"
                :disabled="!editable"
              >
                删除
              </a-button>
            </template>
          </template>
        </a-table>
        
        <div class="total-amount">
          <strong>总计：¥{{ formatAmount(totalAmount) }}</strong>
        </div>
      </a-card>

      <!-- 操作按钮 -->
      <div class="form-actions">
        <a-space>
          <a-button @click="handleCancel">
            {{ editable ? '取消' : '关闭' }}
          </a-button>
          <a-button 
            v-if="editable" 
            @click="handleSaveDraft"
            :loading="saving"
          >
            保存草稿
          </a-button>
          <a-button 
            v-if="editable"
            type="primary" 
            html-type="submit" 
            :loading="saving"
          >
            {{ mode === 'create' ? '创建' : '保存' }}
          </a-button>
        </a-space>
      </div>
    </a-form>
  </div>
</template>

<script setup>
import { ref, reactive, computed, watch, onMounted } from 'vue'
import { message } from 'ant-design-vue'
import { PlusOutlined, ImportOutlined } from '@ant-design/icons-vue'
import dayjs from 'dayjs'

const props = defineProps({
  mode: {
    type: String,
    default: 'create', // create, edit, view
    validator: (value) => ['create', 'edit', 'view'].includes(value)
  },
  initialData: {
    type: Object,
    default: () => ({})
  },
  formType: {
    type: String,
    default: 'estimate' // estimate, budget, settlement
  }
})

const emit = defineEmits(['submit', 'cancel', 'saveDraft'])

const formRef = ref()
const saving = ref(false)

const editable = computed(() => props.mode !== 'view')

const formTitle = computed(() => {
  const titles = {
    create: '新建概算',
    edit: '编辑概算', 
    view: '查看概算'
  }
  return titles[props.mode] || '概算表单'
})

const formDescription = computed(() => {
  const descriptions = {
    create: '请填写项目基本信息和费用明细112',
    edit: '修改项目信息和费用明细',
    view: '查看项目详细信息'
  }
  return descriptions[props.mode] || ''
})

// 表单数据
const formData = reactive({
  name: '',
  type: '',
  amount: null,
  startDate: null,
  endDate: null,
  manager: '',
  status: 'draft',
  description: '',
  costItems: [
    { name: '材料费', unit: '项', quantity: 1, unitPrice: 0 },
    { name: '人工费', unit: '项', quantity: 1, unitPrice: 0 },
    { name: '机械费', unit: '项', quantity: 1, unitPrice: 0 }
  ]
})

// 表单验证规则
const formRules = {
  name: [{ required: true, message: '请输入项目名称' }],
  type: [{ required: true, message: '请选择项目类型' }],
  amount: [{ required: true, message: '请输入预算金额' }],
  manager: [{ required: true, message: '请输入负责人' }]
}

// 费用明细表格列配置
const costColumns = [
  { title: '费用项目', key: 'name', width: '25%' },
  { title: '单位', key: 'unit', width: '10%' },
  { title: '数量', key: 'quantity', width: '15%' },
  { title: '单价（元）', key: 'unitPrice', width: '15%' },
  { title: '金额（元）', key: 'amount', width: '20%' },
  { title: '操作', key: 'action', width: '15%' }
]

// 计算总金额
const totalAmount = computed(() => {
  return formData.costItems.reduce((sum, item) => {
    return sum + (item.quantity || 0) * (item.unitPrice || 0)
  }, 0)
})

// 方法
const formatAmount = (amount) => {
  if (!amount) return '0.00'
  return Number(amount).toLocaleString('zh-CN', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2
  })
}

const addCostItem = () => {
  formData.costItems.push({
    name: '',
    unit: '项',
    quantity: 1,
    unitPrice: 0
  })
}

const removeCostItem = (index) => {
  if (formData.costItems.length > 1) {
    formData.costItems.splice(index, 1)
    calculateTotal()
  } else {
    message.warning('至少保留一个费用项目')
  }
}

const importCostItems = () => {
  message.info('导入模板功能开发中...')
}

const calculateTotal = () => {
  // 更新总金额到表单数据
  formData.amount = totalAmount.value
}

const handleSubmit = async (values) => {
  try {
    saving.value = true
    
    const submitData = {
      ...values,
      costItems: formData.costItems,
      totalAmount: totalAmount.value,
      startDate: values.startDate ? dayjs(values.startDate).format('YYYY-MM-DD') : null,
      endDate: values.endDate ? dayjs(values.endDate).format('YYYY-MM-DD') : null
    }
    
    emit('submit', submitData)
    message.success(`${props.mode === 'create' ? '创建' : '保存'}成功！`)
    
  } catch (error) {
    console.error('提交失败:', error)
    message.error('操作失败，请重试')
  } finally {
    saving.value = false
  }
}

const handleSubmitFailed = (errorInfo) => {
  console.log('表单验证失败:', errorInfo)
  message.error('请检查表单填写')
}

const handleSaveDraft = async () => {
  try {
    saving.value = true
    
    const draftData = {
      ...formData,
      status: 'draft',
      totalAmount: totalAmount.value
    }
    
    emit('saveDraft', draftData)
    message.success('草稿保存成功！')
    
  } catch (error) {
    console.error('保存草稿失败:', error)
    message.error('保存失败，请重试')
  } finally {
    saving.value = false
  }
}

const handleCancel = () => {
  emit('cancel')
}

// 初始化数据
const initFormData = () => {
  if (props.initialData && Object.keys(props.initialData).length > 0) {
    Object.assign(formData, {
      ...props.initialData,
      startDate: props.initialData.startDate ? dayjs(props.initialData.startDate) : null,
      endDate: props.initialData.endDate ? dayjs(props.initialData.endDate) : null,
      costItems: props.initialData.costItems || formData.costItems
    })
  }
}

// 监听初始数据变化
watch(() => props.initialData, initFormData, { immediate: true, deep: true })

onMounted(() => {
  initFormData()
})

defineExpose({
  getFormData: () => formData,
  validateForm: () => formRef.value?.validate(),
  resetForm: () => formRef.value?.resetFields()
})
</script>

<style scoped>
.cost-form-container {
  padding: 24px;
  background: #fff;
  min-height: 100vh;
}

.form-header {
  text-align: center;
  margin-bottom: 32px;
  padding-bottom: 16px;
  border-bottom: 1px solid #f0f0f0;
}

.form-header h2 {
  margin-bottom: 8px;
  color: #1890ff;
  font-size: 24px;
}

.form-section {
  margin-bottom: 24px;
}

.cost-items-header {
  margin-bottom: 16px;
}

.cost-items-table {
  margin-bottom: 16px;
}

.amount-display {
  font-weight: 600;
  color: #1890ff;
}

.total-amount {
  text-align: right;
  font-size: 16px;
  color: #1890ff;
  padding: 16px 0;
  border-top: 1px solid #f0f0f0;
}

.form-actions {
  display: flex;
  justify-content: center;
  margin-top: 32px;
  padding-top: 24px;
  border-top: 1px solid #f0f0f0;
}

:deep(.ant-card-head) {
  background: #fafafa;
}

:deep(.ant-form-item-label > label) {
  font-weight: 600;
}
</style>
