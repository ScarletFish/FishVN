<script>
  import Icon from '../Icon.svelte';
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
          <Icon name="x" size="sm" />
        </button>
      </div>
    {/each}
    <button
      type="button"
      class="staff-rows__add"
      onclick={addStaffEntry}
      disabled={saving}
    >
      <Icon name="plus" size="sm" />
      {t('gameForm.action.addStaff')}
    </button>
  </div>
  <p class="form-hint">{t('gameForm.hint.staff')}</p>
</div>
