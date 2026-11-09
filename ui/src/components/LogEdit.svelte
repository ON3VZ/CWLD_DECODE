<script lang="ts">
  import { untrack } from 'svelte';
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
  import type { LogQso } from '../lib/types';

  let q = $state<LogQso>(untrack(() => ({ ...app.editing! })));
  let nr = $state(String(untrack(() => app.editing!.nr_sent)));

  function save() {
    app.saveEdit({ ...$state.snapshot(q), call: q.call.trim().toUpperCase(), nr_sent: Math.max(0, Math.round(Number(nr) || 0)) } as LogQso);
  }
</script>

<div class="scrim" role="presentation">
  <div class="dlg" role="dialog" aria-modal="true" aria-labelledby="le-title">
    <h2 id="le-title">{t('log.edit')}</h2>
    <div class="grid">
      <label>{t('qso.call')}<input class="mono" bind:value={q.call} /></label>
      <label>{t('log.rst_sent')}<input class="mono" bind:value={q.rst_sent} /></label>
      <label>{t('contest.rcvd.rst')}<input class="mono" bind:value={q.rst_rcvd} /></label>
      <label>{t('log.nr_sent')}<input class="mono" bind:value={nr} inputmode="numeric" /></label>
      <label>{t('contest.rcvd.nr')}<input class="mono" bind:value={q.nr_rcvd} /></label>
      <label>{t('log.extra_rcvd')}<input class="mono" bind:value={q.extra_rcvd} /></label>
      <label>{t('qso.name')}<input bind:value={q.name} /></label>
      <label>{t('qso.qth')}<input bind:value={q.qth} /></label>
    </div>
    <div class="act">
      <button class="btn" onclick={() => (app.editing = null)}>{t('log.cancel')}</button>
      <button class="btn primary" onclick={save}>{t('log.save')}</button>
    </div>
  </div>
</div>

<style>
  .scrim { position: fixed; inset: 0; background: rgba(0, 0, 0, 0.6); display: grid; place-items: center; z-index: 30; }
  .dlg { background: var(--panel); border: 1px solid var(--line); border-radius: 14px; padding: 18px; width: min(520px, 94vw); display: flex; flex-direction: column; gap: 12px; }
  h2 { margin: 0; font-size: 18px; }
  .grid { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
  label { display: flex; flex-direction: column; font-size: 12px; color: var(--muted); gap: 2px; }
  input { background: var(--panel2); border: 1px solid var(--line); border-radius: 8px; padding: 6px 8px; font-size: 16px; color: var(--text); }
  .act { display: flex; gap: 8px; justify-content: flex-end; }
</style>
