<script lang="ts">
  import { looksLikeCall } from '../lib/callsign';
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
  const steep = $derived(!wide && (app.settings?.audio.steep_filter ?? false));
  const rw = $derived(app.rewind);
  const SECONDS = [10, 20, 30, 60];
  const fading = $derived(app.settings?.audio.fading ?? true);

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

{#snippet holeNote()}
  {#if app.audioHole !== null || app.radioFilterIssues.length}
    {@const issues = app.radioFilterIssues}
    <div class="hole" role="alert" title={t('audio.hole.help')}>
      {#if app.audioHole !== null}<p><strong>{t('audio.hole', { hz: Math.round(app.audioHole) })}</strong> {issues.length ? '' : t('audio.hole.do')}</p>{/if}
      {#if issues.length}<p>{t('audio.radio_filters', { list: issues.map((i) => t(`audio.filter.${i}`)).join(', ') })}</p>{/if}
      {#if app.audioHole !== null || issues.length}
        <div class="holebtns">
          {#if app.audioHole !== null || issues.includes('notch') || issues.includes('auto_notch')}<button class="btn small" onclick={() => app.notchOff()} title={t('audio.notch_off.help')}>{t('audio.notch_off')}</button>{/if}
          {#if app.audioHole !== null || issues.includes('pbt')}<button class="btn small" onclick={() => app.pbtCenter()} title={t('audio.pbt_center.help')}>{t('audio.pbt_center')}</button>{/if}
        </div>
      {/if}
    </div>
  {/if}
{/snippet}

<section class="card">
  <div class="head">
    <h2>{t('audio.panel.title')}</h2>
    <button class="link" onclick={() => app.setView('audio_waterfall', false)}>{t('audio.panel.hide')}</button>
  </div>
  <div class="framewrap">
    <div class="frame"><Waterfall height={110} /></div>
    <!-- A warning lies over the waterfall, so nothing below it moves when it appears. -->
    {@render holeNote()}
  </div>
  <small class="muted">{t('audio.panel.help')}</small>

  <div class="peakslot">
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
  </div>

  <div class="ctl fade">
    <span class="lbl" id="fading-lbl">{t('decoder.fading')}</span>
    <div class="seg" role="group" aria-labelledby="fading-lbl">
      <button class:on={fading} aria-pressed={fading} onclick={() => app.setFading(true)} title={t('decoder.fading.on.help')}>{t('decoder.fading.on')}</button>
      <button class:on={!fading} aria-pressed={!fading} onclick={() => app.setFading(false)} title={t('decoder.fading.off.help')}>{t('decoder.fading.off')}</button>
    </div>
    <small class="muted">{t(fading ? 'decoder.fading.on.note' : 'decoder.fading.off.note')}</small>
  </div>
  {#if app.audio?.running}
    <button class="tools" onclick={() => app.toggleRewind()} aria-expanded={rw.open} title={t('rewind.help')}>{rw.open ? '\u25BE' : '\u25B8'} {t('rewind.title')}</button>
    {#if rw.open}
      <div class="rewind" aria-busy={rw.busy}>
        <div class="ctl">
          <div class="seg" role="group" aria-label={t('rewind.seconds')}>
            {#each SECONDS as s (s)}
              <button class:on={rw.seconds === s} aria-pressed={rw.seconds === s} onclick={() => app.replay(rw.asked, s)} disabled={rw.busy}>{s} s</button>
            {/each}
          </div>
          <button class="btn small" onclick={() => app.replay(null)} disabled={rw.busy} title={t('rewind.current.help')}>{t('rewind.current')}</button>
          <button class="btn small" onclick={() => app.replay('strongest')} disabled={rw.busy} title={t('rewind.strongest.help')}>{t('rewind.strongest')}</button>
        </div>
        {#if rw.result && rw.result.tones.length > 0}
          <div class="ctl">
            <span class="lbl">{t('rewind.tones')}</span>
            {#each rw.result.tones as tn (tn.hz)}
              <button class="btn small" class:on={Math.abs(rw.result.tone_hz - tn.hz) < 20} onclick={() => app.replay(Math.round(tn.hz))} disabled={rw.busy} title={t('rewind.tone.help', { db: tn.score_db.toFixed(0) })}>{Math.round(tn.hz)} Hz</button>
            {/each}
          </div>
        {/if}
        {#if rw.busy}
          <small class="muted">{t('rewind.busy')}</small>
        {:else if rw.result}
          <small class="muted">{t('rewind.result', { seconds: rw.result.seconds.toFixed(0), hz: rw.result.tone_hz.toFixed(0), wpm: rw.result.wpm > 0 ? rw.result.wpm.toFixed(0) : '?' })}</small>
          <p class="mono replaytext">{#if rw.result.text}{#each rw.result.text.split(/(\s+)/) as w, i (i)}{#if looksLikeCall(w)}<button class="call" onclick={() => app.selectCall(w)}>{w}</button>{:else}{w}{/if}{/each}{:else}<span class="muted">{t('rewind.empty')}</span>{/if}</p>
        {:else}
          <small class="muted">{t('rewind.intro')}</small>
        {/if}
        <div class="ctl">
          <button class="btn small" onclick={() => app.saveRewind()} title={t('rewind.save.help')}>{t('rewind.save')}</button>
          {#if rw.saved}<small class="muted mono">{rw.saved}</small>{/if}
        </div>
      </div>
    {/if}
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
      <button class:on={!wide && !steep} aria-pressed={!wide && !steep} onclick={() => app.setTuning({ wide_filter: false, steep_filter: false })} title={t('decoder.filter.help')}>{t('decoder.filter.narrow')}</button>
      <button class:on={steep} aria-pressed={steep} onclick={() => app.setTuning({ wide_filter: false, steep_filter: true })} title={t('decoder.filter.steep.help')}>{t('decoder.filter.steep')}</button>
      <button class:on={wide} aria-pressed={wide} onclick={() => app.setTuning({ wide_filter: true, steep_filter: false })} title={t('decoder.filter.help')}>{t('decoder.filter.wide')}</button>
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
  .framewrap { position: relative; }
  .peakslot { min-height: 56px; }
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
  .rewind { display: flex; flex-direction: column; gap: 6px; padding: 8px; border: 1px dashed var(--line); border-radius: 8px; }
  .replaytext { margin: 0; font-size: 14px; line-height: 1.5; word-break: break-word; }
  .replaytext .call { background: none; border: none; padding: 0; font: inherit; color: var(--accent); cursor: pointer; text-decoration: underline; }
  .seg { display: inline-flex; }
  .lbl { font-size: 12px; color: var(--muted); }
  .fade small { flex-basis: 100%; font-size: 11px; line-height: 1.3; }
  .seg button { background: var(--panel2); border: 1px solid var(--line); padding: 3px 10px; color: var(--text); cursor: pointer; font-size: 12px; }
  .seg button:first-child { border-radius: 8px 0 0 8px; }
  .seg button:last-child { border-radius: 0 8px 8px 0; }
  .seg button:not(:first-child) { margin-left: -1px; }
  .seg button.on { border-color: var(--accent); color: var(--accent); font-weight: 600; position: relative; }
  .small { padding: 3px 10px; font-size: 12px; }
  .small.on { border-color: var(--accent); color: var(--accent); }
  kbd { font: inherit; font-size: 10px; color: var(--muted); border: 1px solid var(--line); border-radius: 4px; padding: 0 3px; }
  .hole p { margin: 0 0 4px; }
  .holebtns { display: flex; gap: 6px; flex-wrap: wrap; margin-top: 4px; }
  .holebtns .btn { font-size: 12px; padding: 3px 9px; }
  .hole { position: absolute; left: 6px; right: 6px; bottom: 6px; margin: 0; z-index: 2; padding: 6px 8px; max-height: 100px; overflow: auto; box-shadow: 0 2px 10px rgba(0, 0, 0, 0.35); border: 1px solid var(--warn, #b9770e); border-radius: 6px; font-size: 12.5px; line-height: 1.4; color: var(--warn, #b9770e); background: color-mix(in srgb, var(--warn, #b9770e) 12%, var(--panel)); }
</style>
