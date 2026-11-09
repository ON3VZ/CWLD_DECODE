<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';

  let askDelete = $state<number | null>(null);

  const hhmm = (ts: number) => new Date(ts * 1000).toISOString().slice(11, 16);
  function exch(q: { nr_sent: number; nr_rcvd: string; extra_rcvd: string; rst_rcvd: string }) {
    return [q.rst_rcvd, q.nr_rcvd, q.extra_rcvd].filter(Boolean).join(' ');
  }
</script>

<section class="card" aria-label={t('log.title')}>
  <div class="head">
    <h2>{t('log.title')}{#if app.testMode} <span class="testtag">{t('contest.test.on')}</span>{/if}</h2>
    <span class="muted count">{app.qsoCount}</span>
    <button class="link" onclick={() => (app.exportOpen = true)}>{t('log.export')}</button>
  </div>
  {#if app.recent.length === 0}
    <p class="muted">{t('log.empty')}</p>
  {:else}
    <ul>
      {#each app.recent as q (q.id)}
        <li>
          <span class="muted mono">{hhmm(q.ts)}</span>
          <strong class="mono call">{q.call}</strong>
          <span class="mono ex">{exch(q)}</span>
          <span class="act">
            {#if askDelete === q.id}
              <button class="link danger" onclick={() => { askDelete = null; app.deleteQso(q.id); }}>{t('log.delete.yes')}</button>
              <button class="link" onclick={() => (askDelete = null)}>{t('log.cancel')}</button>
            {:else}
              <button class="link" onclick={() => (app.editing = { ...q })} aria-label={t('log.edit')} title={t('log.edit')}>{t('log.edit')}</button>
              <button class="link" onclick={() => (askDelete = q.id)} aria-label={t('log.delete')} title={t('log.delete')}>×</button>
            {/if}
          </span>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .card { background: var(--panel); border: 1px solid var(--line); border-radius: 12px; padding: 10px 12px; display: flex; flex-direction: column; gap: 6px; }
  .head { display: flex; align-items: baseline; gap: 8px; }
  h2 { margin: 0; font-size: 13px; letter-spacing: 0.06em; text-transform: uppercase; color: var(--muted); flex: 1; }
  .count { font-size: 12px; }
  .testtag { color: var(--warn); }
  p { margin: 0; font-size: 12px; }
  ul { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 3px; }
  li { display: flex; gap: 8px; align-items: baseline; font-size: 13px; }
  .call { color: var(--call); }
  .ex { flex: 1; color: var(--muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .act { display: flex; gap: 8px; }
  .link { background: none; border: none; color: var(--accent); font-size: 12px; padding: 0; cursor: pointer; text-decoration: underline; }
  .link.danger { color: var(--live); }
</style>
