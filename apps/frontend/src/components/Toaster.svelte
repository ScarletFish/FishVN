<script>
  import { portal } from '../lib/portal.js';
  import { items, toast, dismiss, pause, resume } from '../lib/toast.svelte.js';
  import { t } from '../lib/i18n.js';

  const ICONS = {
    success:
      '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><path d="M22 4 12 14.01l-3-3"/></svg>',
    error:
      '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>',
    warning:
      '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>',
    info:
      '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg>',
    silent:
      '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 0 1-3.46 0"/></svg>',
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
      <div class="toast-icon">{@html ICONS[item.type] ?? ICONS.info}</div>
      <div class="toast-text">
        <div class="toast-title">{item.title}</div>
        {#if item.desc}<div class="toast-desc">{item.desc}</div>{/if}
      </div>
      <div class="toast-progress" style="animation-duration: {item.duration}ms"></div>
    </div>
  {/each}
</div>
