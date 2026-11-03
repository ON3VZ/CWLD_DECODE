<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
</script>

<section class="card">
  <h2>{t('signals.title')}</h2>
  {#if app.signals.length === 0}
    <p class="muted">{t('signals.empty')}</p>
  {/if}
  <ul>
    {#each app.signals as s (s.id)}
      <li>
        <button class="row" onclick={() => s.call && app.selectCall(s.call)}>
          <span class="mono call">{s.call || '?'}</span>
          <span class="muted mono">{s.offsetHz > 0 ? '+' : ''}{s.offsetHz} Hz</span>
          <span class="muted">{s.wpm} WPM</span>
          <span class="muted">{s.snr} dB</span>
        </button>
      </li>
    {/each}
  </ul>
</section>

<style>
  .card { background: var(--panel); border: 1px solid var(--line); border-radius: 12px; padding: 12px; }
  h2 { margin: 0 0 8px; font-size: 15px; letter-spacing: 0.06em; text-transform: uppercase; color: var(--muted); }
  ul { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 4px; }
  .row { width: 100%; display: grid; grid-template-columns: 1fr auto auto auto; gap: 8px; background: var(--panel2); border: 1px solid var(--line); border-radius: 8px; padding: 6px 8px; text-align: left; font-size: 13px; }
  .row:hover { border-color: var(--accent); }
  .call { color: var(--call); font-weight: 700; }
  p { margin: 0; }
</style>
