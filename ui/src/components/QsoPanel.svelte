<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';

  function typed() {
    app.qso.call = app.qso.call.toUpperCase();
    app.callLocked = true;
  }
</script>

<section class="card">
  <h2>{t('qso.title')}</h2>
  {#if !app.qso.call}
    <p class="muted">{t('qso.empty')}</p>
  {/if}
  <label>
    {t('qso.call')}
    <input class="mono" bind:value={app.qso.call} oninput={typed} />
  </label>
  <div class="mode" class:locked={app.callLocked}>
    {#if app.callLocked}
      <span>{t('qso.locked')}</span>
      <button class="link" onclick={() => app.unlockCall()}>{t('qso.unlock')}</button>
    {:else}
      <span class="auto">{t('qso.auto')}</span>
    {/if}
  </div>
  <label>{t('qso.name')}<input bind:value={app.qso.name} /></label>
  <label>{t('qso.rst')}<input class="mono" bind:value={app.qso.rst} /></label>
  <label>{t('qso.qth')}<input bind:value={app.qso.qth} /></label>
  <p class="muted small">{t('qso.hint')}</p>
</section>

<style>
  .card { background: var(--panel); border: 1px solid var(--line); border-radius: 12px; padding: 12px; display: flex; flex-direction: column; gap: 8px; }
  h2 { margin: 0; font-size: 15px; letter-spacing: 0.06em; text-transform: uppercase; color: var(--muted); }
  p { margin: 0; font-size: 13px; }
  .small { font-size: 12px; }
  label { display: flex; flex-direction: column; font-size: 12px; color: var(--muted); gap: 2px; }
  input { background: var(--panel2); border: 1px solid var(--line); border-radius: 8px; padding: 6px 8px; font-size: 16px; color: var(--text); }
  .mode { display: flex; gap: 8px; align-items: center; font-size: 12px; color: var(--muted); margin-top: -4px; }
  .auto { color: var(--listen); font-weight: 600; }
  .locked span { color: var(--warn); font-weight: 600; }
  .link { background: none; border: none; padding: 0; color: var(--accent); text-decoration: underline; font-size: 12px; }
</style>
