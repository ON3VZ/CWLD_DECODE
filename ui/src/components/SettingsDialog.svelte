<script lang="ts">
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
  });

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
</style>
