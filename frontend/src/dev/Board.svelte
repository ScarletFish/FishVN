<script>
  import Sidebar from '../components/Sidebar.svelte';
  import BackfillLog from '../components/backfill/BackfillLog.svelte';
  import Toaster from '../components/Toaster.svelte';
  import { toast } from '../lib/toast.svelte.js';
  import { t } from '../lib/i18n.js';
  import { backfillSummaryText } from '../lib/backfill-summary.js';

  const ICON_HOME =
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"></path><polyline points="9 22 9 12 15 12 15 22"></polyline></svg>';
  const ICON_BACKFILL =
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>';
  const ICON_STATS =
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><line x1="18" y1="20" x2="18" y2="10"></line><line x1="12" y1="20" x2="12" y2="4"></line><line x1="6" y1="20" x2="6" y2="14"></line></svg>';

  const ACCENTS = [
    { id: 'default', label: '绯红' },
    { id: 'amber', label: '琥珀' },
    { id: 'ocean', label: '海蓝' },
    { id: 'sakura', label: '樱粉' },
    { id: 'emerald', label: '翠绿' },
    { id: 'violet', label: '紫' },
  ];

  const NORMAL_REPORT = { total: 62, fetched: 58, unmatched: 2, failed: 2, cancelled: false };
  const CANCELLED_REPORT = { total: 62, fetched: 38, unmatched: 1, failed: 1, cancelled: true };

  const ITEMS = [
    { game_id: 'g1', name: '某作' },
    { game_id: 'g2', name: '另一部作品' },
  ];

  const clock = (i) => Date.now() - 60000 + i * 1000;

  const NORMAL_LOGS = [
    {
      seq: 1,
      event: {
        phase: 'resolve',
        run_id: 11,
        ts: clock(1),
        game_id: 'g1',
        detail: '「某作」命中 3 条，取 v123《某作》',
      },
    },
    { seq: 2, event: { phase: 'meta', run_id: 11, ts: clock(2), game_id: 'g1' } },
    { seq: 3, event: { phase: 'characters', run_id: 11, ts: clock(3), game_id: 'g1', detail: '3 位' } },
    { seq: 4, event: { phase: 'images', run_id: 11, ts: clock(4), game_id: 'g1', done: 8, total: 8 } },
    { seq: 5, event: { phase: 'backfill', run_id: 11, ts: clock(5), game_id: 'g1' } },
    { seq: 6, event: { phase: 'finished', run_id: 11, ts: clock(6) } },
  ];

  const CANCELLED_LOGS = [
    {
      seq: 1,
      event: {
        phase: 'resolve',
        run_id: 12,
        ts: clock(1),
        game_id: 'g2',
        detail: '「另一部作品」命中 1 条，取 v456《另一部作品》',
      },
    },
    { seq: 2, event: { phase: 'paused', run_id: 12, ts: clock(2), game_id: 'g2' } },
    { seq: 3, event: { phase: 'finished', run_id: 12, ts: clock(3) } },
  ];

  const NORMAL_STATUS = { run_id: 11, items: ITEMS, finished: NORMAL_REPORT };
  const CANCELLED_STATUS = { run_id: 12, items: ITEMS, finished: CANCELLED_REPORT };

  const BARS = [
    { id: 'idle', title: '无任务', dot: null, tip: undefined },
    { id: 'run', title: '在跑（脉冲）', dot: 'running', tip: '正在补全 3/62' },
    { id: 'done', title: '完成未读（静止）', dot: 'done', tip: backfillSummaryText(NORMAL_REPORT) },
  ];

  let theme = $state(document.documentElement.dataset.theme === 'dark' ? 'dark' : 'light');
  let accent = $state(document.documentElement.dataset.accent ?? 'default');
  let reduce = $state(false);

  $effect(() => {
    const root = document.documentElement;
    if (theme === 'dark') root.dataset.theme = 'dark';
    else delete root.dataset.theme;
  });

  $effect(() => {
    const root = document.documentElement;
    if (accent === 'default') delete root.dataset.accent;
    else root.dataset.accent = accent;
  });

  $effect(() => {
    const root = document.documentElement;
    if (reduce) root.dataset.reduceMotion = 'on';
    else delete root.dataset.reduceMotion;
  });

  function itemsOf(bar) {
    return [
      { id: 'home', label: '首页', icon: ICON_HOME },
      { id: 'backfill', label: '补全', icon: ICON_BACKFILL, dot: bar.dot, tip: bar.tip },
      { id: 'stats', label: '统计', icon: ICON_STATS },
    ];
  }

  function popNormal() {
    toast(`${t('mylist.backfillDoneToast')} · ${backfillSummaryText(NORMAL_REPORT)}`, 'success');
  }

  function popCancelled() {
    toast(
      `${t('mylist.backfillCancelledToast')} · ${backfillSummaryText(CANCELLED_REPORT)}`,
      'info',
    );
  }
</script>

<div class="board">
  <header class="board__head">
    <h1 class="board__title">资料补全 · 视觉状态板</h1>
    <p class="board__hint">
      只验证外观：圆点、提示、日志汇总与 toast。事件时机由单测覆盖，不连后端。
    </p>
  </header>

  <div class="board__controls">
    <div class="board__group">
      <span class="board__label">主题</span>
      <button
        type="button"
        class="board__chip"
        class:is-active={theme === 'light'}
        onclick={() => (theme = 'light')}
      >
        浅色
      </button>
      <button
        type="button"
        class="board__chip"
        class:is-active={theme === 'dark'}
        onclick={() => (theme = 'dark')}
      >
        深色
      </button>
    </div>

    <div class="board__group">
      <span class="board__label">强调色</span>
      {#each ACCENTS as item (item.id)}
        <button
          type="button"
          class="board__chip"
          class:is-active={accent === item.id}
          onclick={() => (accent = item.id)}
        >
          {item.label}
        </button>
      {/each}
    </div>

    <label class="board__check">
      <input type="checkbox" bind:checked={reduce} />
      减弱动效
    </label>
  </div>

  <section class="board__section">
    <h2 class="board__section-title">侧栏圆点</h2>
    <div class="board__bars">
      {#each BARS as bar (bar.id)}
        <div class="board__bar">
          <span class="board__bar-title">{bar.title}</span>
          <div class="board__bar-frame">
            <Sidebar items={itemsOf(bar)} current="home" />
          </div>
        </div>
      {/each}
    </div>
  </section>

  <section class="board__section">
    <h2 class="board__section-title">运行日志 · 汇总行</h2>
    <div class="board__logs">
      <div class="board__log">
        <span class="board__bar-title">正常完结</span>
        <div class="bf-panel">
          <BackfillLog logs={NORMAL_LOGS} status={NORMAL_STATUS} />
        </div>
      </div>
      <div class="board__log">
        <span class="board__bar-title">取消完结</span>
        <div class="bf-panel">
          <BackfillLog logs={CANCELLED_LOGS} status={CANCELLED_STATUS} />
        </div>
      </div>
    </div>
  </section>

  <section class="board__section">
    <h2 class="board__section-title">toast</h2>
    <div class="board__controls">
      <button type="button" class="board__chip" onclick={popNormal}>成功</button>
      <button type="button" class="board__chip" onclick={popCancelled}>取消</button>
    </div>
  </section>
</div>

<Toaster />
