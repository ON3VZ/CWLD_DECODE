<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
  import type { RadioSettings, Settings } from '../lib/types';
  import { STEPS, pickInput, pickPort, silentTest, splitCalls, stationProblems } from '../lib/wizard';

  const base = $state.snapshot(app.settings) as Settings;
  let step = $state(0);
  let radio = $state<RadioSettings>(structuredClone(base.radio));
  let input = $state(base.audio.input);
  let callsText = $state(base.station.calls.join(', '));
  let name = $state(base.station.name);
  let qth = $state(base.station.qth);
  let locator = $state(base.station.locator);
  let tested = $state<'idle' | 'ok' | 'fail'>('idle');
  let autoPicked = $state(false);
  let startFreq = $state(app.status.frequency_hz);
  let problems = $state<Array<{ key: string; call?: string }>>([]);
  let busy = $state(false);

  const kind = $derived(radio.kind);
  const simulator = $derived(kind === 'simulator');
  const levelPct = $derived(app.audioLevel === null ? 0 : Math.max(0, Math.min(100, ((app.audioLevel + 60) / 60) * 100)));
  const levelHint = $derived(app.audioLevel === null ? '' : app.audioLevel > -6 ? 'audio.level.high' : app.audioLevel < -45 ? 'audio.level.low' : 'audio.level.ok');
  const moved = $derived(app.status.connected && startFreq > 0 && app.status.frequency_hz !== startFreq);
  const firstCall = $derived(splitCalls(callsText)[0] ?? 'MYCALL');
  const test = $derived(silentTest(firstCall));
  const khz = $derived((app.status.frequency_hz / 1000).toFixed(1));

  async function search() {
    await app.refreshPorts();
    if (!radio.port || !app.ports.some((p) => p.name === radio.port)) {
      radio.port = pickPort($state.snapshot(app.ports), kind);
      autoPicked = !!radio.port;
    }
  }
  $effect(() => {
    void kind;
    tested = 'idle';
    void search();
  });
  $effect(() => {
    if (step !== 2) return;
    void app.refreshInputs().then(() => {
      input = pickInput($state.snapshot(app.inputs), input);
    });
  });

  /** Always start from the newest settings, then change only what the wizard owns. */
  async function commit(change: (s: Settings) => void) {
    if (!app.settings) return;
    const s = structuredClone($state.snapshot(app.settings)) as Settings;
    change(s);
    await app.saveSettings(s);
  }

  async function testRadio() {
    busy = true;
    tested = 'idle';
    const ok = simulator ? true : await app.connectRadio(radio);
    tested = ok && (simulator || app.status.connected) ? 'ok' : 'fail';
    startFreq = app.status.frequency_hz;
    if (tested === 'ok') await commit((s) => { s.radio = $state.snapshot(radio) as RadioSettings; });
    busy = false;
  }

  async function startListening() {
    busy = true;
    await commit((s) => { s.audio.input = input; });
    await app.startAudio();
    busy = false;
  }

  function next() {
    if (step === 0) startFreq = app.status.frequency_hz;
    if (step < STEPS.length - 1) step += 1;
  }

  async function finish() {
    problems = stationProblems({ calls: callsText, locator });
    if (problems.length) return;
    const connected = app.status.connected && !simulator;
    const listening = !!app.audio?.running;
    await commit((s) => {
      s.station.calls = splitCalls(callsText);
      s.station.name = name.trim().toUpperCase();
      s.station.qth = qth.trim().toUpperCase();
      s.station.locator = locator.trim().toUpperCase();
      s.audio.input = input;
      if (connected) s.radio.auto_connect = true;
      if (listening) s.audio.auto_start = true;
      s.first_run_done = true;
    });
    app.wizardOpen = false;
  }

  async function skip() {
    await commit((s) => { s.first_run_done = true; });
    app.wizardOpen = false;
  }
</script>

