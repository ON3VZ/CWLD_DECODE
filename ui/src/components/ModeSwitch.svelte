<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
  import type { TxPolicy } from '../lib/types';

  const modes: Array<{ id: TxPolicy; key: string; hint: string }> = [
    { id: 'live', key: 'mode.live', hint: 'mode.live.hint' },
    { id: 'listen_only', key: 'mode.listen', hint: 'mode.listen.hint' },
    { id: 'practice', key: 'mode.practice', hint: 'mode.practice.hint' },
  ];
</script>

<div class="switch" role="radiogroup" aria-label="Mode">
  {#each modes as m (m.id)}
    <button
      role="radio"
      aria-checked={app.policy === m.id}
      class="seg {m.id}"
      class:active={app.policy === m.id}
      title={t(m.hint)}
      onclick={() => app.requestPolicy(m.id)}>{t(m.key)}</button>
  {/each}
</div>

<style>
  .switch { display: inline-flex; border: 1px solid var(--line); border-radius: 10px; overflow: hidden; }
  .seg { background: var(--panel2); border: none; padding: 7px 14px; font-weight: 600; font-size: 13px; letter-spacing: 0.04em; color: var(--muted); }
  .seg + .seg { border-left: 1px solid var(--line); }
  .seg.active.live { background: var(--live); color: #fff; }
  .seg.active.listen_only { background: var(--listen); color: #04170f; }
  .seg.active.practice { background: #8b6cf0; color: #fff; }
  .seg:not(.active):hover { color: var(--text); }
</style>
