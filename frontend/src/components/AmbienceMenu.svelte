<script>
  import { ContextMenu } from 'bits-ui';
  import { t } from '../lib/i18n.js';
  import { pickImage } from '../lib/pick-path.js';
  import { toast } from '../lib/toast.svelte.js';

  let {
    source = 'follow',
    mode = 'soft',
    modeKey = 'ambience_mode',
    plate = false,
    showPlate = false,
    showFocal = false,
    canFocal = true,
    onpatch,
    onfocal,
    trigger,
  } = $props();

  let menuKey = $state(0);

  async function setMode(next) {
    if ((await onpatch({ [modeKey]: next })) === false) menuKey += 1;
  }

  async function setPlate(next) {
    if ((await onpatch({ ambience_plate: next })) === false) menuKey += 1;
  }

  async function chooseImage() {
    let picked;
    try {
      picked = await pickImage(t('gameForm.field.ambience'));
    } catch (err) {
      toast(err.message, 'error');
      return false;
    }
    if (!picked) return false;
    const ok = await onpatch({ ambience_path: picked });
    return ok !== false;
  }

  async function setSource(next) {
    if (next === 'follow') {
      if (source === 'follow') return;
      if ((await onpatch({ ambience_path: null })) === false) menuKey += 1;
      return;
    }
    if (!(await chooseImage())) menuKey += 1;
  }
</script>

<ContextMenu.Root>
  <ContextMenu.Portal to="#modal-root">
    <ContextMenu.Content class="context-menu context-menu--panel" strategy="fixed" sideOffset={2}>
      <div class="context-menu__heading">{t('gameForm.ambience.source')}</div>
      {#key menuKey}
        <div
          class="seg-radio-group"
          role="radiogroup"
          aria-label={t('gameForm.ambience.source')}
        >
          <label class="seg-radio-item">
            <input
              type="radio"
              name="ambience-source-menu"
              value="follow"
              checked={source === 'follow'}
              onchange={() => setSource('follow')}
            />
            {t('gameForm.ambience.follow')}
          </label>
          <label class="seg-radio-item">
            <input
              type="radio"
              name="ambience-source-menu"
              value="custom"
              checked={source === 'custom'}
              onchange={() => setSource('custom')}
            />
            {t('gameForm.ambience.custom')}
          </label>
        </div>
        <div class="context-menu__heading">{t('gameForm.ambience.mode')}</div>
        <div class="seg-radio-group" role="radiogroup" aria-label={t('gameForm.ambience.mode')}>
          <label class="seg-radio-item">
            <input
              type="radio"
              name="ambience-mode-menu"
              value="soft"
              checked={mode === 'soft'}
              onchange={() => setMode('soft')}
            />
            {t('gameForm.ambience.soft')}
          </label>
          <label class="seg-radio-item">
            <input
              type="radio"
              name="ambience-mode-menu"
              value="clear"
              checked={mode === 'clear'}
              onchange={() => setMode('clear')}
            />
            {t('gameForm.ambience.clear')}
          </label>
        </div>
        {#if showPlate && mode === 'clear'}
          <label class="context-menu__toggle">
            <span>{t('gameForm.field.ambiencePlate')}</span>
            <span class="toggle-switch toggle-switch--sm">
              <input
                type="checkbox"
                checked={plate}
                onchange={(e) => setPlate(e.currentTarget.checked)}
              />
              <span class="toggle-slider"></span>
            </span>
          </label>
        {/if}
      {/key}
      {#if showFocal}
        <div class="context-menu__sep" role="separator"></div>
        <ContextMenu.Item class="menu__item" disabled={!canFocal} onSelect={onfocal}>
          {t('gameForm.ambience.focalJump')}
        </ContextMenu.Item>
      {/if}
    </ContextMenu.Content>
  </ContextMenu.Portal>
  <ContextMenu.Trigger>
    {#snippet child({ props })}
      {@render trigger(props)}
    {/snippet}
  </ContextMenu.Trigger>
</ContextMenu.Root>
