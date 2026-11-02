<script>
  import { t } from '../../lib/i18n.js';
  import { STAFF_ROLES, staffRoleLabel } from '../../lib/staff.js';
  import SelectDd from '../SelectDd.svelte';

  let {
    staff = $bindable([]),
    staffRoleOpen = $bindable(null),
    saving = false,
  } = $props();

  const STAFF_ROLE_OPTIONS = STAFF_ROLES.map((role) => ({
    value: role,
    label: staffRoleLabel(role),
  }));

  function addStaffEntry() {
    staff = [...staff, { role: 'scenario', name: '', original: '', note: null }];
  }

  function removeStaffEntry(index) {
    staff = staff.filter((_, i) => i !== index);
  }
</script>

<div class="form-group panel-grid__full">
  <label for="game-form-staff">{t('gameForm.field.staff')}</label>
  <div class="staff-rows">
    {#each staff as person, i (i)}
      <div class="staff-row">
        <SelectDd
          options={STAFF_ROLE_OPTIONS}
          value={person.role}
          open={staffRoleOpen === i}
          onopenchange={(v) => (staffRoleOpen = v ? i : null)}
          onchange={(v) => (person.role = v)}
          ariaLabel={t('gameForm.field.staffRole')}
          disabled={saving}
        />
        <input
          type="text"
          bind:value={person.original}
          placeholder={t('gameForm.placeholder.staffOriginal')}
          disabled={saving}
          aria-label={t('gameForm.field.staffOriginal')}
        />
        <input
          type="text"
          bind:value={person.name}
          placeholder={t('gameForm.placeholder.staffName')}
          disabled={saving}
          aria-label={t('gameForm.field.staffAlias')}
        />
        <button
          type="button"
          class="staff-row__remove"
          aria-label={t('gameForm.action.removeStaff')}
          onmousedown={(e) => e.preventDefault()}
          onclick={() => removeStaffEntry(i)}
          disabled={saving}
        >
          <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      </div>
    {/each}
    <button
      type="button"
      class="staff-rows__add"
      onclick={addStaffEntry}
      disabled={saving}
    >
      <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
        <line x1="12" y1="5" x2="12" y2="19" />
        <line x1="5" y1="12" x2="19" y2="12" />
      </svg>
      {t('gameForm.action.addStaff')}
    </button>
  </div>
  <p class="form-hint">{t('gameForm.hint.staff')}</p>
</div>
