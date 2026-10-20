import { onMount } from 'svelte';

export function useStat(fetcher) {
  let data = $state(null);
  let loading = $state(true);
  let error = $state(null);

  onMount(async () => {
    try {
      data = await fetcher();
    } catch (e) {
      error = e?.message ?? String(e);
    } finally {
      loading = false;
    }
  });

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
  };
}
