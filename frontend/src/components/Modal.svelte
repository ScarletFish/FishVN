<script>
  import { portal } from '../lib/portal.js';

  let {
    open = $bindable(false),
    title,
    variant = '',
    onclose,
    asideOpen = $bindable(false),
    asideTitle,
    aside,
    children,
  } = $props();

  $effect(() => {
    const app = document.getElementById('app');
    if (!app) return;
    app.inert = open;
    return () => {
      app.inert = false;
    };
  });

  function close() {
    if (onclose?.() === false) return;
    open = false;
  }

  function dismiss() {
    if (asideOpen) {
      asideOpen = false;
      return;
    }
    close();
  }

  function onOverlayClick(e) {
    if (e.target === e.currentTarget) dismiss();
  }

  function onKeydown(e) {
    if (e.key === 'Escape' && open) dismiss();
  }
</script>

<svelte:window onkeydown={onKeydown} />

{#if open}
  <div
    class="modal-overlay show"
    class:modal-overlay--split={asideOpen}
    use:portal
    onclick={onOverlayClick}
    role="presentation"
  >
    <div
      class="modal {variant}"
      role="dialog"
      aria-modal="true"
      aria-label={title}
    >
      {#if title}
        <div class="modal-header">
          <h2>{title}</h2>
        </div>
      {/if}
      {@render children()}
    </div>

    {#if asideOpen && aside}
      <div class="modal modal--aside" role="dialog" aria-label={asideTitle}>
        {#if asideTitle}
          <div class="modal-header">
            <h2>{asideTitle}</h2>
          </div>
        {/if}
        {@render aside()}
      </div>
    {/if}
  </div>
{/if}
