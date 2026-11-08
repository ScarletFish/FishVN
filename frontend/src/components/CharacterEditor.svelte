<script>
  import { onMount, tick, untrack } from 'svelte';
  import { t } from '../lib/i18n.js';
  import { api } from '../lib/api.js';
  import { initScrollDots } from '../lib/scroll-dots.svelte.js';
  import { panelScrollbars } from '../lib/scrollbars.js';
  import { shouldReduceMotion } from '../lib/motion.js';
  import { coverSrc } from '../lib/cover-src.js';
  import { pickImage } from '../lib/pick-path.js';
  import {
    blankCharacter,
    blankVa,
    cleanCharNumber,
    cleanCup,
    displayName,
    moveCharacters,
    newLocalId,
    normalizeCharNumber,
  } from '../lib/characters.js';
  import SelectDd from './SelectDd.svelte';
  import TagField from './TagField.svelte';
  import { traitLabel, getTraitDict } from '../lib/tag-label.svelte.js';
  import { staffDisplay, staffOriginal } from '../lib/staff-label.js';
  import { isValidBirthday } from '../lib/form-metadata.js';

  let { characters = $bindable([]), disabled = false, onTraitBrowse, selectedId = $bindable(null) } = $props();

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

  const BLOOD_TYPES = ['A', 'B', 'AB', 'O'];
  function bloodLabel(value) {
    return value
      ? t(`detail.bloodType.${String(value).toLowerCase()}`)
      : t('gameForm.placeholder.charBloodTypeEmpty');
  }
  const BLOOD_OPTIONS = [
    { value: '', label: bloodLabel('') },
    ...BLOOD_TYPES.map((key) => ({ value: key, label: bloodLabel(key) })),
  ];

  let scrollEl = $state(null);
  let dotsHostEl = $state(null);
  let artUrls = $state({});
  let vaDraft = $state('');
  let vaOpen = $state(false);
  let vaField = $state(null);
  let vaEntry = $state(null);
  let staffOptions = $state([]);
  let vaEditingKey = $state(null);
  let vaNameDraft = $state('');
  let vaOriginalDraft = $state('');
  let vaMergeQuery = $state('');
  let vaBusy = $state(false);
  let portraitError = $state('');
  let birthdayError = $state(false);
  let genderOpen = $state(false);
  let roleOpen = $state(false);
  let bloodOpen = $state(false);
  let aliasDraft = $state('');
  let aliasOpen = $state(false);
  let aliasField = $state(null);
  let aliasEntry = $state(null);
  let traitDraft = $state('');

  const selected = $derived(characters.find((c) => c.id === selectedId) ?? null);
  const selectedIndex = $derived(characters.findIndex((c) => c.id === selectedId));

  function onNumberInput(field, event) {
    const cleaned = cleanCharNumber(field, event.currentTarget.value);
    if (cleaned !== event.currentTarget.value) event.currentTarget.value = cleaned;
    selected[field] = cleaned;
  }

  function onNumberBlur(field) {
    selected[field] = normalizeCharNumber(field, selected[field]);
  }

  function onCupInput(event) {
    const cleaned = cleanCup(event.currentTarget.value);
    if (cleaned !== event.currentTarget.value) event.currentTarget.value = cleaned;
    selected.cup = cleaned;
  }

  function onBirthdayInput() {
    birthdayError = false;
  }

  function onBirthdayBlur() {
    birthdayError = !isValidBirthday(selected.birthday);
  }

  const TRAIT_GROUP_ORDER = [
    'Hair',
    'Eyes',
    'Body',
    'Clothes',
    'Items',
    'Personality',
    'Role',
    'Engages in',
    'Subject of',
    'uncategorized',
  ];

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

  const reduceMotion = shouldReduceMotion;
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
    if (selectedIndex < 0) return;
    portraitError = '';
    try {
      const picked = await pickImage(t('gameForm.field.charPortrait'));
      if (picked) {
        characters = characters.map((c, i) => (i === selectedIndex ? { ...c, imagePath: picked } : c));
      }
    } catch (err) {
      portraitError = err.message;
    }
  }

  onMount(async () => {
    try {
      staffOptions = (await api.listStaff()) ?? [];
    } catch {
      staffOptions = [];
    }
  });

  function staffHay(s) {
    return [staffDisplay(s), s.name, staffOriginal(s), s.original]
      .filter(Boolean)
      .join('\n')
      .toLowerCase();
  }

  const vaSuggestions = $derived.by(() => {
    const query = vaDraft.trim().toLowerCase();
    if (!query || selectedIndex < 0) return [];
    const owned = new Set(characters[selectedIndex].va.map((entry) => entry.staffId));
    return staffOptions
      .filter((s) => !owned.has(s.id) && staffHay(s).includes(query))
      .slice(0, 6);
  });

  const editingVa = $derived(
    selectedIndex >= 0
      ? (characters[selectedIndex].va.find((entry) => entry.key === vaEditingKey) ?? null)
      : null,
  );

  const editingStaff = $derived(
    editingVa ? (staffOptions.find((s) => s.id === editingVa.staffId) ?? null) : null,
  );

  const vaMergeTargets = $derived.by(() => {
    const query = vaMergeQuery.trim().toLowerCase();
    if (!query || !editingVa) return [];
    return staffOptions
      .filter((s) => s.id !== editingVa.staffId && staffHay(s).includes(query))
      .slice(0, 6);
  });

  function patchVa(key, patch) {
    if (selectedIndex < 0) return;
    const next = characters[selectedIndex].va.map((entry) =>
      entry.key === key ? { ...entry, ...patch } : entry,
    );
    characters = characters.map((c, i) => (i === selectedIndex ? { ...c, va: next } : c));
  }

  function pushVa(entry) {
    if (selectedIndex < 0) return;
    const next = [...characters[selectedIndex].va, entry];
    characters = characters.map((c, i) => (i === selectedIndex ? { ...c, va: next } : c));
    vaDraft = '';
    vaField?.focus();
  }

  function addVa() {
    const value = vaDraft.trim();
    if (!value || selectedIndex < 0) return;
    const hit = staffOptions.find((s) =>
      [staffDisplay(s), s.name, staffOriginal(s), s.original].some(
        (n) => (n ?? '').trim().toLowerCase() === value.toLowerCase(),
      ),
    );
    if (hit) {
      pushVa({
        key: newLocalId(),
        staffId: hit.id,
        name: staffDisplay(hit) || value,
        original: staffOriginal(hit),
        note: '',
      });
      return;
    }
    pushVa({ ...blankVa(), name: value });
  }

  function pickVaSuggestion(s) {
    pushVa({
      key: newLocalId(),
      staffId: s.id,
      name: staffDisplay(s),
      original: staffOriginal(s),
      note: '',
    });
  }

  function openVaEdit(key) {
    const entry = selectedIndex >= 0
      ? characters[selectedIndex].va.find((item) => item.key === key)
      : null;
    if (!entry) return;
    vaEditingKey = key;
    const staff = staffOptions.find((s) => s.id === entry.staffId) ?? null;
    vaNameDraft = staffDisplay(staff) || entry.name || '';
    vaOriginalDraft = staffOriginal(staff) || entry.original || '';
    vaMergeQuery = '';
  }

  function closeVaEdit() {
    vaEditingKey = null;
    vaMergeQuery = '';
  }

  async function saveStaffMeta() {
    if (!editingVa || vaBusy || !vaNameDraft.trim()) return;
    vaBusy = true;
    try {
      const updated = await api.updateStaff({
        id: editingVa.staffId,
        name: vaNameDraft.trim(),
        original: vaOriginalDraft.trim(),
      });
      staffOptions = staffOptions.map((s) => (s.id === updated.id ? updated : s));
      patchVa(editingVa.key, {
        name: updated.displayName || vaNameDraft.trim(),
        original: updated.displayOriginal || '',
      });
    } finally {
      vaBusy = false;
    }
  }

  async function mergeVaInto(target) {
    if (!editingVa || vaBusy) return;
    vaBusy = true;
    try {
      const updated = await api.mergeStaff({
        target_id: target.id,
        source_id: editingVa.staffId,
      });
      staffOptions = staffOptions
        .filter((s) => s.id !== editingVa.staffId)
        .map((s) => (s.id === updated.id ? updated : s));
      patchVa(editingVa.key, {
        staffId: updated.id,
        name: updated.displayName || '',
        original: updated.displayOriginal || '',
      });
      closeVaEdit();
    } finally {
      vaBusy = false;
    }
  }

  async function openVoice() {
    vaOpen = true;
    await tick();
    vaField?.focus();
  }

  async function closeVoice() {
    vaOpen = false;
    vaDraft = '';
    await tick();
    vaEntry?.focus();
  }

  function onVoiceBlur() {
    if (!vaDraft.trim()) vaOpen = false;
  }

  function revealVoice(e) {
    if (e.target.closest('button, input, .tag-pill')) return;
    if (vaOpen) vaField?.focus();
    else openVoice();
  }

  function onVaKeydown(e) {
    if (e.key === 'Enter' || e.key === ',') {
      e.preventDefault();
      addVa();
      return;
    }
    if (e.key === 'Escape') {
      e.preventDefault();
      closeVoice();
    }
  }

  function removeVa(key) {
    if (selectedIndex < 0) return;
    const next = characters[selectedIndex].va.filter((entry) => entry.key !== key);
    characters = characters.map((c, i) => (i === selectedIndex ? { ...c, va: next } : c));
  }

  function addAlias() {
    const value = aliasDraft.trim();
    if (!value || selectedIndex < 0) return;
    if (!characters[selectedIndex].aliases.includes(value)) {
      const next = [...characters[selectedIndex].aliases, value];
      characters = characters.map((c, i) => (i === selectedIndex ? { ...c, aliases: next } : c));
    }
    aliasDraft = '';
    aliasField?.focus();
  }

  async function openAlias() {
    aliasOpen = true;
    await tick();
    aliasField?.focus();
  }

  async function closeAlias() {
    aliasOpen = false;
    aliasDraft = '';
    await tick();
    aliasEntry?.focus();
  }

  function onAliasBlur() {
    if (!aliasDraft.trim()) aliasOpen = false;
  }

  function revealAlias(e) {
    if (e.target.closest('button, input, .tag-pill')) return;
    if (aliasOpen) aliasField?.focus();
    else openAlias();
  }

  function onAliasKeydown(e) {
    if (e.key === 'Enter' || e.key === ',') {
      e.preventDefault();
      addAlias();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      closeAlias();
    }
  }
  function removeAlias(name) {
    if (selectedIndex < 0) return;
    const next = characters[selectedIndex].aliases.filter((a) => a !== name);
    characters = characters.map((c, i) => (i === selectedIndex ? { ...c, aliases: next } : c));
  }

  function resolveTrait(text) {
    const q = text.toLowerCase();
    const dict = getTraitDict();
    const hit = Object.entries(dict).find(
      ([id, entry]) => entry?.name?.toLowerCase() === q || id.toLowerCase() === q,
    );
    if (hit) {
      const [id, entry] = hit;
      return { id, name: entry.name, group_name: entry.group ?? null, spoiler: 0, lie: false };
    }
    return { id: null, name: text, group_name: null, spoiler: 0, lie: false };
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
            <div class="unit-field">
              <input
                id="char-age"
                type="text"
                inputmode="numeric"
                value={selected.age}
                oninput={(e) => onNumberInput('age', e)}
                onblur={() => onNumberBlur('age')}
                {disabled}
              />
              <span class="unit-field__suffix">{t('detail.unitAge')}</span>
            </div>
          </div>
          <div class="form-group">
            <label for="char-birthday">{t('gameForm.field.charBirthday')}</label>
            <input
              id="char-birthday"
              type="text"
              bind:value={selected.birthday}
              placeholder={t('gameForm.placeholder.charBirthday')}
              class:is-invalid={birthdayError}
              oninput={onBirthdayInput}
              onblur={onBirthdayBlur}
              {disabled}
            />
            {#if birthdayError}
              <p class="form-hint form-hint--error">{t('gameForm.error.charBirthdayFormat')}</p>
            {/if}
          </div>

          <div class="panel-grid__sub panel-grid__sub--3">
            <div class="form-group">
              <label for="char-height">{t('gameForm.field.charHeight')}</label>
              <div class="unit-field">
                <input
                  id="char-height"
                  type="text"
                  inputmode="numeric"
                  value={selected.height}
                  oninput={(e) => onNumberInput('height', e)}
                  onblur={() => onNumberBlur('height')}
                  {disabled}
                />
                <span class="unit-field__suffix">{t('detail.unitCm')}</span>
              </div>
            </div>
            <div class="form-group">
              <label for="char-weight">{t('gameForm.field.charWeight')}</label>
              <div class="unit-field">
                <input
                  id="char-weight"
                  type="text"
                  inputmode="numeric"
                  value={selected.weight}
                  oninput={(e) => onNumberInput('weight', e)}
                  onblur={() => onNumberBlur('weight')}
                  {disabled}
                />
                <span class="unit-field__suffix">{t('detail.unitKg')}</span>
              </div>
            </div>
            <div class="form-group">
              <label for="char-blood">{t('gameForm.field.charBloodType')}</label>
              <SelectDd
                id="char-blood"
                options={BLOOD_OPTIONS}
                value={selected.bloodType}
                open={bloodOpen}
                onopenchange={(v) => (bloodOpen = v)}
                onchange={(v) => (selected.bloodType = v)}
                {disabled}
              />
            </div>
          </div>

          <div
            class="measure-group panel-grid__full"
            role="group"
            aria-label={t('gameForm.field.charMeasure')}
          >
            <div class="measure-group__field">
              <label class="measure-group__tag" for="char-bust">B<span class="sr-only"> {t('gameForm.field.charBust')}</span></label>
              <input
                id="char-bust"
                type="text"
                inputmode="numeric"
                value={selected.bust}
                oninput={(e) => onNumberInput('bust', e)}
                onblur={() => onNumberBlur('bust')}
                {disabled}
              />
            </div>
            <div class="measure-group__field">
              <label class="measure-group__tag" for="char-waist">W<span class="sr-only"> {t('gameForm.field.charWaist')}</span></label>
              <input
                id="char-waist"
                type="text"
                inputmode="numeric"
                value={selected.waist}
                oninput={(e) => onNumberInput('waist', e)}
                onblur={() => onNumberBlur('waist')}
                {disabled}
              />
            </div>
            <div class="measure-group__field">
              <label class="measure-group__tag" for="char-hips">H<span class="sr-only"> {t('gameForm.field.charHips')}</span></label>
              <input
                id="char-hips"
                type="text"
                inputmode="numeric"
                value={selected.hips}
                oninput={(e) => onNumberInput('hips', e)}
                onblur={() => onNumberBlur('hips')}
                {disabled}
              />
            </div>
            <div class="measure-group__field">
              <label class="measure-group__tag" for="char-cup">{t('gameForm.field.charCup')}</label>
              <input
                id="char-cup"
                type="text"
                value={selected.cup}
                oninput={onCupInput}
                {disabled}
              />
            </div>
          </div>
        </div>

        <div class="form-group">
          <div class="form-label">{t('gameForm.field.charVoice')}</div>
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div class="tag-input" use:panelScrollbars onpointerdown={revealVoice}>
            <div class="tag-input__flow">
              {#each selected.va as entry (entry.key)}
                <span class="tag-pill" class:tag-pill--on={vaEditingKey === entry.key}>
                  <button
                    type="button"
                    class="tag-pill__label"
                    onclick={() => openVaEdit(entry.key)}
                    disabled={disabled}
                  >
                    {entry.original || entry.name}
                  </button>
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
              {#if vaOpen}
                <input
                  bind:this={vaField}
                  class="tag-input__field"
                  type="text"
                  bind:value={vaDraft}
                  onkeydown={onVaKeydown}
                  onblur={onVoiceBlur}
                  placeholder={t('gameForm.placeholder.charVoice')}
                  aria-label={t('gameForm.field.charVoice')}
                  {disabled}
                />
              {:else}
                <button
                  type="button"
                  class="tag-input__entry"
                  bind:this={vaEntry}
                  onclick={openVoice}
                  {disabled}
                >
                  <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                    <line x1="12" y1="5" x2="12" y2="19" />
                    <line x1="5" y1="12" x2="19" y2="12" />
                  </svg>
                  {t('gameForm.action.addVoice')}
                </button>
              {/if}
            </div>
          </div>
          {#if vaOpen && vaSuggestions.length}
            <div class="va-suggest">
              {#each vaSuggestions as suggestion (suggestion.id)}
                <button
                  type="button"
                  class="va-suggest__item"
                  onmousedown={(e) => e.preventDefault()}
                  onclick={() => pickVaSuggestion(suggestion)}
                >
                  {staffDisplay(suggestion)}{#if staffOriginal(suggestion)}<span class="va-suggest__orig">（{staffOriginal(suggestion)}）</span>{/if}
                  <span class="tag-pill__count">×{suggestion.roles}</span>
                </button>
              {/each}
            </div>
          {/if}
          {#if editingVa}
            <div class="va-edit">
              <div class="va-edit__head">
                <span class="va-edit__title">{t('gameForm.voice.editTitle')}</span>
                <button
                  type="button"
                  class="va-edit__close"
                  aria-label={t('common.close')}
                  onclick={closeVaEdit}
                >
                  <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
                    <line x1="18" y1="6" x2="6" y2="18" />
                    <line x1="6" y1="6" x2="18" y2="18" />
                  </svg>
                </button>
              </div>
              <div class="va-edit__row">
                <label class="va-edit__label" for="va-note">{t('gameForm.voice.noteLabel')}</label>
                <input
                  id="va-note"
                  class="va-edit__field"
                  type="text"
                  value={editingVa.note ?? ''}
                  oninput={(e) => patchVa(editingVa.key, { note: e.currentTarget.value })}
                  placeholder={t('gameForm.voice.notePlaceholder')}
                  {disabled}
                />
              </div>
              <div class="va-edit__row">
                <label class="va-edit__label" for="va-name">{t('gameForm.voice.staffName')}</label>
                <input
                  id="va-name"
                  class="va-edit__field"
                  type="text"
                  bind:value={vaNameDraft}
                  disabled={disabled || vaBusy}
                />
                <label class="va-edit__label" for="va-original">{t('gameForm.voice.staffOriginal')}</label>
                <input
                  id="va-original"
                  class="va-edit__field"
                  type="text"
                  bind:value={vaOriginalDraft}
                  disabled={disabled || vaBusy}
                />
                <button
                  type="button"
                  class="btn"
                  onclick={saveStaffMeta}
                  disabled={disabled || vaBusy || !vaNameDraft.trim()}
                >{t('gameForm.voice.save')}</button>
              </div>
              <p class="va-edit__hint">{t('gameForm.voice.globalHint')}</p>
              <div class="va-edit__row">
                <label class="va-edit__label" for="va-merge">{t('gameForm.voice.mergeLabel')}</label>
                <input
                  id="va-merge"
                  class="va-edit__field"
                  type="text"
                  bind:value={vaMergeQuery}
                  placeholder={t('gameForm.voice.mergePlaceholder')}
                  disabled={disabled || vaBusy}
                />
              </div>
              {#if vaMergeTargets.length}
                <div class="va-edit__targets">
                  {#each vaMergeTargets as target (target.id)}
                    <button
                      type="button"
                      class="tag-pill tag-pill--pick"
                      onclick={() => mergeVaInto(target)}
                      disabled={disabled || vaBusy}
                    >
                      {staffDisplay(target)}
                      <span class="tag-pill__count">×{target.roles}</span>
                    </button>
                  {/each}
                </div>
              {/if}
            </div>
          {/if}
        </div>

        <div class="form-group">
          <div class="form-label">{t('gameForm.field.charAliases')}</div>
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div class="tag-input" use:panelScrollbars onpointerdown={revealAlias}>
            <div class="tag-input__flow">
              {#each selected.aliases as name (name)}
                <span class="tag-pill">
                  {name}
                  <button
                    type="button"
                    class="tag-pill__remove"
                    aria-label={t('gameForm.action.removeAlias')}
                    onmousedown={(e) => e.preventDefault()}
                    onclick={() => removeAlias(name)}
                    {disabled}
                  >
                    <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
                      <line x1="18" y1="6" x2="6" y2="18" />
                      <line x1="6" y1="6" x2="18" y2="18" />
                    </svg>
                  </button>
                </span>
              {/each}
              {#if aliasOpen}
                <input
                  bind:this={aliasField}
                  class="tag-input__field"
                  type="text"
                  bind:value={aliasDraft}
                  onkeydown={onAliasKeydown}
                  onblur={onAliasBlur}
                  placeholder={t('gameForm.placeholder.charAliases')}
                  aria-label={t('gameForm.field.charAliases')}
                  {disabled}
                />
              {:else}
                <button
                  type="button"
                  class="tag-input__entry"
                  bind:this={aliasEntry}
                  onclick={openAlias}
                  {disabled}
                >
                  <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                    <line x1="12" y1="5" x2="12" y2="19" />
                    <line x1="5" y1="12" x2="19" y2="12" />
                  </svg>
                  {t('gameForm.action.addAlias')}
                </button>
              {/if}
            </div>
          </div>
        </div>

        <TagField
          items={selected.traits ?? []}
          onItemsChange={(v) => {
            if (selectedIndex >= 0) {
              characters = characters.map((c, i) => (i === selectedIndex ? { ...c, traits: v } : c));
            }
          }}
          bind:draft={traitDraft}
          label={t('gameForm.field.charTraits')}
          browseText={t('gameForm.action.browseTraits')}
          placeholder={t('gameForm.placeholder.charTraits')}
          emptyText={t('gameForm.placeholder.noTraits')}
          removeText={t('gameForm.action.removeTrait')}
          resolve={resolveTrait}
          keyOf={(tr) => tr.id ?? tr.name}
          labelOf={traitLabel}
          groupOf={(tr) => tr.group_name || 'uncategorized'}
          groupOrder={TRAIT_GROUP_ORDER}
          groupLabel={(key) => t('detail.traitGroups.' + key)}
          onbrowse={() => onTraitBrowse?.()}
          {disabled}
        />

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
    scroll-snap-type: none;
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
    background: var(--bg-modal);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
  }

  .char-panel-body > .panel-grid {
    margin-bottom: var(--space-5);
  }

  .char-panel-body > .form-group--desc {
    margin-bottom: 0;
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

  .va-suggest {
    display: flex;
    flex-direction: column;
    margin-top: var(--space-1);
    background: var(--bg-raised);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .va-suggest__item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    color: var(--fg-secondary);
    text-align: left;
    background: none;
    border: none;
    cursor: pointer;
    transition:
      color var(--duration-fast) var(--ease-out),
      background var(--duration-fast) var(--ease-out);
  }

  .va-suggest__item:hover {
    color: var(--fg-default);
    background: var(--bg-hover);
  }

  .va-suggest__item + .va-suggest__item {
    border-top: 1px solid var(--border-default);
  }

  .va-suggest__orig {
    color: var(--fg-muted);
  }

  .va-edit {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    margin-top: var(--space-2);
    padding: var(--space-3);
    background: var(--bg-raised);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
  }

  .va-edit__head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .va-edit__title {
    font-size: var(--text-xs);
    font-weight: var(--fw-semibold);
    letter-spacing: 0.04em;
    color: var(--fg-muted);
  }

  .va-edit__close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    color: var(--fg-muted);
    background: none;
    border: none;
    border-radius: var(--radius-pill);
    cursor: pointer;
  }

  .va-edit__close:hover {
    color: var(--fg-default);
    background: var(--bg-hover);
  }

  .va-edit__row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
  }

  .va-edit__label {
    flex: none;
    font-size: var(--text-xs);
    color: var(--fg-muted);
  }

  .va-edit__field {
    flex: 1 1 120px;
    min-width: 0;
  }

  .va-edit__hint {
    margin: 0;
    font-size: var(--text-xs);
    color: var(--fg-muted);
  }

  .va-edit__targets {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }
</style>
