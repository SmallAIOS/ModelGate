// Shared rendering helpers. Plain functions, no React.

export function humanBytes(n: number): string {
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let v = n;
  let i = 0;
  while (v >= 1024 && i + 1 < units.length) {
    v /= 1024;
    i += 1;
  }
  return i === 0 ? `${n} ${units[0]}` : `${v.toFixed(1)} ${units[i]}`;
}
