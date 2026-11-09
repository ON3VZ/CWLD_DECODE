<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
  import { validGrid, validWwffRef } from '../lib/logcheck';

  let open = $state(false);
  let details = $state(false);
  let wwff = $state('');
  let pota = $state('');
  let sota = $state('');
  let grid = $state('');
  let power = $state('');
  let note = $state('');
  let problem = $state<string | null>(null);
  let askEnd = $state(false);

  const portable = $derived(!!app.profile && app.profile.suffix !== '');
  const act = $derived(app.activation);
  /** The WWFF threshold is not confirmed by the ONFF coordinator yet, so it is shown as a hint only. */
  const WWFF_HINT = 44;

  function begin() {
    wwff = '';
    pota = '';
    sota = '';
    grid = app.profile?.grid ?? '';
    power = app.profile?.power ?? '';
    note = '';
    problem = null;
    open = true;
  }

  const wwffBad = $derived(!!wwff.trim() && !validWwffRef(wwff.trim().toUpperCase()));
  const gridBad = $derived(!!grid.trim() && !validGrid(grid));

  async function start() {
    problem = null;
    const code = await app.startActivation({ wwff, pota, sota, grid, power, note });
    if (code) problem = t(`error.${code}`);
    else open = false;
  }
</script>

<div class="wrap">
  {#if act}
    <button class="btn act on" onclick={() => (details = !details)} aria-expanded={details} title={t('act.chip.help')}>
      <span class="mono">{act.wwff || act.pota || act.sota || t('act.noref')}</span>
      <span class="count">{app.actStats?.total ?? 0}</span>
      {#if app.actStats?.crosses_midnight}<span class="alarm" role="img" aria-label={t('act.midnight')}></span>{/if}
    </button>
    {#if details}
      <div class="pop" role="dialog" aria-label={t('act.title')}>
        <div class="refs mono">{act.wwff || act.pota || act.sota || t('act.noref')}</div>
        <div class="grid mono" title={t('act.grid')}>{act.grid || app.settings?.station.locator}</div>
        <div class="muted small">{t('act.counter', { total: String(app.actStats?.total ?? 0), unique: String(app.actStats?.unique ?? 0) })}{#if act.wwff}<span class="gap">{t('act.threshold', { n: String(WWFF_HINT) })}</span>{/if}</div>
        {#if act.wwff && app.actStats && app.actStats.total >= WWFF_HINT}<div class="ok small">{t('act.reached')}</div>{/if}
        {#if app.actStats?.crosses_midnight}<div class="warn small" role="alert">{t('act.midnight')}</div>{/if}
        {#if act.note}<div class="muted small">{act.note}</div>{/if}
        {#if askEnd}
          <div class="ask">
            <span>{t('act.end.ask')}</span>
            <button class="btn" onclick={() => (askEnd = false)}>{t('log.cancel')}</button>
            <button class="btn primary" onclick={() => { askEnd = false; details = false; app.endActivation(); }}>{t('act.end')}</button>
          </div>
        {:else}
          <div class="row"><button class="btn" onclick={() => (askEnd = true)}>{t('act.end')}</button><button class="btn" onclick={() => (details = false)}>{t('act.close')}</button></div>
        {/if}
      </div>
    {/if}
  {:else}
    <button class="btn act" onclick={begin} title={portable ? t('act.none.portable') : t('act.none')}>{t('act.title')}</button>
  {/if}
</div>

{#if open}
  <div class="scrim" role="presentation">
    <div class="dlg" role="dialog" aria-modal="true" aria-labelledby="act-title">
      <h2 id="act-title">{t('act.new')}</h2>
      <p class="muted small">{t('act.intro')}</p>
      <label>{t('act.wwff')}
        <input class="mono" class:bad={wwffBad} bind:value={wwff} placeholder="ONFF-0599" autocapitalize="characters" />
        <span class="muted small">{wwffBad ? t('act.wwff.bad') : t('act.wwff.help')}</span>
      </label>
      <div class="two">
        <label>{t('act.pota')}<input class="mono" bind:value={pota} placeholder="ON-00001" /></label>
        <label>{t('act.sota')}<input class="mono" bind:value={sota} placeholder="ON/LG-001" /></label>
      </div>
      <div class="two">
        <label>{t('act.grid')}
          <input class="mono" class:bad={gridBad} bind:value={grid} placeholder="JO20QE" autocapitalize="characters" />
          <span class="muted small">{gridBad ? t('act.grid.bad') : portable ? t('act.grid.help.portable') : t('act.grid.help')}</span>
        </label>
        <label>{t('act.power')}<input class="mono" bind:value={power} placeholder="5" inputmode="decimal" /></label>
      </div>
      <label>{t('act.note')}<input bind:value={note} /></label>
      {#if problem}<p class="err" role="alert">{problem}</p>{/if}
      <div class="act-row">
        <button class="btn" onclick={() => (open = false)}>{t('log.cancel')}</button>
        <button class="btn primary" disabled={wwffBad || gridBad} onclick={start}>{t('act.start')}</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .wrap { position: relative; margin-bottom: 4px; }
  .act { white-space: nowrap; display: inline-flex; align-items: center; gap: 8px; border-color: color-mix(in srgb, var(--listen) 55%, var(--line)); color: var(--listen); background: color-mix(in srgb, var(--listen) 10%, var(--panel2)); }
  .act.on { background: color-mix(in srgb, var(--listen) 22%, var(--panel2)); font-weight: 700; }
  .count { background: var(--listen); color: #04180f; border-radius: 999px; padding: 0 7px; font-size: 12px; font-weight: 800; }
  .alarm { width: 9px; height: 9px; border-radius: 50%; background: var(--live); box-shadow: 0 0 6px var(--live); }
  .pop { position: absolute; right: 0; top: calc(100% + 6px); z-index: 20; width: min(340px, 92vw); background: var(--panel); border: 1px solid var(--line); border-radius: 12px; padding: 12px; display: flex; flex-direction: column; gap: 6px; box-shadow: 0 12px 40px rgba(0, 0, 0, 0.5); }
  .refs { font-size: 18px; font-weight: 700; color: var(--listen); }
  .grid { font-size: 26px; font-weight: 800; letter-spacing: 0.06em; }
  .small { font-size: 12px; }
  .gap { margin-left: 0.4em; }
  p { margin: 0; }
  .ok { color: var(--listen); }
  .warn { color: var(--warn); }
  .err { color: var(--live); font-weight: 600; }
  .ask { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; font-size: 13px; }
  .row { display: flex; gap: 8px; }
  .scrim { position: fixed; inset: 0; background: rgba(0, 0, 0, 0.6); display: grid; place-items: center; z-index: 30; }
  .dlg { background: var(--panel); border: 1px solid var(--line); border-radius: 14px; padding: 18px; width: min(520px, 94vw); display: flex; flex-direction: column; gap: 10px; }
  .dlg h2 { margin: 0; font-size: 18px; color: var(--text); }
  .two { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
  label { display: flex; flex-direction: column; font-size: 12px; color: var(--muted); gap: 2px; }
  input { background: var(--panel2); border: 1px solid var(--line); border-radius: 8px; padding: 6px 8px; font-size: 16px; color: var(--text); }
  input.bad { border-color: var(--live); }
  .act-row { display: flex; gap: 8px; justify-content: flex-end; }
</style>
