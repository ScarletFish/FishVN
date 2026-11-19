<script>
  import Icon from './Icon.svelte';
  import { portal } from '../lib/portal.js';
  import { copyText } from '../lib/clipboard.js';
  import { currentEditMenu } from '../lib/edit-menu.svelte.js';
  import { placeBubble, selectionInfo } from '../lib/selection.js';
  import { t } from '../lib/i18n.js';
  import { toast } from '../lib/toast.svelte.js';

  let box = $state(null);
  let el = $state(null);
  let pos = $state({ left: 0, top: 0 });

  function refresh() {
    const info = selectionInfo(window.getSelection());
    box = info
      ? { cx: info.rect.left + info.rect.width / 2, top: info.rect.top, text: info.text }
      : null;
  }

  function hide() {
    box = null;
  }

  async function copy() {
    const text = box?.text;
    hide();
    if (!text) return;
    try {
      await copyText(text);
      toast(t('toast.copied'), 'success', { duration: 1500 });
    } catch {
      toast(t('toast.copyFailed'), 'error');
    }
  }

  $effect(() => {
    const onUp = () => refresh();
    const onDown = (event) => {
      if (el && el.contains(event.target)) return;
      hide();
    };
    const onSelectionChange = () => {
      const sel = window.getSelection();
      if (!sel || sel.isCollapsed || !sel.toString().trim()) hide();
    };
    const onKey = (event) => {
      if (event.key === 'Escape') hide();
      else refresh();
    };
    window.addEventListener('mouseup', onUp);
    window.addEventListener('mousedown', onDown, true);
    document.addEventListener('selectionchange', onSelectionChange);
    window.addEventListener('keyup', onKey);
    window.addEventListener('scroll', hide, true);
    window.addEventListener('resize', hide);
    return () => {
      window.removeEventListener('mouseup', onUp);
      window.removeEventListener('mousedown', onDown, true);
      document.removeEventListener('selectionchange', onSelectionChange);
      window.removeEventListener('keyup', onKey);
      window.removeEventListener('scroll', hide, true);
      window.removeEventListener('resize', hide);
    };
  });

  $effect(() => {
    if (currentEditMenu()) hide();
  });

  $effect(() => {
    if (!box) return;
    const width = el?.offsetWidth || 80;
    const height = el?.offsetHeight || 32;
    pos = placeBubble(box.cx, box.top, width, height, {
      width: window.innerWidth,
      height: window.innerHeight,
    });
  });
</script>

{#if box}
  <button
    type="button"
    class="selection-copy"
    bind:this={el}
    style:left="{pos.left}px"
    style:top="{pos.top}px"
    onmousedown={(event) => event.stopPropagation()}
    onclick={(event) => {
      event.stopPropagation();
      copy();
    }}
    use:portal
  >
    <Icon name="copy" size="md" />
    <span>{t('selectionCopy.label')}</span>
  </button>
{/if}

<style>
  .selection-copy {
    position: fixed;
    z-index: var(--z-popover);
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    padding: var(--space-1) var(--space-2);
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    color: var(--fg-default);
    white-space: nowrap;
    background: var(--bg-floating);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    box-shadow: var(--shadow-md);
    cursor: pointer;
    animation: selection-copy-in var(--duration-fast) var(--ease-out);
  }

  .selection-copy:hover {
    background: var(--bg-raised);
  }

  @keyframes selection-copy-in {
    from {
      opacity: 0;
      transform: translateY(2px);
    }
    to {
      opacity: 1;
      transform: none;
    }
  }

  :global(:root[data-reduce-motion='on']) .selection-copy {
    animation: none;
  }
</style>
