import { onMount } from 'svelte';

export function useStat(fetcher) {
  let data = $state(null);
  let loading = $state(true);
  let error = $state(null);

  async function load() {
    try {
      data = await fetcher();
      error = null;
    } catch (e) {
      error = e?.message ?? String(e);
    } finally {
      loading = false;
    }
  }

  onMount(load);

  return {
    get data() {
      return data;
    },
    get loading() {
      return loading;
    },
    get error() {
      return error;
    },
    refresh: load,
  };
}
