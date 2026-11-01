export function formatScore(rating) {
  return (rating / 10).toFixed(2);
}

export function formatScoreShort(rating) {
  const text = (rating / 10).toFixed(1);
  return text.endsWith('.0') ? text.slice(0, -2) : text;
}
