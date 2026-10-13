<script>
  /** 列表卡片：封面缺失时用游戏名首字符占位 */
  import { t } from '../lib/i18n.js';
  import Menu from './Menu.svelte';

  let { item, onopen, onedit } = $props();

  const initial = $derived((item.name || '?')[0].toUpperCase());

  const menuItems = $derived([
    { label: t('common.actionEdit'), onclick: () => onedit?.(item.id) },
  ]);
</script>

<div class="cover-card">
  <div class="cover-card__media">
    {#if item.cover_path}
      <img src={item.cover_path} alt="" loading="lazy" decoding="async" />
    {:else}
      <div class="cover-ph">{initial}</div>
    {/if}
  </div>

  <!-- 可点区域独立成真按钮，与菜单互为兄弟：键盘原生可用，也不必拦截冒泡 -->
  <button
    type="button"
    class="cover-card__hit"
    aria-label={item.name}
    onclick={() => onopen?.(item.id)}
  ></button>

  <div class="cover-card__overlay">
    <h3 class="cover-card__title">{item.name}</h3>
  </div>

  <div class="cover-card__menu">
    <Menu items={menuItems} label={t('common.actionMore')} variant="plain" />
  </div>
</div>
