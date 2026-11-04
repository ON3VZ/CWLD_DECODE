<script lang="ts">
  import { tick } from 'svelte';
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
  import type { Settings } from '../lib/types';

  const original = $state.snapshot(app.settings) as Settings;
  let draft = $state<Settings>(structuredClone(original));
  let callsText = $state(original.station.calls.join(', '));
  let problem = $state<string | null>(null);
  let addressText = $state(original.radio.civ_address.toString(16).toUpperCase().padStart(2, '0'));
  let showAdvanced = $state(false);

  $effect(() => {
    app.refreshPorts();
    app.refreshInputs();
  });
  let wavPath = $state('');
  // Input level as a bar: -60 dBFS empty, 0 dBFS full.
  const levelPct = $derived(app.audioLevel === null ? 0 : Math.max(0, Math.min(100, ((app.audioLevel + 60) / 60) * 100)));
  const levelHint = $derived(app.audioLevel === null ? '' : app.audioLevel > -6 ? 'audio.level.high' : app.audioLevel < -45 ? 'audio.level.low' : 'audio.level.ok');

  async function applyAudio() {
    // The input is part of the settings: save first, then (re)start with it.
    await app.saveSettings(draft);
    await app.startAudio();
  }

  function readAddress(): number | null {
    const v = parseInt(addressText.trim().replace(/^0x/i, ''), 16);
    return Number.isNaN(v) || v < 1 || v > 0xdf ? null : v;
  }

  async function connect() {
    const addr = readAddress();
    if (addr === null) {
      problem = t('error.invalid_argument');
      return;
    }
    problem = null;
    draft.radio.civ_address = addr;
    await app.connectRadio(draft.radio);
  }

  const CALL = /^(?:[A-Z]{1,2}|[0-9][A-Z])[0-9][A-Z]{1,4}(?:\/[A-Z0-9]{1,4})?$/;
  const LOC = /^[A-R]{2}[0-9]{2}(?:[A-X]{2})?$/i;

  async function save() {
    const addr = readAddress();
    if (addr === null) {
      problem = t('error.invalid_argument');
      return;
    }
    draft.radio.civ_address = addr;
    draft.swr.limit = Math.min(3, Math.max(1.1, Number(draft.swr.limit) || 1.5));
    const calls = callsText.split(',').map((c) => c.trim().toUpperCase()).filter(Boolean);
    const bad = calls.find((c) => !CALL.test(c));
    if (bad) {
      problem = t('settings.invalid.call', { call: bad });
      return;
    }
    if (draft.station.locator && !LOC.test(draft.station.locator)) {
      problem = t('settings.invalid.locator');
      return;
    }
    problem = null;
    draft.station.calls = calls;
    draft.first_run_done = true;
    await app.saveSettings(draft);
    app.settingsOpen = false;
  }
  function reset() {
    draft = structuredClone(original);
    callsText = original.station.calls.join(', ');
  }
</script>

