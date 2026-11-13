<script lang="ts">
  import { untrack } from 'svelte';
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
  import type { LogQso } from '../lib/types';
  import { bandOf, parseKhz, parseUtc } from '../lib/logbook';

  const orig = untrack(() => app.editing!);
  const iso = (ts: number) => new Date(ts * 1000).toISOString();

  let q = $state<LogQso>(untrack(() => ({ ...app.editing! })));
  let nr = $state(String(orig.nr_sent));
  let day = $state(orig.ts ? iso(orig.ts).slice(0, 10) : '');
  let time = $state(orig.ts ? iso(orig.ts).slice(11, 19) : '');
  let khz = $state(orig.freq_hz ? String(orig.freq_hz / 1000) : '');
  // Empty means: listened where we transmitted. Only a split QSO has its own receive frequency.
  let rxKhz = $state(orig.freq_rx_hz && orig.freq_rx_hz !== orig.freq_hz ? String(orig.freq_rx_hz / 1000) : '');
  let confirming = $state(false);
  let problem = $state('');

  const newHz = $derived(parseKhz(khz));
  const newBand = $derived(Number.isNaN(newHz) ? '' : bandOf(newHz));
  const newRxHz = $derived(parseKhz(rxKhz));

  function check(): LogQso | null {
    const ts = parseUtc(day, time);
    if (Number.isNaN(ts)) {
      problem = t('log.edit.bad_time');
      return null;
    }
    if (Number.isNaN(newHz) || Number.isNaN(newRxHz)) {
      problem = t('log.edit.bad_freq');
      return null;
    }
    if (!q.call.trim()) {
      problem = t('log.nocall');
      return null;
    }
    problem = '';
    return {
      ...$state.snapshot(q),
      call: q.call.trim().toUpperCase(),
      nr_sent: Math.max(0, Math.round(Number(nr) || 0)),
      ts,
      freq_hz: newHz,
      freq_rx_hz: newRxHz === newHz ? 0 : newRxHz,
      band: newBand || q.band,
    } as LogQso;
  }

  function save() {
    const out = check();
    if (!out) return;
    // A changed frequency (and with it the band) always needs a second, explicit yes.
    if (out.freq_hz !== orig.freq_hz && !confirming) {
      confirming = true;
      return;
    }
    app.saveEdit(out);
  }

  const fmt = (hz: number) => (hz ? `${(hz / 1000).toFixed(1)} kHz` : t('log.edit.none'));
</script>

