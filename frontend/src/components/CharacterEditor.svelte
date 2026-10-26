<script>
  import { tick, untrack } from 'svelte';
  import { t } from '../lib/i18n.js';
  import { initScrollDots } from '../lib/scroll-dots.svelte.js';
  import { panelScrollbars } from '../lib/scrollbars.js';
  import { coverSrc } from '../lib/cover-src.js';
  import { pickImage } from '../lib/pick-path.js';
  import {
    blankCharacter,
    blankVa,
    displayName,
    moveCharacters,
  } from '../lib/characters.js';
  import SelectDd from './SelectDd.svelte';

  let { characters = $bindable([]), disabled = false } = $props();

  const GENDERS = ['f', 'm', 't', 'i'];
  const ROLES = ['main', 'primary', 'side', 'appears'];

  function genderLabel(value) {
    return value ? t(`detail.gender.${value}`) : t('gameForm.placeholder.charGenderEmpty');
  }
  function roleLabel(value) {
    return value ? t(`gameForm.charRole.${value}`) : t('gameForm.placeholder.charRoleEmpty');
  }

  const GENDER_OPTIONS = [
    { value: '', label: genderLabel('') },
    ...GENDERS.map((key) => ({ value: key, label: genderLabel(key) })),
  ];
  const ROLE_OPTIONS = [
    { value: '', label: roleLabel('') },
    ...ROLES.map((key) => ({ value: key, label: roleLabel(key) })),
  ];

  let scrollEl = $state(null);
  let dotsHostEl = $state(null);
  let selectedId = $state(null);
  let artUrls = $state({});
  let vaDraft = $state('');
  let vaOpen = $state(false);
  let portraitError = $state('');
  let genderOpen = $state(false);
  let roleOpen = $state(false);

  const selected = $derived(characters.find((c) => c.id === selectedId) ?? null);
  const selectedIndex = $derived(characters.findIndex((c) => c.id === selectedId));

  $effect(() => {
    const ids = characters.map((c) => c.id);
    untrack(() => {
      if (characters.length === 0) {
        selectedId = null;
        return;
      }
      const current = selectedId;
      if (!current || !ids.includes(current)) {
        selectedId = characters[0].id;
      }
    });
  });

  $effect(() => {
    if (!scrollEl || !dotsHostEl) return;
    return initScrollDots({
      scroll: scrollEl,
      total: characters.length + (disabled ? 0 : 1),
      dotsParent: dotsHostEl,
    });
  });

  $effect(() => {
    let live = true;
    for (const character of characters) {
      const path = character.imagePath.trim();
      if (!path || artUrls[path]) continue;
      coverSrc(path)
        .then((url) => {
          if (live) artUrls = { ...artUrls, [path]: url };
        })
        .catch(() => {});
    }
    return () => {
      live = false;
    };
  });

  function art(path) {
    if (!path) return null;
    return artUrls[path.trim()] ?? null;
  }

  let moving = false;
  const SCROLL_SECONDS = 0.3;
  const CARD_SECONDS = 0.3;

  const reduceMotion = () => matchMedia('(prefers-reduced-motion: reduce)').matches;
  const stripCards = () => scrollEl?.querySelectorAll('.char-card:not(.char-card--add)');

  const layoutLeft = (node) => node.offsetLeft;

  function tweenScroll(target) {
    const gsap = globalThis.gsap;
    const to = Math.max(0, target);
    if (!gsap || reduceMotion()) {
      scrollEl.scrollLeft = to;
      return Promise.resolve();
    }
    return new Promise((resolve) => {
      gsap.to(scrollEl, {
        scrollLeft: to,
        duration: SCROLL_SECONDS,
        ease: 'power2.out',
        overwrite: true,
        onInterrupt: resolve,
        onComplete: resolve,
      });
    });
  }

  function flipCards(before) {
    const gsap = globalThis.gsap;
    if (!gsap || reduceMotion()) return Promise.resolve();
    const running = [];
    for (const [card, from] of before) {
      const delta = from - layoutLeft(card);
      if (Math.abs(delta) < 1) continue;
      running.push(
        new Promise((resolve) => {
          gsap.fromTo(card, { x: delta }, {
            x: 0,
            duration: CARD_SECONDS,
            ease: 'power2.out',
            overwrite: true,
            onInterrupt: resolve,
            onComplete: () => {
              gsap.set(card, { clearProps: 'transform' });
              resolve();
            },
          });
        })
      );
    }
    return Promise.all(running);
  }

  function stepIntoView(index) {
    const cards = stripCards();
    const card = cards?.[index];
    if (!card || !scrollEl) return;
    const gap = parseFloat(getComputedStyle(scrollEl).columnGap) || 0;
    const step = card.offsetWidth + gap;
    if (!step) return;
    const visible = Math.max(1, Math.floor((scrollEl.clientWidth + gap) / step));
    const slack = visible > 2 ? 1 : 0;
    const first = Math.round(scrollEl.scrollLeft / step);
    let next = first;
    if (index - first < slack) next = index - slack;
    else if (index - first > visible - 1 - slack) next = index - (visible - 1 - slack);
    next = Math.max(0, Math.min(next, cards.length - 1));
    const target = layoutLeft(cards[next]) - layoutLeft(cards[0]);
    if (Math.abs(target - scrollEl.scrollLeft) < 1) return;
    return tweenScroll(target);
  }

  async function addCharacter() {
    const created = blankCharacter();
    characters = [...characters, created];
    selectedId = created.id;
    await tick();
    globalThis.gsap?.killTweensOf(scrollEl);
    scrollEl?.scrollTo({ left: scrollEl.scrollWidth, behavior: 'smooth' });
  }

  function removeCharacter() {
    if (!selected) return;
    characters = characters.filter((c) => c.id !== selected.id);
    selectedId = null;
  }

  async function moveSelected(delta) {
    if (moving || !scrollEl || selectedIndex < 0) return;
    const target = selectedIndex + delta;
    if (target < 0 || target >= characters.length) return;
    moving = true;
    const before = new Map();
    for (const card of stripCards() ?? []) before.set(card, layoutLeft(card));
    scrollEl.style.scrollSnapType = 'none';
    characters = moveCharacters(characters, selectedIndex, delta);
    try {
      await tick();
      await Promise.all([flipCards(before), stepIntoView(target)]);
    } finally {
      scrollEl.style.scrollSnapType = '';
      moving = false;
    }
  }

  async function browsePortrait() {
    if (!selected) return;
    portraitError = '';
    try {
      const picked = await pickImage(t('gameForm.field.charPortrait'));
      if (picked) selected.imagePath = picked;
    } catch (err) {
      portraitError = err.message;
    }
  }

  function addVa() {
    const value = vaDraft.trim();
    if (!value || !selected) return;
    selected.va = [...selected.va, { ...blankVa(), name: value }];
    vaDraft = '';
  }

  function onVaKeydown(e) {
    if (e.key === 'Enter' || e.key === ',') {
      e.preventDefault();
      addVa();
      return;
    }
    if (e.key === 'Escape') {
      e.preventDefault();
      vaDraft = '';
      vaOpen = false;
    }
  }

  function removeVa(key) {
    if (!selected) return;
    selected.va = selected.va.filter((entry) => entry.key !== key);
  }

  function toggle(id) {
    selectedId = selectedId === id ? null : id;
  }
