<script>
  import { onMount, tick } from 'svelte';
  import SettingsNav from '../components/settings/SettingsNav.svelte';
  import AppearanceSection from './settings/AppearanceSection.svelte';
  import ContentSection from './settings/ContentSection.svelte';
  import StartupSection from './settings/StartupSection.svelte';
  import VndbSection from './settings/VndbSection.svelte';
  import BackupSection from './settings/BackupSection.svelte';
  import AboutSection from './settings/AboutSection.svelte';
  import { t } from '../lib/i18n.js';
  import { getScrollViewport } from '../lib/scrollbars.js';
  import { getReduceMotion } from '../lib/motion.js';
  import { SETTINGS_SECTIONS, pickActiveSection } from '../lib/settings-nav.js';

  let { scroll = 0, onnavigate } = $props();

  const SPY_RATIO = 0.25;
  const GOTO_PAD = 16;
  const GOTO_LOCK_MS = 700;
  const SPY_SETTLE_MS = 90;

  const sectionEls = {};
  let rootEl = $state(null);
  let active = $state(SETTINGS_SECTIONS[0].id);
  let frame = 0;
  let settleTimer = null;
  let lockUntil = 0;
  let pinnedId = null;
  let unbindScroll = null;

  const navSections = $derived(
    SETTINGS_SECTIONS.map((section) => ({ id: section.id, label: t(section.label) })),
  );

  function collectSections() {
    if (!rootEl) return;
    for (const section of SETTINGS_SECTIONS) {
      const node = rootEl.querySelector(`#settings-${section.id}`);
      if (node) sectionEls[section.id] = node;
    }
  }

  function computeActive() {
    const viewport = getScrollViewport();
    if (!viewport) return active;
    const base = viewport.getBoundingClientRect().top;
    const tops = SETTINGS_SECTIONS.map((section) => ({
      id: section.id,
      top: (sectionEls[section.id]?.getBoundingClientRect().top ?? Infinity) - base,
    }));
    const atBottom = viewport.scrollTop + viewport.clientHeight >= viewport.scrollHeight - 2;
    return pickActiveSection(tops, viewport.clientHeight * SPY_RATIO, atBottom) ?? active;
  }

  function settleActive() {
    if (pinnedId || performance.now() < lockUntil) return;
    const next = computeActive();
    if (next === active) {
      clearTimeout(settleTimer);
      return;
    }
    clearTimeout(settleTimer);
    settleTimer = setTimeout(() => {
      active = next;
    }, SPY_SETTLE_MS);
  }

  function onScroll() {
    if (pinnedId && performance.now() >= lockUntil) pinnedId = null;
    if (frame) return;
    frame = requestAnimationFrame(() => {
      frame = 0;
      settleActive();
    });
  }

  function onScrollEnd() {
    lockUntil = 0;
  }

  function gotoSection(id) {
    const node = sectionEls[id];
    const viewport = getScrollViewport();
    if (!node || !viewport) return;
    active = id;
    pinnedId = id;
    lockUntil = performance.now() + GOTO_LOCK_MS;
    const offset = node.getBoundingClientRect().top - viewport.getBoundingClientRect().top;
    viewport.scrollTo({
      top: viewport.scrollTop + offset - GOTO_PAD,
      behavior: getReduceMotion() ? 'auto' : 'smooth',
    });
  }

  async function restoreScroll() {
    await tick();
    if (!rootEl) return;
    const viewport = getScrollViewport();
    if (viewport && scroll > 0) viewport.scrollTop = scroll;
    active = computeActive();
  }

  onMount(() => {
    const viewport = getScrollViewport();
    if (viewport) {
      viewport.addEventListener('scroll', onScroll, { passive: true });
      viewport.addEventListener('scrollend', onScrollEnd);
      unbindScroll = () => {
        viewport.removeEventListener('scroll', onScroll);
        viewport.removeEventListener('scrollend', onScrollEnd);
      };
    }
    collectSections();
    active = computeActive();
    restoreScroll();

    return () => {
      unbindScroll?.();
      cancelAnimationFrame(frame);
      clearTimeout(settleTimer);
    };
  });
</script>

<div class="view settings" bind:this={rootEl}>
  <header class="view-header">
    <h1>{t('settings.title')}</h1>
  </header>

  <div class="settings__nav">
    <SettingsNav sections={navSections} {active} ongoto={gotoSection} />
  </div>

  <div class="settings__body">
    <AppearanceSection />
    <ContentSection />
    <StartupSection />
    <VndbSection {onnavigate} />
    <BackupSection />
    <AboutSection />
  </div>
</div>
