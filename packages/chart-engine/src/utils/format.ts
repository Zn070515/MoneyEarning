export function formatVolumeCN(n: number): string {
  if (!isFinite(n) || n < 0) return "0";
  if (n >= 1e8) return (n / 1e8).toFixed(2) + "亿";
  if (n >= 1e4) return (n / 1e4).toFixed(0) + "万";
  return n.toFixed(0);
}

export function formatAmountCN(n: number): string {
  if (!isFinite(n) || n < 0) return "0.00";
  if (n >= 1e8) return (n / 1e8).toFixed(2) + "亿";
  if (n >= 1e4) return (n / 1e4).toFixed(2) + "万";
  return n.toFixed(2);
}

export function autoPrecision(price: number): number {
  if (price < 1) return 4;
  if (price < 10) return 3;
  if (price < 100) return 2;
  if (price < 1000) return 1;
  return 0;
}

export function formatPriceCN(price: number, maxPrice: number): string {
  return price.toFixed(autoPrecision(maxPrice));
}