</script>

<section class="char-editor">
  <div class="char-editor-head">
    <h4>{t('gameForm.tab.characters')}</h4>
    <span class="char-editor-count">{characters.length}</span>
    <span class="char-editor-dots" bind:this={dotsHostEl}></span>
  </div>

  <div class="char-strip hscroll-section" bind:this={scrollEl}>
    {#each characters as c, i (c.id)}
      <div
        class="char-card hscroll-card"
        class:expanded={selectedId === c.id}
        class:has-art={Boolean(art(c.imagePath))}
        role="button"
        tabindex="0"
        aria-expanded={selectedId === c.id}
        title={c.va[0] ? `${displayName(c)} · CV ${c.va[0].original || c.va[0].name}` : displayName(c)}
        onclick={() => toggle(c.id)}
        onkeydown={(e) => {
          if (e.key === 'Enter') toggle(c.id);
        }}
      >
        <div class="char-card-art">
          {#if art(c.imagePath)}
            <img src={art(c.imagePath)} alt={displayName(c)} />
          {:else}
            <span class="char-card-ph">{displayName(c) || `${t('gameForm.placeholder.charName')}${i + 1}`}</span>
          {/if}
        </div>
        {#if art(c.imagePath)}
          <div class="char-card-foot">
            <span class="char-card-name">{displayName(c) || `${t('gameForm.placeholder.charName')}${i + 1}`}</span>
          </div>
        {/if}
      </div>
    {/each}
    {#if !disabled}
      <button
        type="button"
        class="char-card char-card--add hscroll-card"
        onclick={addCharacter}
        aria-label={t('gameForm.action.addCharacter')}
      >
        <svg
          viewBox="0 0 24 24"
          width="22"
          height="22"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          aria-hidden="true"
        >
          <line x1="12" y1="5" x2="12" y2="19" />
          <line x1="5" y1="12" x2="19" y2="12" />
        </svg>
        <span>{t('gameForm.action.addCharacter')}</span>
      </button>
    {/if}
  </div>

  {#if selected}
    <div class="char-panel">
      <div class="char-panel-toolbar">
        <button type="button" class="btn" onclick={browsePortrait} {disabled}>
          {t('gameForm.action.browsePortrait')}
        </button>
        <div class="char-panel-move">
          <button
            type="button"
            class="btn"
            onclick={() => moveSelected(-1)}
            disabled={disabled || selectedIndex === 0}
            aria-label={t('gameForm.action.moveEarlier')}
          >←</button>
          <button
            type="button"
            class="btn"
            onclick={() => moveSelected(1)}
            disabled={disabled || selectedIndex === characters.length - 1}
            aria-label={t('gameForm.action.moveLater')}
          >→</button>
        </div>
        {#if portraitError}
          <p class="form-hint form-hint--error char-panel-error">{portraitError}</p>
        {/if}
        <button type="button" class="btn btn-danger char-panel-remove" onclick={removeCharacter} {disabled}>
          {t('gameForm.action.removeCharacter')}
        </button>
      </div>

      <div class="char-panel-body">
        <div class="panel-grid">
          <div class="form-group">
            <label for="char-name">{t('gameForm.field.charName')}</label>
            <input
              id="char-name"
              type="text"
              bind:value={selected.name}
              placeholder={t('gameForm.placeholder.charName')}
              {disabled}
            />
          </div>
          <div class="form-group">
            <label for="char-original">{t('gameForm.field.charOriginal')}</label>
            <input id="char-original" type="text" bind:value={selected.original} {disabled} />
          </div>
          <div class="form-group">
            <label for="char-gender">{t('gameForm.field.charGender')}</label>
            <SelectDd
              id="char-gender"
              options={GENDER_OPTIONS}
              value={selected.gender}
              open={genderOpen}
              onopenchange={(v) => (genderOpen = v)}
              onchange={(v) => (selected.gender = v)}
              {disabled}
            />
          </div>
          <div class="form-group">
            <label for="char-role">{t('gameForm.field.charRole')}</label>
            <SelectDd
              id="char-role"
              options={ROLE_OPTIONS}
              value={selected.role}
              open={roleOpen}
              onopenchange={(v) => (roleOpen = v)}
              onchange={(v) => (selected.role = v)}
              {disabled}
            />
          </div>
          <div class="form-group">
            <label for="char-age">{t('gameForm.field.charAge')}</label>
            <input id="char-age" type="text" inputmode="numeric" bind:value={selected.age} {disabled} />
          </div>
          <div class="form-group">
            <label for="char-height">{t('gameForm.field.charHeight')}</label>
            <input id="char-height" type="text" inputmode="numeric" bind:value={selected.height} {disabled} />
          </div>
          <div class="form-group">
            <label for="char-weight">{t('gameForm.field.charWeight')}</label>
            <input id="char-weight" type="text" inputmode="numeric" bind:value={selected.weight} {disabled} />
          </div>
        </div>

        <div class="form-group">
          <label for="char-va">{t('gameForm.field.charVoice')}</label>
          <div class="tag-input" use:panelScrollbars>
            <div class="tag-input__flow">
              {#each selected.va as entry (entry.key)}
                <span class="tag-pill">
                  {entry.original || entry.name}
                  {#if entry.note}<span class="tag-pill__note">· {entry.note}</span>{/if}
                  <button
                    type="button"
                    class="tag-pill__remove"
                    aria-label={t('gameForm.action.removeVoice')}
                    onmousedown={(e) => e.preventDefault()}
                    onclick={() => removeVa(entry.key)}
                    {disabled}
                  >
                    <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
                      <line x1="18" y1="6" x2="6" y2="18" />
                      <line x1="6" y1="6" x2="18" y2="18" />
                    </svg>
                  </button>
                </span>
              {/each}
              <button
                type="button"
                class="tag-input__add"
                aria-expanded={vaOpen}
                aria-label={t('gameForm.action.addVoice')}
                onclick={() => (vaOpen = !vaOpen)}
                {disabled}
              >
                <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                  <line x1="12" y1="5" x2="12" y2="19" />
                  <line x1="5" y1="12" x2="19" y2="12" />
                </svg>
              </button>
            </div>
          </div>
          {#if vaOpen}
            <input
              id="char-va"
              class="tag-input__new"
              type="text"
              bind:value={vaDraft}
              onkeydown={onVaKeydown}
              placeholder={t('gameForm.placeholder.charVoice')}
              {disabled}
            />
          {/if}
        </div>

        <div class="form-group form-group--desc">
          <label for="char-description">{t('gameForm.field.charDescription')}</label>
          <textarea
            id="char-description"
            class="scroll-thin"
            bind:value={selected.description}
            placeholder={t('gameForm.placeholder.charDescription')}
            {disabled}
          ></textarea>
        </div>
      </div>
    </div>
  {/if}
</section>

<style>
  .char-strip {
    --card-w: 180px;
    padding-bottom: var(--space-2);
  }

  .char-editor-head {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .char-editor-head h4 {
    font-size: var(--text-body);
    font-weight: var(--fw-semibold);
    color: var(--fg-default);
  }

  .char-editor-dots {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-left: auto;
  }

  .char-editor-count {
    font-size: var(--text-xs);
    font-weight: var(--fw-medium);
    color: var(--fg-muted);
  }

  .char-card {
    position: relative;
    display: flex;
    flex-direction: column;
    background: var(--bg-surface);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    overflow: hidden;
    cursor: pointer;
    transition:
      border-color var(--duration-fast) var(--ease-out),
      background var(--duration-fast) var(--ease-out);
  }

  .char-card:hover {
    border-color: var(--border-strong);
  }

.char-card.expanded {
  border-color: var(--accent-border);
}

  .char-card--add {
    align-items: center;
    justify-content: center;
    gap: var(--space-1);
    aspect-ratio: 208 / 300;
    margin: 0;
    padding: 0;
    appearance: none;
    -webkit-appearance: none;
    color: var(--fg-muted);
    background: var(--bg-page);
    border-style: dashed;
    font: inherit;
    cursor: pointer;
    transition:
      border-color var(--duration-fast) var(--ease-out),
      background var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out);
  }

.char-card--add:hover {
  color: var(--fg-default);
  border-color: var(--border-strong);
  background: var(--bg-hover);
}

  .char-card--add span {
    font-size: var(--text-xs);
    font-weight: var(--fw-medium);
  }

  .char-card-art {
    aspect-ratio: 208 / 300;
    background: var(--bg-page);
  }

  .char-card-art img {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: cover;
    object-position: top;
  }

  .char-card-ph {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    padding: var(--space-3);
    text-align: center;
    color: var(--fg-muted);
    font-size: var(--text-sm);
    font-weight: var(--fw-medium);
    line-height: 1.4;
    word-break: break-word;
  }

  .char-card-foot {
    position: absolute;
    inset-inline: 0;
    bottom: 0;
    z-index: 1;
    pointer-events: none;
    padding: var(--space-3);
    background: var(--cover-scrim);
  }

  .char-card-name {
    display: block;
    max-width: 100%;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
    font-size: var(--text-xs);
    font-weight: var(--fw-medium);
    line-height: 1.35;
    color: var(--fg-on-accent);
  }

  .char-panel {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-4);
    background: var(--bg-page);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
  }

  .char-panel-toolbar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
  }

  .char-panel-error {
    margin: 0;
  }

  .char-panel-remove {
    margin-left: auto;
  }

  .char-panel-move {
    display: flex;
    gap: var(--space-2);
  }

  .char-panel-move .btn {
    padding: var(--space-1) var(--space-3);
  }
</style>
