<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';

  // In practice mode the keyed text is known, so the real accuracy is shown. Otherwise the
  // decoder's own estimate is shown.
  const real = $derived(app.decoder?.accuracy != null);
  const value = $derived(app.decoder ? (app.decoder.accuracy ?? app.decoder.confidence) : null);
  const pct = $derived(value === null ? 0 : Math.round(value * 100));
  const level = $derived(value === null ? 'none' : value >= 0.9 ? 'good' : value >= 0.7 ? 'fair' : 'poor');
  const snr = $derived(app.decoder?.snr_db != null ? Math.round(app.decoder.snr_db) : null);
  const label = $derived(real ? t('meter.accuracy') : t('meter.confidence'));
  const title = $derived(real ? t('meter.accuracy.help') : t('meter.confidence.help'));
</script>

<div class="meter {level}" {title} role="meter" aria-label={label} aria-valuemin="0" aria-valuemax="100" aria-valuenow={value === null ? undefined : pct}>
  <span class="lbl">{label}</span>
  <span class="bar"><span class="fill" style:width={`${pct}%`}></span></span>
  <span class="num mono">{value === null ? t('meter.none') : `${pct}%`}</span>
  {#if snr !== null}<span class="snr muted">{t('meter.snr', { snr })}</span>{/if}
</div>

<style>
  .meter { display: inline-flex; align-items: center; gap: 6px; font-size: 13px; background: var(--panel2); border: 1px solid var(--line); border-radius: 999px; padding: 2px 10px; }
  .lbl { color: var(--muted); }
  .bar { width: 64px; height: 8px; border-radius: 4px; background: var(--line); overflow: hidden; display: inline-block; }
  .fill { display: block; height: 100%; background: var(--muted); transition: width 0.4s ease, background 0.4s ease; }
  .good .fill { background: #2e9e5b; }
  .fair .fill { background: #d98e04; }
  .poor .fill { background: #c0392b; }
  .num { min-width: 3em; text-align: right; }
  .snr { font-size: 12px; }
</style>
