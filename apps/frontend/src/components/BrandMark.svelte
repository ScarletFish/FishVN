<script>
  // 品牌角标：圆角矩形 + 鱼形剪影。
  // 鱼身填背景色而非白色，在 accent 底上呈挖空效果；眼睛再挖回accent 色。
  let { label = 'FishVN' } = $props();
</script>

<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" role="img" aria-label={label}>
  <rect x="0" y="0" width="16" height="16" rx="3" fill="var(--accent)" stroke="var(--border-default)" stroke-width="0.5" />
  <path
    d="M6.5 4.4c2.4 0 4.3 1.5 4.3 3.6s-1.9 3.6-4.3 3.6-4.3-1.5-4.3-3.6 1.9-3.6 4.3-3.6z"
    fill="var(--bg-page)"
  />
  <path d="M10.4 8l4.4-3.6-1.6 3.6 1.6 3.6-4.4-3.6z" fill="var(--bg-page)" />
  <circle cx="5" cy="7.2" r="0.85" fill="var(--accent)" />
</svg>