<script lang="ts">
  import { untrack } from 'svelte';
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
  import Waterfall from './Waterfall.svelte';

  // The peak meter keeps the highest value a little while, like a real level meter.
  let held = $state(0);
  $effect(() => {
    const p = app.audioPeak;
    held = Math.max(p, untrack(() => held) * 0.9);
  });
  const pct = $derived(Math.min(100, held * 100));
  /** 5 to 90 percent of the full range is good, above 90 percent the sound card input is overloaded. */
  const zone = $derived(pct > 90 ? 'high' : pct < 5 ? 'low' : 'ok');
  const gain = $derived(app.settings?.audio.gain_db ?? 0);
  const wide = $derived(app.settings?.audio.wide_filter ?? false);

  // The tuning buttons are tucked away by default. The choice is remembered on this computer.
  const KEY = 'cwld.audio.tools';
  let tools = $state(read());
  function read(): boolean {
    try {
      return typeof localStorage !== 'undefined' && localStorage.getItem(KEY) === '1';
    } catch {
      return false;
    }
  }
  function toggle() {
    tools = !tools;
    try {
      localStorage.setItem(KEY, tools ? '1' : '0');
    } catch {
      // not remembered, still works
    }
  }
</script>

<section class="card">
  <div class="head">
    <h2>{t('audio.panel.title')}</h2>
    <button class="link" onclick={() => app.setView('audio_waterfall', false)}>{t('audio.panel.hide')}</button>
  </div>
  <div class="frame"><Waterfall height={110} /></div>
  <small class="muted">{t('audio.panel.help')}</small>

  {#if app.audio?.running}
    <div class="peak" title={t('meter.peak.help')}>
      <span class="label">{t('meter.peak')}</span>
      <span class="bar" role="meter" aria-valuemin="0" aria-valuemax="100" aria-valuenow={Math.round(pct)} aria-label={t('meter.peak')}>
        <span class="zone lo"></span><span class="zone hi"></span>
        <span class="fill {zone}" style:width={`${pct}%`}></span>
      </span>
      <span class="mono val {zone}">{Math.round(pct)}%</span>
    </div>
    <small class:warn={zone !== 'ok'} class="muted">{t(`meter.peak.${zone}`)}</small>
  {/if}

  <button class="tools" onclick={toggle} aria-expanded={tools} title={t('audio.tools.help')}>{tools ? '\u25BE' : '\u25B8'} {t('audio.tools')}</button>
  {#if tools}
  {#if app.audio?.running}
    <label class="gain">
      <span>{t('decoder.gain')}: <span class="mono">{gain > 0 ? '+' : ''}{gain.toFixed(0)} dB</span></span>
      <input type="range" min="-20" max="20" step="1" value={gain} oninput={(e) => app.setTuning({ gain_db: Number((e.currentTarget as HTMLInputElement).value) })} aria-label={t('decoder.gain')} />
    </label>
  {/if}
  <div class="ctl">
    <div class="seg" role="group" aria-label={t('decoder.preset.title')}>
      {#each ['weak', 'medium', 'strong'] as const as k (k)}
        <button class:on={app.preset === k} aria-pressed={app.preset === k} onclick={() => app.setPreset(k)} title={t('decoder.preset.help')}>{t(`decoder.preset.${k}`)}</button>
      {/each}
    </div>
    <div class="seg" role="group" aria-label={t('decoder.filter')}>
      <button class:on={!wide} aria-pressed={!wide} onclick={() => app.setTuning({ wide_filter: false })} title={t('decoder.filter.help')}>{t('decoder.filter.narrow')}</button>
      <button class:on={wide} aria-pressed={wide} onclick={() => app.setTuning({ wide_filter: true })} title={t('decoder.filter.help')}>{t('decoder.filter.wide')}</button>
    </div>
  </div>
  <div class="ctl">
    <button class="btn small" onclick={() => app.goStrongest()} title={t('audio.panel.strongest.help')}><kbd>{app.settings?.contest_mode ? 'Ctrl+F6' : 'F6'}</kbd> {t('audio.panel.strongest')}</button>
    <button class="btn small" class:on={app.settings?.audio.afc ?? true} aria-pressed={app.settings?.audio.afc ?? true} onclick={() => app.setAfc(!(app.settings?.audio.afc ?? true))} title={t('decoder.afc.help')}><kbd>{app.settings?.contest_mode ? 'Ctrl+F7' : 'F7'}</kbd> AFC</button>
  </div>
  {/if}
</section>

<style>
  .card { background: var(--panel); border: 1px solid var(--line); border-radius: 12px; padding: 12px; display: flex; flex-direction: column; gap: 6px; }
  .head { display: flex; align-items: baseline; justify-content: space-between; gap: 8px; }
  h2 { margin: 0; font-size: 15px; letter-spacing: 0.06em; text-transform: uppercase; color: var(--muted); }
  .frame { border-radius: 8px; overflow: hidden; border: 1px solid var(--line); }
  .link { background: none; border: none; color: var(--accent); font-size: 12px; cursor: pointer; padding: 0; }
  small { font-size: 11px; }
  small.warn { color: var(--warn); font-weight: 600; }
  .peak { display: flex; align-items: center; gap: 8px; margin-top: 4px; }
  .label { font-size: 12px; color: var(--muted); }
  .bar { position: relative; flex: 1; height: 10px; border-radius: 5px; background: var(--line); overflow: hidden; }
  .zone { position: absolute; top: 0; bottom: 0; background: rgba(255, 255, 255, 0.12); }
  .zone.lo { left: 0; width: 5%; }
  .zone.hi { left: 90%; right: 0; }
  .fill { position: absolute; left: 0; top: 0; bottom: 0; transition: width 0.08s linear; }
  .fill.ok { background: #2e9e5b; }
  .fill.low { background: #3d7fc4; }
  .fill.high { background: #c0392b; }
  .val { min-width: 3.2em; text-align: right; font-size: 13px; }
  .val.high { color: #e0584a; font-weight: 700; }
  .gain { display: flex; flex-direction: column; gap: 2px; font-size: 12px; color: var(--muted); }
  .tools { align-self: flex-start; background: none; border: none; color: var(--accent); font-size: 12px; cursor: pointer; padding: 0; }
  .ctl { display: flex; gap: 8px; flex-wrap: wrap; align-items: center; }
  .seg { display: inline-flex; }
  .seg button { background: var(--panel2); border: 1px solid var(--line); padding: 3px 10px; color: var(--text); cursor: pointer; font-size: 12px; }
  .seg button:first-child { border-radius: 8px 0 0 8px; }
  .seg button:last-child { border-radius: 0 8px 8px 0; }
  .seg button:not(:first-child) { margin-left: -1px; }
  .seg button.on { border-color: var(--accent); color: var(--accent); font-weight: 600; position: relative; }
  .small { padding: 3px 10px; font-size: 12px; }
  .small.on { border-color: var(--accent); color: var(--accent); }
  kbd { font: inherit; font-size: 10px; color: var(--muted); border: 1px solid var(--line); border-radius: 4px; padding: 0 3px; }
</style>
