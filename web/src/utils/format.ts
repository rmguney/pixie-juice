/** "1.2 MB" */
export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const units = ['KB', 'MB', 'GB'];
  let value = bytes / 1024;
  let unit = 0;
  // Compare what will be shown, so 1023.6 KB becomes "1.0 MB" rather than "1024 KB".
  while (Math.round(value) >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit++;
  }
  return `${value < 10 ? value.toFixed(1) : Math.round(value)} ${units[unit]}`;
}

/** Signed percentage change from `before` to `after`: "−60%", "+4.2%". */
export function formatChange(before: number, after: number): string {
  const change = before > 0 ? ((after - before) / before) * 100 : 0;
  const magnitude = Math.abs(change);
  if (magnitude < 0.05) return '0%';
  const digits = magnitude < 10 ? magnitude.toFixed(1) : Math.round(magnitude).toString();
  return `${change < 0 ? '−' : '+'}${digits}%`;
}
