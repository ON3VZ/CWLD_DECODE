<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
  import ModeSwitch from './ModeSwitch.svelte';
  import ConfidenceMeter from './ConfidenceMeter.svelte';

  const mhz = $derived((app.status.frequency_hz / 1_000_000).toFixed(6));
  const sUnits = $derived(Math.min(9, Math.round((app.status.s_meter / 100) * 9 * 1.0)));
  let editing = $state(false);
  // A UTC clock, because logbooks and contests run on UTC.
  let utc = $state(new Date().toISOString().slice(11, 19));
  $effect(() => {
    const id = setInterval(() => (utc = new Date().toISOString().slice(11, 19)), 1000);
    return () => clearInterval(id);
  });
  /** A real radio is set up, it does not answer now, and CWLD tries again by itself. */
  const searching = $derived(!app.status.connected && !!app.settings && app.settings.radio.kind !== 'simulator' && app.settings.radio.auto_connect && !!app.settings.radio.port);
  /** The radio type is still Simulator: nothing real is connected, whatever the status says. */
  const simulated = $derived(!!app.settings && app.settings.radio.kind === 'simulator' && app.status.connected);
  let value = $state('');

  function startEdit() {
    value = (app.status.frequency_hz / 1000).toFixed(1);
    editing = true;
  }
  async function commit() {
    const khz = parseFloat(value.replace(',', '.'));
    editing = false;
    if (!Number.isNaN(khz)) await app.setFrequency(Math.round(khz * 1000));
  }
</script>

