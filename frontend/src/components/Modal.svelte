<script>
  import { portal } from '../lib/portal.js';

  let { open = $bindable(false), title, variant = '', onclose, children } = $props();

  // pointer-events 挡不住 Tab，键盘仍会走到背景里，所以用 inert。
  // 弹窗挂 #modal-root 与 #app 平级，只屏蔽背景不影响弹窗内交互。
  $effect(() => {
    const app = document.getElementById('app');
    if (!app) return;
    app.inert = open;
    return () => {
      app.inert = false;
    };
  });

  // onclose 返回 false 可否决关闭，用于「保存中不许关」这类拦截
  function close() {
    if (onclose?.() === false) return;
    open = false;
  }

  // 弹窗内部的点击会冒泡到遮罩，不能一律当成点遮罩
  function onOverlayClick(e) {
    if (e.target === e.currentTarget) close();
  }

  function onKeydown(e) {
    if (e.key === 'Escape' && open) close();
  }
</script>

<svelte:window onkeydown={onKeydown} />

{#if open}
  <div class="modal-overlay show" use:portal onclick={onOverlayClick} role="presentation">
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
  </div>
{/if}
