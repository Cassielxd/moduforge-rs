import { computed } from 'vue'

export function useCostCalculation() {
  
  const calculateTotal = (data) => {
    if (!Array.isArray(data)) return 0
    
    return data.reduce((total, item) => {
      const amount = parseFloat(item.amount || item.total || 0)
      return total + amount
    }, 0).toFixed(2)
  }

  const calculateSelected = (selectedData) => {
    if (!Array.isArray(selectedData)) return 0
    
    return selectedData.reduce((total, item) => {
      const amount = parseFloat(item.amount || item.total || 0)
      return total + amount
    }, 0).toFixed(2)
  }

  const calculateRowTotal = (row) => {
    const quantity = parseFloat(row.quantity || 0)
    const unitPrice = parseFloat(row.unitPrice || 0)
    return (quantity * unitPrice).toFixed(2)
  }

  const formatCurrency = (amount) => {
    return new Intl.NumberFormat('zh-CN', {
      style: 'currency',
      currency: 'CNY'
    }).format(amount)
  }

  return {
    calculateTotal,
    calculateSelected,
    calculateRowTotal,
    formatCurrency
  }
}