<header class="bar">
  <div class="brand">{t('app.name')}</div>
  <div class="freq" aria-label={t('status.frequency')}>
    {#if editing}
      <input class="mono edit" bind:value onblur={commit} onkeydown={(e) => e.key === 'Enter' && commit()} aria-label={t('status.frequency')} />
      <span class="muted">kHz</span>
    {:else}
      <button class="mono f" onclick={startEdit} title={t('status.frequency')}>{mhz}</button>
      <span class="muted">MHz</span>
    {/if}
    {#if app.split.on}
      <span class="splitchip on mono" title={t('split.chip.on.help')}>SPLIT TX {app.split.tx_hz !== null ? (app.split.tx_hz / 1000).toFixed(2) : '?'}</span>
    {:else}
      <button class="splitchip" class:open={app.splitOpen} onclick={() => { app.splitEnded = null; app.splitOpen = !app.splitOpen; }} title={t('split.chip.help')} disabled={!app.status.connected}>SPLIT</button>
    {/if}
  </div>
  <div class="info">
    {#if app.settings && app.settings.profiles.length > 1}
      <select class="chip prof" aria-label={t('profile.station')} title={app.activation ? t('profile.locked') : t('profile.station')} disabled={!!app.activation} value={app.settings.profile} onchange={(e) => app.setProfile((e.currentTarget as HTMLSelectElement).value)}>
        {#each app.settings.profiles as p (p.id)}<option value={p.id}>{p.name} {app.settings.station.calls[0] ? `(${(app.settings.station.calls[0] + p.suffix).toUpperCase()})` : ''}</option>{/each}
      </select>
    {/if}
    <span class="chip">{app.status.mode.toUpperCase()}</span>
    <span class="chip">{app.status.cw_wpm} WPM</span>
    <span class="chip" class:tx={app.status.transmitting}>{app.status.transmitting ? t('status.tx') : t('status.rx')}</span>
    <span class="chip" title={t('status.smeter')}>S{sUnits}</span>
    <ConfidenceMeter />
    {#if app.audio?.running && !app.demo}
      <span class="chip sig" class:on={app.signalPresent} title={t('signal.help')}><span class="led"></span>{app.signalPresent ? t('signal.yes') : t('signal.no')}</span>
    {/if}
    {#if app.audio?.running}
      <button class="chip rec" class:on={app.audio.recording} onclick={() => app.toggleRecording()} title={app.audio.recording ? t('rec.stop.help') : t('rec.start.help')}>
        {app.audio.recording ? `■ ${t('rec.stop')} ${Math.floor(app.audio.record_seconds / 60)}:${String(Math.floor(app.audio.record_seconds % 60)).padStart(2, '0')}` : `● ${t('rec.start')}`}
      </button>
    {:else if app.audio && !app.demo}
      <button class="chip warn" onclick={() => (app.settingsOpen = true)} title={app.audio.error ?? ''}>{t('audio.off')}</button>
    {/if}
    {#if app.swr !== null && app.status.transmitting}
      <span class="chip swr" class:bad={app.swr > (app.settings?.swr.limit ?? 1.5)}>{t('swr.chip', { swr: app.swr.toFixed(1) })}</span>
    {/if}
  </div>
  <div class="utc mono" title={t('status.utc.help')}>{utc}<span class="muted"> UTC</span></div>
  <div class="conn" class:off={!app.status.connected} class:search={searching} class:sim={simulated} title={simulated ? t('status.simulator.help') : searching ? t('status.searching.help') : ''}>
    <span class="dot"></span>
    {simulated ? t('status.simulator') : app.status.connected ? t('status.connected') : searching ? t('status.searching') : t('status.disconnected')}
    {#if !simulated}<span class="muted">{app.radioName}</span>{/if}
  </div>
  <ModeSwitch />
  <button class="btn" onclick={() => (app.bookOpen = true)} title={t('book.open.help')}>{t('book.open')}</button>
  <button class="btn" onclick={() => (app.settingsOpen = true)} aria-label={t('settings.title')}>⚙</button>
</header>

<style>
  .prof { background: var(--panel2); border: 1px solid var(--line); color: var(--text); max-width: 15em; }
  .bar { display: flex; align-items: center; gap: 16px; padding: 10px 16px; background: var(--panel); border-bottom: 1px solid var(--line); flex-wrap: wrap; }
  .brand { font-weight: 800; letter-spacing: 0.08em; color: var(--accent); }
  .freq { display: flex; align-items: baseline; gap: 6px; }
  .f { background: none; border: none; font-size: 26px; font-weight: 600; padding: 0; }
  .f:hover { color: var(--accent); }
  .edit { font-size: 22px; width: 9em; background: var(--panel2); border: 1px solid var(--accent); border-radius: 6px; padding: 2px 6px; }
  .splitchip { align-self: center; margin-left: 6px; font-size: 12px; font-weight: 700; letter-spacing: 0.06em; padding: 2px 9px; border-radius: 6px; border: 1px solid var(--line); background: var(--panel2); color: var(--muted); }
  button.splitchip { cursor: pointer; }
  button.splitchip:hover:not(:disabled), .splitchip.open { color: var(--text); border-color: var(--accent); }
  .splitchip.on { background: var(--warn); border-color: var(--warn); color: #1a1205; }
  .info { display: flex; gap: 6px; }
  .chip { background: var(--panel2); border: 1px solid var(--line); border-radius: 999px; padding: 2px 10px; font-size: 13px; }
  .chip.sig { display: inline-flex; align-items: center; gap: 6px; color: var(--muted); }
  .chip.sig.on { color: var(--text); }
  .led { width: 8px; height: 8px; border-radius: 50%; background: var(--muted); opacity: 0.5; }
  .sig.on .led { background: var(--listen); opacity: 1; }
  .chip.swr.bad { background: #b3141a; color: #fff; border-color: #b3141a; font-weight: 700; }
  .chip.rec { cursor: pointer; color: var(--text); }
  .chip.rec.on { background: #b3141a; color: #fff; border-color: #b3141a; font-weight: 700; }
  .chip.warn { cursor: pointer; border-color: #d98e04; color: #d98e04; }
  .chip.tx { background: var(--live); color: #fff; border-color: var(--live); font-weight: 700; }
  .utc { margin-left: auto; font-size: 15px; white-space: nowrap; }
  .conn { display: flex; align-items: center; gap: 6px; font-size: 14px; }
  .dot { width: 9px; height: 9px; border-radius: 50%; background: var(--listen); }
  .conn.off .dot { background: var(--muted); }
  .conn.sim .dot { background: var(--warn); }
  .conn.sim { color: var(--warn); font-weight: 600; }
  .conn.search .dot { background: var(--warn); animation: pulse 1.4s ease-in-out infinite; }
  @keyframes pulse { 50% { opacity: 0.3; } }
  @media (prefers-reduced-motion: reduce) { .conn.search .dot { animation: none; } }
</style>
