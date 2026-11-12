<script>
  import { portal } from '../lib/portal.js';
  import { items, toast, dismiss, pause, resume } from '../lib/toast.svelte.js';
  import { t } from '../lib/i18n.js';
  import Icon from './Icon.svelte';

  const ICONS = {
    success: 'check-circle',
    error: 'x-circle',
    warning: 'warning',
    info: 'info',
    silent: 'bell-slash',
  };

  function copy(item) {
    const text = item.title + (item.desc ? '\n' + item.desc : '');
    navigator.clipboard
      .writeText(text)
      .then(() => toast(t('toast.copied'), 'success', { duration: 1500 }))
      .catch(() => toast(t('toast.copyFailed'), 'error'));
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div class="toast-container" role="alert" aria-live="polite" use:portal>
  {#each items as item (item.id)}
    <div
      class="toast"
      class:dismissing={item.dismissing}
      data-type={item.type}
      role="button"
      tabindex="0"
      onmouseenter={() => pause(item)}
      onmouseleave={() => resume(item)}
      onclick={() => dismiss(item.id)}
      onkeydown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') dismiss(item.id);
      }}
      oncontextmenu={(e) => {
        e.preventDefault();
        e.stopPropagation();
        copy(item);
      }}
    >
      <div class="toast-icon"><Icon name={ICONS[item.type] ?? ICONS.info} size="lg" /></div>
      <div class="toast-text">
        <div class="toast-title">{item.title}</div>
        {#if item.desc}<div class="toast-desc">{item.desc}</div>{/if}
      </div>
      <div class="toast-progress" style="animation-duration: {item.duration}ms"></div>
    </div>
  {/each}
</div>