<div class="scrim" role="presentation">
  <div class="dlg" role="dialog" aria-modal="true" aria-labelledby="le-title">
    <h2 id="le-title">{t('log.edit')}</h2>
    {#if confirming}
      <div class="confirm" role="alertdialog" aria-labelledby="le-ask">
        <strong id="le-ask">{t('log.edit.freq_ask')}</strong>
        <p>{t('log.edit.freq_change', { from: fmt(orig.freq_hz), to: fmt(newHz) })}</p>
        <p>{newBand && newBand !== orig.band ? t('log.edit.band_change', { from: orig.band || t('log.edit.none'), to: newBand }) : newBand ? t('log.edit.band_same', { band: newBand }) : t('log.edit.band_out', { band: orig.band || t('log.edit.none') })}</p>
        <p class="muted">{t('log.edit.exported_again')}</p>
      </div>
      <div class="act">
        <button class="btn" onclick={() => (confirming = false)}>{t('log.edit.back')}</button>
        <button class="btn primary" onclick={save}>{t('log.edit.freq_yes')}</button>
      </div>
    {:else}
      <div class="grid">
        <label>{t('qso.call')}<input class="mono" bind:value={q.call} /></label>
        <label>{t('book.col.mode')}<input class="mono" bind:value={q.mode} /></label>
        <label>{t('book.col.date')}<input class="mono" bind:value={day} placeholder="2026-11-12" /></label>
        <label>{t('book.col.time')} (UTC)<input class="mono" bind:value={time} placeholder="14:05:00" /></label>
        <label>{t('book.col.freq')}<input class="mono" bind:value={khz} inputmode="decimal" /></label>
        <label title={t('log.edit.rx.help')}>{t('book.col.freq_rx')}<input class="mono" bind:value={rxKhz} inputmode="decimal" placeholder={t('log.edit.rx.same')} title={t('log.edit.rx.help')} /></label>
        <label>{t('book.col.band')}<input class="mono" value={newBand || q.band} disabled /></label>
        <label>{t('log.rst_sent')}<input class="mono" bind:value={q.rst_sent} /></label>
        <label>{t('contest.rcvd.rst')}<input class="mono" bind:value={q.rst_rcvd} /></label>
        <label>{t('log.nr_sent')}<input class="mono" bind:value={nr} inputmode="numeric" /></label>
        <label>{t('contest.rcvd.nr')}<input class="mono" bind:value={q.nr_rcvd} /></label>
        <label>{t('log.extra_sent')}<input class="mono" bind:value={q.extra_sent} /></label>
        <label>{t('log.extra_rcvd')}<input class="mono" bind:value={q.extra_rcvd} /></label>
        <label>{t('qso.name')}<input bind:value={q.name} /></label>
        <label>{t('qso.qth')}<input bind:value={q.qth} /></label>
        <label>{t('log.edit.their_prog')}<input class="mono" bind:value={q.their_prog} /></label>
        <label>{t('book.col.their_ref')}<input class="mono" bind:value={q.their_ref} /></label>
        <label class="wide">{t('book.col.comment')}<input bind:value={q.comment} /></label>
      </div>
      <details>
        <summary>{t('log.edit.mine')}</summary>
        <div class="grid">
          <label>{t('book.col.station')}<input class="mono" bind:value={q.station_call} /></label>
          <label>{t('book.col.operator')}<input class="mono" bind:value={q.operator} /></label>
          <label>{t('book.col.my_grid')}<input class="mono" bind:value={q.my_grid} /></label>
          <label>{t('book.col.power')}<input class="mono" bind:value={q.tx_pwr} /></label>
          <label>{t('book.col.rig')}<input bind:value={q.my_rig} /></label>
          <label>{t('book.col.antenna')}<input bind:value={q.my_antenna} /></label>
          <label>WWFF<input class="mono" bind:value={q.my_wwff} /></label>
          <label>POTA<input class="mono" bind:value={q.my_pota} /></label>
          <label>SOTA<input class="mono" bind:value={q.my_sota} /></label>
        </div>
      </details>
      {#if problem}<p class="problem" role="alert">{problem}</p>{/if}
      <div class="act">
        <button class="btn" onclick={() => (app.editing = null)}>{t('log.cancel')}</button>
        <button class="btn primary" onclick={save}>{t('log.save')}</button>
      </div>
    {/if}
  </div>
</div>

<style>
  .scrim { position: fixed; inset: 0; background: rgba(0, 0, 0, 0.6); display: grid; place-items: center; z-index: 30; }
  .dlg { background: var(--panel); border: 1px solid var(--line); border-radius: 14px; padding: 18px; width: min(600px, 94vw); max-height: 92vh; overflow: auto; display: flex; flex-direction: column; gap: 12px; }
  h2 { margin: 0; font-size: 18px; }
  .grid { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
  .wide { grid-column: 1 / -1; }
  label { display: flex; flex-direction: column; font-size: 12px; color: var(--muted); gap: 2px; }
  input { background: var(--panel2); border: 1px solid var(--line); border-radius: 8px; padding: 6px 8px; font-size: 16px; color: var(--text); }
  input:disabled { opacity: 0.6; }
  details { border-top: 1px solid var(--line); padding-top: 8px; }
  summary { cursor: pointer; font-size: 13px; color: var(--muted); margin-bottom: 8px; }
  .confirm { border: 1px solid var(--warn); border-radius: 10px; padding: 12px; display: flex; flex-direction: column; gap: 6px; }
  .confirm p { margin: 0; }
  .muted { color: var(--muted); font-size: 13px; }
  .problem { color: var(--warn); margin: 0; font-size: 14px; }
  .act { display: flex; gap: 8px; justify-content: flex-end; }
</style>