<div class="scrim" role="presentation">
  <div class="dlg" role="dialog" aria-modal="true" aria-labelledby="wz-title">
    <header>
      <h2 id="wz-title">{t('wiz.title')}</h2>
      <span class="muted">{t('wiz.step', { n: String(step + 1) })}</span>
    </header>
    <ol class="steps" aria-label={t('wiz.title')}>
      {#each STEPS as s, i (s)}
        <li class:on={i === step} class:done={i < step}><span class="dot">{i + 1}</span><span class="lbl">{t(`wiz.s${i + 1}`)}</span></li>
      {/each}
    </ol>

    <div class="pane">
      {#if step === 0}
        <h3>{t('wiz.s1')}</h3>
        <p>{t('wiz.s1.intro')}</p>
        <label>{t('radio.kind')}
          <select bind:value={radio.kind}>
            <option value="icom">{t('radio.kind.icom')}</option>
            <option value="qmx">{t('radio.kind.qmx')}</option>
            <option value="simulator">{t('radio.kind.simulator')}</option>
          </select>
        </label>
        {#if simulator}
          <p class="note">{t('wiz.s1.sim')}</p>
        {:else}
          <label>{t('radio.port')}
            <div class="row">
              <select bind:value={radio.port} onchange={() => (autoPicked = false)}>
                <option value="">{app.ports.length ? t('radio.port.choose') : t('radio.port.none')}</option>
                {#each app.ports as p (p.name)}
                  <option value={p.name}>{p.name}{p.description ? `, ${p.description}` : ''}</option>
                {/each}
              </select>
              <button class="btn" type="button" onclick={search}>{t('wiz.s1.search')}</button>
            </div>
          </label>
          {#if app.ports.length === 0}
            <div class="note warn">
              <b>{t('wiz.s1.none.title')}</b>
              <ol><li>{t('wiz.s1.none.1')}</li><li>{t('wiz.s1.none.2')}</li><li>{t('wiz.s1.none.3')}</li></ol>
            </div>
          {:else}
            {#if autoPicked}<small class="muted">{t('wiz.s1.found')}</small>{/if}
            {#if kind === 'icom'}<small class="muted">{t('wiz.s1.several')}</small>{/if}
          {/if}
        {/if}
        <div class="row">
          <button class="btn primary" type="button" disabled={busy || app.connecting || (!simulator && !radio.port)} onclick={testRadio}>{t('wiz.s1.test')}</button>
          {#if tested === 'ok'}
            <span class="ok" role="status">{simulator ? '✓' : t('wiz.s1.ok', { name: app.radioName, freq: khz })}</span>
          {:else if tested === 'fail'}
            <span class="err" role="alert">{t('wiz.s1.fail')}</span>
          {/if}
        </div>
      {:else if step === 1}
        <h3>{t('wiz.s2')}</h3>
        {#if simulator}
          <p>{t('wiz.s2.sim')}</p>
        {:else if kind === 'qmx'}
          <p>{t('wiz.s2.qmx')}</p>
        {:else}
          <p>{t('wiz.s2.intro')}</p>
          <p class="note">{t('wiz.s2.civ')}</p>
          <small class="muted">{t('wiz.s2.cant')}</small>
          <p>{t('wiz.s2.test')}</p>
          <div class="freq mono"><span class="muted">{t('wiz.s2.freq')}</span> {khz} kHz</div>
          {#if moved}
            <p class="ok" role="status">✓ {t('wiz.s2.moved')}</p>
          {:else}
            <p class="muted">{t('wiz.s2.notmoved')}</p>
            <small class="muted">{t('wiz.s2.notmoved.help')}</small>
          {/if}
        {/if}
      {:else if step === 2}
        <h3>{t('wiz.s3')}</h3>
        <p>{t('wiz.s3.intro')}</p>
        <label>{t('wiz.s3.input')}
          <div class="row">
            <select bind:value={input}>
              <option value="">{t('audio.input.auto')}</option>
              {#each app.inputs as d (d.name)}
                <option value={d.name}>{d.name}{d.suggested ? ` (${t('audio.input.radio')})` : ''}{d.default ? ` (${t('audio.input.default')})` : ''}</option>
              {/each}
            </select>
            <button class="btn" type="button" onclick={() => app.refreshInputs()}>{t('radio.port.refresh')}</button>
          </div>
        </label>
        <div class="row">
          <button class="btn primary" type="button" disabled={busy} onclick={startListening}>{app.audio?.running ? t('wiz.s3.restart') : t('wiz.s3.start')}</button>
          <span class="muted status">
            {app.audio?.running ? t('audio.running', { source: app.audio.source, rate: String(app.audio.rate) }) : app.audio?.error ? t('audio.failed', { error: app.audio.error }) : t('audio.stopped')}
          </span>
        </div>
        {#if app.audio?.running}
          <div class="level" title={t('wiz.s3.level')}>
            <span class="bar"><span class="fill" style:width={`${levelPct}%`}></span></span>
            <span class="mono muted">{app.audioLevel === null ? '' : `${app.audioLevel.toFixed(0)} dBFS`}</span>
          </div>
          {#if levelHint}<small class="muted">{t(levelHint)}</small>{/if}
        {/if}
        <div class="note">
          <b>{t('wiz.s3.win.title')}</b>
          <ol><li>{t('wiz.s3.win.1')}</li><li>{t('wiz.s3.win.2')}</li><li>{t('wiz.s3.win.3')}</li></ol>
        </div>
        {#if kind === 'qmx'}
          <div class="note warn"><b>{t('wiz.s3.qmx.title')}</b><p>{t('wiz.s3.qmx.body')}</p></div>
        {/if}
      {:else}
        <h3>{t('wiz.s4')}</h3>
        <p>{t('wiz.s4.intro')}</p>
        <label>{t('settings.calls')}
          <input class="mono" bind:value={callsText} />
          <small class="muted">{t('wiz.s4.calls')}</small>
        </label>
        <div class="row">
          <label class="grow">{t('settings.name')}<input bind:value={name} /></label>
          <label class="grow">{t('settings.qth')}<input bind:value={qth} /></label>
          <label>{t('settings.locator')}<input class="mono short" bind:value={locator} placeholder="JO20EE" /></label>
        </div>
        <div class="note">
          <b>{t('wiz.s4.test.title')}</b>
          <p>{t('wiz.s4.test.body')}</p>
          <p class="mono big">{test.text}</p>
          <small class="muted">{t('wiz.s4.test.time', { s: String(test.seconds) })}</small>
        </div>
        <small class="muted">{t('wiz.s4.mode')}</small>
        {#each problems as p (p.key + (p.call ?? ''))}<p class="err" role="alert">{t(p.key, p.call ? { call: p.call } : {})}</p>{/each}
      {/if}
    </div>

    <div class="act">
      <button class="btn" type="button" onclick={skip}>{t('wiz.skip')}</button>
      <span class="spacer"></span>
      {#if step > 0}<button class="btn" type="button" onclick={() => (step -= 1)}>{t('wiz.back')}</button>{/if}
      {#if step < STEPS.length - 1}
        <button class="btn primary" type="button" onclick={next}>{t('wiz.next')}</button>
      {:else}
        <button class="btn primary" type="button" onclick={finish}>{t('wiz.finish')}</button>
      {/if}
    </div>
  </div>
</div>

<style>
  .scrim { position: fixed; inset: 0; background: rgba(0, 0, 0, 0.65); display: grid; place-items: center; z-index: 40; }
  .dlg { background: var(--panel); border: 1px solid var(--line); border-radius: 14px; padding: 20px; width: min(720px, 96vw); max-height: 94vh; display: flex; flex-direction: column; gap: 12px; }
  header { display: flex; justify-content: space-between; align-items: baseline; gap: 12px; }
  h2 { margin: 0; }
  h3 { margin: 0; font-size: 18px; }
  .steps { list-style: none; margin: 0; padding: 0; display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; }
  .steps li { display: flex; align-items: center; gap: 8px; color: var(--muted); font-size: 13px; border-top: 3px solid var(--line); padding-top: 6px; }
  .steps li.on { color: var(--text); border-top-color: var(--accent); font-weight: 600; }
  .steps li.done { border-top-color: var(--ok, var(--accent)); }
  .dot { width: 22px; height: 22px; border-radius: 50%; display: grid; place-items: center; background: var(--panel2); font-size: 12px; flex: none; }
  .steps li.on .dot { background: var(--accent); color: #04180f; }
  .pane { overflow: auto; display: flex; flex-direction: column; gap: 10px; min-height: 320px; padding-right: 4px; }
  .pane p { margin: 0; }
  .note { background: var(--panel2); border: 1px solid var(--line); border-radius: 10px; padding: 10px 12px; font-size: 14px; }
  .note.warn { border-color: var(--live); }
  .note ol { margin: 6px 0 0; padding-left: 20px; display: flex; flex-direction: column; gap: 4px; }
  label { display: flex; flex-direction: column; gap: 3px; font-size: 14px; color: var(--muted); }
  input, select { background: var(--panel2); border: 1px solid var(--line); border-radius: 8px; padding: 7px 9px; color: var(--text); font-size: 16px; }
  .row { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
  .row select { flex: 1; min-width: 160px; }
  .grow { flex: 1; min-width: 120px; }
  .short { width: 8em; }
  .ok { color: var(--ok, #2ecf8a); font-weight: 600; }
  .err { color: var(--live); font-weight: 600; margin: 0; }
  .status { font-size: 13px; }
  .freq { font-size: 28px; }
  .big { font-size: 18px; color: var(--accent); }
  .level { display: flex; gap: 8px; align-items: center; font-size: 13px; }
  .bar { flex: 1; height: 8px; background: var(--line); border-radius: 6px; overflow: hidden; }
  .fill { display: block; height: 100%; background: linear-gradient(90deg, #2e9e5b 70%, #d98e04 90%, #c0392b); transition: width 0.15s; }
  .act { display: flex; gap: 8px; align-items: center; }
  .spacer { flex: 1; }
  @media (max-width: 560px) { .steps .lbl { display: none; } }
</style>
