<script>
  import { tick } from 'svelte';
  import Modal from './Modal.svelte';
  import { api } from '../lib/api.js';
  import { t } from '../lib/i18n.js';

  let { open = $bindable(false), onsaved } = $props();

  let name = $state('');
  let error = $state('');
  let saving = $state(false);
  let input = $state(null);

  // 关闭时组件不卸载，字段要在这里重置
  $effect(() => {
    if (!open) return;
    name = '';
    error = '';
    tick().then(() => input?.focus());
  });

  function close() {
    if (saving) return false;
    open = false;
  }

  async function submit(e) {
    e.preventDefault();
    if (saving) return;

    const value = name.trim();
    if (!value) {
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
      // 后端与网络错误不走 i18n，原样显示
      error = err.message;
    } finally {
      saving = false;
    }
  }
</script>

<Modal bind:open title={t('gameForm.title')} variant="modal--form" onclose={close}>
  <!-- form 只负责回车隐式提交，dialog 角色必须挂在非交互元素上 -->
  <form class="form-panel" onsubmit={submit}>
    <div class="modal-panels">
      <div class="modal-panel active">
        <div class="form-group">
          <label for="game-form-name">{t('gameForm.field.name')}</label>
          <input
            id="game-form-name"
            type="text"
            bind:value={name}
            bind:this={input}
            placeholder={t('gameForm.namePlaceholder')}
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
</Modal>
