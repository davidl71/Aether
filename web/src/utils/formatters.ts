export function formatCurrency(value: number) {
  return value.toLocaleString('en-US', {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: 2
  });
}

export function formatPercent(value: number) {
  return `${value.toFixed(2)}%`;
}
