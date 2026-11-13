<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
  import PlaceLine from './PlaceLine.svelte';

  function typed() {
    app.qso.call = app.qso.call.toUpperCase();
    app.callLocked = true;
    // Same path as a click: dupe check and (after a short pause) the QRZ.com lookup.
    app.noteCallChange();
  }
</script>

<section class="card">
  <h2>{t('qso.title')}</h2>
  <label title={app.qso.call ? undefined : t('qso.empty')}>
    {t('qso.call')}
    <input class="mono" bind:value={app.qso.call} oninput={typed} title={app.qso.call ? undefined : t('qso.empty')} />
  </label>
  <div class="mode" class:locked={app.callLocked}>
    {#if app.callLocked}
      <span>{t('qso.locked')}</span>
      <button class="link" onclick={() => app.unlockCall()}>{t('qso.unlock')}</button>
    {:else}
      <span class="auto">{t('qso.auto')}</span>
    {/if}
  </div>
  <PlaceLine />
  <label>{t('qso.name')}<input bind:value={app.qso.name} /></label>
  <label>{t('qso.qth')}<input bind:value={app.qso.qth} /></label>
  {#if app.lookupNote && app.lookupNote.call === app.qso.call.trim().toUpperCase()}<small class="muted lookup-note">{t('lookup.from', { text: app.lookupNote.text })}</small>{/if}
  <div class="pair">
    <label>{t('qso.rst')}<input class="mono" bind:value={app.qso.rst} /></label>
    {#if !app.settings?.contest_mode}<label>{t('contest.rcvd.rst')}<input class="mono" bind:value={app.entry.rst} /></label>{/if}
  </div>
  {#if app.activation && !app.settings?.contest_mode}
    <label>{t('qso.theirref')}
      <span class="refrow">
        <select bind:value={app.theirRef.prog} aria-label={t('qso.theirref')}><option>WWFF</option><option>POTA</option><option>SOTA</option></select>
        <input class="mono" bind:value={app.theirRef.ref} placeholder={app.theirRef.prog === 'WWFF' ? 'ONFF-0123' : ''} />
      </span>
    </label>
  {/if}
  {#if !app.settings?.contest_mode}
    <button class="btn primary" onclick={() => app.logQso()} disabled={!app.qso.call.trim()} title={t('log.qso.help')}>{t('contest.log')}</button>
  {/if}
  <p class="muted small">{t('qso.hint')}</p>
</section>

<style>
  .card { background: var(--panel); border: 1px solid var(--line); border-radius: 12px; padding: 12px; display: flex; flex-direction: column; gap: 8px; }
  h2 { margin: 0; font-size: 15px; letter-spacing: 0.06em; text-transform: uppercase; color: var(--muted); }
  p { margin: 0; font-size: 13px; }
  .small { font-size: 12px; }
  label { display: flex; flex-direction: column; font-size: 12px; color: var(--muted); gap: 2px; }
  input { background: var(--panel2); border: 1px solid var(--line); border-radius: 8px; padding: 6px 8px; font-size: 16px; color: var(--text); }
  .pair { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
  .pair label { min-width: 0; }
  .pair input { min-width: 0; width: 100%; box-sizing: border-box; }
  .refrow { display: grid; grid-template-columns: 5.5em 1fr; gap: 6px; }
  select { background: var(--panel2); border: 1px solid var(--line); border-radius: 8px; padding: 6px 8px; font-size: 14px; color: var(--text); }
  .mode { display: flex; gap: 8px; align-items: center; font-size: 12px; color: var(--muted); margin-top: -4px; }
  .auto { color: var(--listen); font-weight: 600; }
  .locked span { color: var(--warn); font-weight: 600; }
  .link { background: none; border: none; padding: 0; color: var(--accent); text-decoration: underline; font-size: 12px; }
</style>
