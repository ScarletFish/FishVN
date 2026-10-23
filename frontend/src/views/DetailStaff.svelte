<script>
  import { t } from '../lib/i18n.js';
  import { api } from '../lib/api.js';
  import { groupStaff } from '../lib/staff.js';
  import DetailSection from '../components/DetailSection.svelte';
  import { panelScrollbars } from '../lib/scrollbars.js';

  let { gameId } = $props();

  let staff = $state([]);

  $effect(() => {
    staff = [];
    let live = true;
    api
      .getGameMetadata(gameId)
      .then((meta) => {
        if (live) staff = meta?.staff ?? [];
      })
      .catch(() => {});
    return () => {
      live = false;
    };
  });

  const groups = $derived(groupStaff(staff));

  const COLLAPSED_COUNT = 8;
  let expandedLabels = $state([]);

  const toggleGroup = (label) => {
    expandedLabels = expandedLabels.includes(label)
      ? expandedLabels.filter((l) => l !== label)
      : [...expandedLabels, label];
  };
</script>

{#if groups.length}
  <DetailSection title={t('detail.staff')} count={staff.length}>
    <div class="detail-staff-scroll" use:panelScrollbars>
      <div class="detail-staff-grid">
        {#each groups as g (g.label)}
          <div class="detail-staff-group">
            <div class="detail-staff-role">{g.label}</div>
            <div class="detail-staff-names">
              {#each (expandedLabels.includes(g.label) ? g.names : g.names.slice(0, COLLAPSED_COUNT)) as person, i (i)}
                <div class="detail-staff-name" data-tooltip={person.tip || null}>
                  <span class="detail-staff-name-text">{person.name}</span>
                  {#if person.note}
                    <span class="detail-staff-note">{person.note}</span>
                  {/if}
                </div>
              {/each}
              {#if g.names.length > COLLAPSED_COUNT}
                <button
                  type="button"
                  class="detail-staff-toggle"
                  onclick={() => toggleGroup(g.label)}
                >
                  {
                    expandedLabels.includes(g.label)
                      ? t('detail.staffLess')
                      : t('detail.staffMore', { count: g.names.length - COLLAPSED_COUNT })
                  }
                </button>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    </div>
  </DetailSection>
{/if}
