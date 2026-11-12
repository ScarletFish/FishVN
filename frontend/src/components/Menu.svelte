<script>
  import Icon from './Icon.svelte';
  import { DropdownMenu } from 'bits-ui';

  let { items = [], label, variant = 'btn', open = $bindable(false), trigger = null, align = 'end' } = $props();
</script>

<div class="menu" data-variant={variant} class:is-open={open}>
  <DropdownMenu.Root bind:open>
    <DropdownMenu.Trigger>
      {#snippet child({ props })}
        {#if trigger}
          {@render trigger(props)}
        {:else}
          <button
            {...props}
            type="button"
            class={variant === 'plain' ? 'menu__trigger' : 'menu__trigger btn btn-ghost'}
            aria-label={label}
          >
            <Icon name="dots-three" size="lg" />
          </button>
        {/if}
      {/snippet}
    </DropdownMenu.Trigger>
    <DropdownMenu.Portal to="#modal-root">
      <DropdownMenu.Content class="menu__list" align={align} sideOffset={4}>
        {#each items as item (item.label)}
          <DropdownMenu.Item class="menu__item" onSelect={() => item.onclick()}>
            {item.label}
          </DropdownMenu.Item>
        {/each}
      </DropdownMenu.Content>
    </DropdownMenu.Portal>
  </DropdownMenu.Root>
</div>
