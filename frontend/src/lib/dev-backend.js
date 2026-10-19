let ready = null;

async function wait() {
  for (;;) {
    try {
      const res = await fetch('/__dev_backend_ready', { cache: 'no-store' });
      if (res.ok) return;
    } catch {}
    await new Promise((resolve) => setTimeout(resolve, 300));
  }
}

export function ensureBackend() {
  ready ??= wait();
  return ready;
}