<div class="scrim" role="presentation">
  <div class="dlg" role="dialog" aria-modal="true" aria-labelledby="st-title">
    <h2 id="st-title">{t('settings.title')}</h2>

    <label>{t('settings.language')}
      <select bind:value={draft.language}>
        <option value="en">English</option>
        <option value="nl">Nederlands</option>
      </select>
    </label>
    <label>{t('settings.theme')}
      <select bind:value={draft.theme}>
        <option value="dark">{t('settings.theme.dark')}</option>
        <option value="light">{t('settings.theme.light')}</option>
        <option value="system">{t('settings.theme.system')}</option>
      </select>
    </label>
    <label>{t('settings.calls')}
      <input class="mono" bind:value={callsText} />
      <small class="muted">{t('settings.calls.help')}</small>
    </label>
    <label>{t('settings.name')}<input bind:value={draft.station.name} /></label>
    <label>{t('settings.qth')}<input bind:value={draft.station.qth} /></label>
    <label>{t('settings.locator')}
      <input class="mono" bind:value={draft.station.locator} />
      <small class="muted">{t('settings.locator.help')}</small>
    </label>

    <h3>{t('radio.title')}</h3>
    <label>{t('radio.kind')}
      <select bind:value={draft.radio.kind}>
        <option value="simulator">{t('radio.kind.simulator')}</option>
        <option value="icom">{t('radio.kind.icom')}</option>
        <option value="qmx">{t('radio.kind.qmx')}</option>
      </select>
    </label>

    {#if draft.radio.kind !== 'simulator'}
      <label>{t('radio.port')}
        <div class="row">
          <select bind:value={draft.radio.port}>
            <option value="">{app.ports.length ? t('radio.port.choose') : t('radio.port.none')}</option>
            {#each app.ports as p (p.name)}
              <option value={p.name}>{p.name}{p.description ? `, ${p.description}` : ''}{p.hint === 'icom' ? `, ${t('radio.port.icom')}` : ''}</option>
            {/each}
          </select>
          <button class="btn" type="button" onclick={() => app.refreshPorts()}>{t('radio.port.refresh')}</button>
        </div>
        <small class="muted">{t('radio.port.help')}</small>
      </label>
    {/if}

    {#if draft.radio.kind === 'icom'}
      <label>{t('radio.address')}
        <input class="mono short" bind:value={addressText} maxlength="4" />
        <small class="muted">{t('radio.address.help')}</small>
      </label>
    {/if}
    {#if draft.radio.kind === 'qmx'}
      <p class="muted note">{t('radio.qmx.help')}</p>
    {/if}

    {#if draft.radio.kind !== 'simulator'}
      <button class="link" type="button" onclick={() => (showAdvanced = !showAdvanced)}>{showAdvanced ? '▾' : '▸'} {t('radio.baud')} / {t('radio.pipeline')}</button>
      {#if showAdvanced}
        <label>{t('radio.baud')}
          <input class="mono short" type="number" bind:value={draft.radio.baud} />
          <small class="muted">{t('radio.baud.help')}</small>
        </label>
        {#if draft.radio.kind === 'icom'}
          <label class="check"><input type="checkbox" bind:checked={draft.radio.pipeline} /> {t('radio.pipeline')}
            <small class="muted">{t('radio.pipeline.help')}</small>
          </label>
        {/if}
      {/if}
    {/if}

    <div class="row">
      <button class="btn primary" type="button" disabled={app.connecting} onclick={connect}>{t('radio.connect')}</button>
      <button class="btn" type="button" onclick={() => app.disconnectRadio()}>{t('radio.disconnect')}</button>
      <span class="muted status">
        {app.status.connected ? t('radio.connected', { name: app.radioName }) : t('radio.notconnected')}
      </span>
    </div>
    {#if app.error}<p class="err" role="alert">{app.error}</p>{/if}

    <h3>{t('swr.title')}</h3>
    <label class="check"><input type="checkbox" bind:checked={draft.swr.enabled} /> {t('swr.enabled')}
      <small class="muted">{t('swr.enabled.help')}</small>
    </label>
    {#if draft.swr.enabled}
      <label>{t('swr.limit')}
        <input class="mono short" type="number" min="1.1" max="3" step="0.1" bind:value={draft.swr.limit} />
        <small class="muted">{t('swr.limit.help')}</small>
      </label>
      <label class="check"><input type="checkbox" bind:checked={draft.swr.qrx_after_stop} /> {t('swr.qrx')}
        <small class="muted">{t('swr.qrx.help')}</small>
      </label>
    {/if}
    {#if draft.radio.kind === 'qmx'}
      <p class="muted note">{t('swr.qmx_unverified')}</p>
    {/if}

    <h3>{t('update.title')}</h3>
    <p class="muted">{t('update.version', { version: app.version })}</p>
    <div class="row">
      <button class="btn" type="button" disabled={app.updateState === 'checking' || app.updateState === 'downloading'} onclick={() => app.checkUpdate()}>{t('update.check')}</button>
      {#if app.updateState === 'available' && app.updateInfo}
        <button class="btn primary" type="button" onclick={() => app.installUpdate()}>{t('update.install')}</button>
      {/if}
    </div>
    {#if app.updateState === 'checking'}<p class="muted">{t('update.checking')}</p>{/if}
    {#if app.updateState === 'none'}<p class="muted">{t('update.none')}</p>{/if}
    {#if app.updateState === 'error'}<p class="muted">{t('update.error')}</p>{/if}
    {#if app.updateState === 'available' && app.updateInfo}
      <p><strong>{t('update.available', { version: app.updateInfo.version })}</strong></p>
      {#if app.updateInfo.notes}<pre class="notes">{app.updateInfo.notes}</pre>{/if}
    {/if}
    <small class="muted">{t('update.privacy')}</small>

    <h3>{t('audio.title')}</h3>
    <label>{t('audio.input')}
      <div class="row">
        <select bind:value={draft.audio.input} onchange={async () => { await tick(); await applyAudio(); }}>
          <option value="">{t('audio.input.auto')}</option>
          {#each app.inputs as d (d.name)}
            <option value={d.name}>{d.name}{d.suggested ? ` (${t('audio.input.radio')})` : ''}{d.default ? ` (${t('audio.input.default')})` : ''}</option>
          {/each}
        </select>
        <button class="btn" type="button" onclick={() => app.refreshInputs()}>{t('radio.port.refresh')}</button>
      </div>
      <small class="muted">{t('audio.input.help')}</small>
    </label>
    <label>{t('audio.tone')}
      <input class="mono short" type="number" min="200" max="1500" step="10" bind:value={draft.audio.tone_hz} />
      <small class="muted">{t('audio.tone.help')}</small>
    </label>
    <label>{t('audio.squelch')}: <span class="mono">{(draft.audio.squelch ?? 3.5).toFixed(1)}</span>
      <input type="range" min="1" max="8" step="0.5" bind:value={draft.audio.squelch} onchange={() => app.setSquelch(draft.audio.squelch)} />
      <small class="muted">{t('audio.squelch.help')}</small>
    </label>
    <label class="check"><input type="checkbox" bind:checked={draft.audio.auto_tone} onchange={() => app.setAutoTone(draft.audio.auto_tone)} /> {t('audio.autotone')}</label>
    <small class="muted">{t('audio.autotone.help')}</small>
    <label class="check"><input type="checkbox" bind:checked={draft.audio.auto_start} /> {t('audio.autostart')}</label>
    <label>{t('audio.recdir')}
      <input class="mono" bind:value={draft.audio.record_dir} placeholder={t('audio.recdir.default')} />
      <small class="muted">{t('audio.recdir.help')}</small>
    </label>
    <div class="row">
      {#if app.audio?.running}
        <button class="btn" type="button" onclick={() => app.stopAudio()}>{t('audio.stop')}</button>
        <button class="btn" type="button" onclick={applyAudio}>{t('audio.restart')}</button>
      {:else}
        <button class="btn primary" type="button" onclick={applyAudio}>{t('audio.start')}</button>
      {/if}
      <span class="muted status">
        {app.audio?.running ? t('audio.running', { source: app.audio.source, rate: String(app.audio.rate) }) : app.audio?.error ? t('audio.failed', { error: app.audio.error }) : t('audio.stopped')}
      </span>
    </div>
    {#if app.audio?.running}
      <div class="level" title={t('audio.level')}>
        <span class="bar"><span class="fill" style:width={`${levelPct}%`}></span></span>
        <span class="mono muted">{app.audioLevel === null ? '' : `${app.audioLevel.toFixed(0)} dBFS`}</span>
      </div>
      {#if levelHint}<small class="muted">{t(levelHint)}</small>{/if}
      {#if app.audio.overruns > 0}<small class="err">{t('audio.overruns', { n: String(app.audio.overruns) })}</small>{/if}
    {/if}
    <label>{t('audio.file')}
      <div class="row">
        <input class="mono" bind:value={wavPath} placeholder="C:\Users\...\opname.wav" />
        <button class="btn" type="button" disabled={!wavPath.trim()} onclick={() => app.decodeFile(wavPath)}>{t('audio.file.decode')}</button>
      </div>
      <small class="muted">{t('audio.file.help')}</small>
    </label>

    {#if problem}<p class="err" role="alert">{problem}</p>{/if}

    <div class="act">
      <button class="btn" onclick={reset}>{t('settings.reset')}</button>
      <span class="spacer"></span>
      <button class="btn" onclick={() => (app.settingsOpen = false)}>{t('settings.close')}</button>
      <button class="btn primary" onclick={save}>OK</button>
    </div>
  </div>
</div>

<style>
  .scrim { position: fixed; inset: 0; background: rgba(0, 0, 0, 0.6); display: grid; place-items: center; z-index: 10; }
  .dlg { background: var(--panel); border: 1px solid var(--line); border-radius: 14px; padding: 20px; width: min(460px, 92vw); max-height: 92vh; overflow: auto; display: flex; flex-direction: column; gap: 10px; }
  h2 { margin: 0; }
  label { display: flex; flex-direction: column; gap: 3px; font-size: 14px; color: var(--muted); }
  input, select { background: var(--panel2); border: 1px solid var(--line); border-radius: 8px; padding: 7px 9px; color: var(--text); font-size: 16px; }
  .act { display: flex; gap: 8px; align-items: center; margin-top: 6px; }
  .spacer { flex: 1; }
  h3 { margin: 6px 0 0; font-size: 15px; letter-spacing: 0.06em; text-transform: uppercase; color: var(--muted); }
  .row { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
  .row select { flex: 1; min-width: 160px; }
  .short { width: 7em; }
  .note { margin: 0; font-size: 13px; }
  .status { font-size: 13px; }
  .check { flex-direction: row; flex-wrap: wrap; align-items: baseline; gap: 6px; }
  .check small { flex-basis: 100%; }
  .link { background: none; border: none; padding: 0; color: var(--accent); text-align: left; font-size: 13px; }
  .err { color: var(--live); margin: 0; font-weight: 600; }
  .level { display: flex; gap: 8px; align-items: center; font-size: 13px; }
  .level .bar { flex: 1; height: 8px; border-radius: 4px; background: var(--line); overflow: hidden; }
  .level .fill { display: block; height: 100%; background: linear-gradient(90deg, #2e9e5b 70%, #d98e04 90%, #c0392b); }
</style>
