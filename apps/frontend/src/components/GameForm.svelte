<script>
  import { tick } from 'svelte';
  import { portal } from '../lib/portal.js';
  import { api } from '../lib/api.js';
  import { t } from '../lib/i18n.js';

  let { open = $bindable(false), onsaved } = $props();

  let name = $state('');
  let error = $state('');
  let saving = $state(false);
  let input = $state(null);

  // inert 而不是 CSS pointer-events：后者挡得住鼠标但 Tab 仍能走到背景里。
  // 弹窗挂 #modal-root 与 #app 平级，只屏蔽背景不影响弹窗内交互。
  $effect(() => {
    const app = document.getElementById('app');
    if (!app) return;
    app.inert = open;
    return () => {
      app.inert = false;
    };
  });

  // 每次打开都重置，避免上次没关干净的内容留在输入框里
  $effect(() => {
    if (!open) return;
    name = '';
    error = '';
    tick().then(() => input?.focus());
  });

  function close() {
    if (saving) return;
    open = false;
  }

  async function submit(e) {
    e.preventDefault();
    if (saving) return;

    const value = name.trim();
    if (!value) {
      // 必填在前端拦，走 i18n；后端的必填错误只作为兜底出现
      error = t('gameForm.error.nameRequired');
      return;
    }

    saving = true;
    error = '';
    try {
      await api.post('/games', { name: value });
      open = false;
      onsaved?.();
    } catch (err) {
      error = err.message;
    } finally {
      saving = false;
    }
  }

  function onOverlayClick(e) {
    // 判定落点，弹窗内部的点击会冒泡上来，不能一律当成点遮罩
    if (e.target === e.currentTarget) close();
  }

  function onKeydown(e) {
    if (e.key === 'Escape' && open) close();
  }
</script>

<svelte:window onkeydown={onKeydown} />

{#if open}
  <div class="modal-overlay show" use:portal onclick={onOverlayClick} role="presentation">
    <!-- form 元素承担 dialog 角色：回车提交走浏览器原生隐式提交，
         隐藏 submit 按钮会被 Svelte 判为无标签控件 -->
    <form
      class="modal modal--form"
      role="dialog"
      aria-modal="true"
      aria-label={t('gameForm.title')}
      onsubmit={submit}
    >
      <div class="modal-header">
        <h2>{t('gameForm.title')}</h2>
      </div>

      <div class="modal-panels">
        <div class="modal-panel active">
          <div class="form-group">
            <label for="game-form-name">{t('gameForm.field.name')}</label>
            <input
              id="game-form-name"
              type="text"
              bind:value={name}
              bind:this={input}
              disabled={saving}
            />
            {#if error}
              <p class="form-hint form-hint--error">{error}</p>
            {/if}
          </div>
        </div>
      </div>

      <div class="modal-actions">
        <button type="button" class="btn" onclick={close} disabled={saving}>
          {t('common.cancel')}
        </button>
        <button type="submit" class="btn btn-primary" disabled={saving}>
          {t('common.save')}
        </button>
      </div>
    </form>
  </div>
{/if}
