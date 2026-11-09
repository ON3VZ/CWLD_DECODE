<script lang="ts">
  import ContestProfiles from './ContestProfiles.svelte';
  import { tick } from 'svelte';
  import { app, PRESETS } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
  import type { Settings } from '../lib/types';

  const original = $state.snapshot(app.settings) as Settings;
  let draft = $state<Settings>(structuredClone(original));
  let qrzPassword = $state('');
  let callsText = $state(original.station.calls.join(', '));
  let problem = $state<string | null>(null);
  let addressText = $state(original.radio.civ_address.toString(16).toUpperCase().padStart(2, '0'));
  let showAdvanced = $state(false);
  type Section = 'station' | 'radio' | 'audio' | 'decoder' | 'send' | 'view' | 'contest' | 'log' | 'updates';
  let section = $state<Section>('station');
  const sections: Section[] = ['station', 'radio', 'audio', 'decoder', 'send', 'view', 'contest', 'log', 'updates'];

  /** A slider or switch of the decoder acts at once, and the draft keeps the same values. */
  function tune() {
    const a = draft.audio;
    app.setTuning({ gain_db: a.gain_db, wide_filter: a.wide_filter, min_wpm: a.min_wpm, max_wpm: a.max_wpm, lock_wpm: a.lock_wpm });
  }
  function preset(name: 'weak' | 'medium' | 'strong') {
    draft.audio.squelch = PRESETS[name];
    app.setPreset(name);
  }
  const presetNow = $derived((Object.keys(PRESETS) as Array<keyof typeof PRESETS>).find((k) => Math.abs(PRESETS[k] - (draft.audio.squelch ?? 3.5)) < 0.05) ?? null);

  $effect(() => {
    app.refreshPorts();
    app.refreshInputs();
    app.lookupRefresh();
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
    // These are changed on the main screen or in their own section: keep the newest values.
    if (app.settings) {
      draft.contest_mode = app.settings.contest_mode;
      draft.contest_nr = app.settings.contest_nr;
      draft.user_words = $state.snapshot(app.settings.user_words);
      draft.contest = $state.snapshot(app.settings.contest);
      draft.profile = app.settings.profile;
    }
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

    <div class="body">
      <nav class="nav" aria-label={t('settings.title')}>
        {#each sections as k (k)}
          <button type="button" class:on={section === k} aria-current={section === k ? 'page' : undefined} onclick={() => (section = k)}>{t(`settings.sec.${k}`)}</button>
        {/each}
      </nav>

      <div class="pane">
        {#if section === 'station'}
          <h3>{t('settings.sec.station')}</h3>
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

          <h3>{t('profile.title')}</h3>
          <small class="muted">{t('profile.help')}</small>
          {#each draft.profiles as p (p.id)}
            <div class="group">
              <div class="row">
                <label>{t('profile.name')}<input bind:value={p.name} /></label>
                <label>{t('profile.suffix')}<input class="mono short" bind:value={p.suffix} placeholder="/P" /></label>
                <label>{t('profile.grid')}<input class="mono short" bind:value={p.grid} placeholder="JO20EE" /></label>
                <label>{t('profile.power')}<input class="mono short" bind:value={p.power} placeholder="20" /></label>
              </div>
              <div class="row">
                <label>{t('profile.rig')}<input bind:value={p.rig} /></label>
                <label>{t('profile.antenna')}<input bind:value={p.antenna} /></label>
                <label>{t('profile.comment')}<input bind:value={p.comment} /></label>
              </div>
            </div>
          {/each}
        {/if}

        {#if section === 'radio'}
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
            <label class="check"><input type="checkbox" bind:checked={draft.radio.auto_connect} /> {t('radio.autoconnect')}
              <small class="muted">{t('radio.autoconnect.help')}</small>
            </label>
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
        {/if}

        {#if section === 'audio'}
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
        {/if}

        {#if section === 'decoder'}
          <h3>{t('settings.sec.decoder')}</h3>
          <label>{t('audio.tone')}
            <input class="mono short" type="number" min="200" max="1500" step="10" bind:value={draft.audio.tone_hz} />
            <small class="muted">{t('audio.tone.help')}</small>
          </label>
          <label class="check"><input type="checkbox" bind:checked={draft.audio.auto_tone} onchange={() => app.setAutoTone(draft.audio.auto_tone)} /> {t('audio.autotone')}</label>
          <small class="muted">{t('audio.autotone.help')}</small>

          <div class="group">
            <strong>{t('decoder.preset.title')}</strong>
            <div class="seg" role="group" aria-label={t('decoder.preset.title')}>
              {#each ['weak', 'medium', 'strong'] as const as k (k)}
                <button type="button" class:on={presetNow === k} aria-pressed={presetNow === k} onclick={() => preset(k)}>{t(`decoder.preset.${k}`)}</button>
              {/each}
            </div>
            <small class="muted">{t('decoder.preset.help')}</small>
          </div>
          <label>{t('audio.squelch')}: <span class="mono">{(draft.audio.squelch ?? 3.5).toFixed(1)}</span>
            <input type="range" min="1" max="8" step="0.5" bind:value={draft.audio.squelch} onchange={() => app.setSquelch(draft.audio.squelch)} />
            <small class="muted">{t('audio.squelch.help')}</small>
          </label>

          <div class="group">
            <strong>{t('decoder.filter')}</strong>
            <div class="seg" role="group" aria-label={t('decoder.filter')}>
              <button type="button" class:on={!draft.audio.wide_filter} aria-pressed={!draft.audio.wide_filter} onclick={() => { draft.audio.wide_filter = false; tune(); }}>{t('decoder.filter.narrow')}</button>
              <button type="button" class:on={draft.audio.wide_filter} aria-pressed={draft.audio.wide_filter} onclick={() => { draft.audio.wide_filter = true; tune(); }}>{t('decoder.filter.wide')}</button>
            </div>
            <small class="muted">{t('decoder.filter.help')}</small>
          </div>
          <label class="check"><input type="checkbox" bind:checked={draft.audio.afc} onchange={() => app.setAfc(draft.audio.afc)} /> {t('decoder.afc')}
            <small class="muted">{t('decoder.afc.help')}</small>
          </label>
          <label>{t('decoder.gain')}: <span class="mono">{draft.audio.gain_db > 0 ? '+' : ''}{draft.audio.gain_db.toFixed(0)} dB</span>
            <input type="range" min="-20" max="20" step="1" bind:value={draft.audio.gain_db} onchange={tune} />
            <small class="muted">{t('decoder.gain.help')}</small>
          </label>

          <div class="group">
            <strong>{t('decoder.speed.title')}</strong>
            <div class="row">
              <label>{t('decoder.minwpm')}
                <input class="mono short" type="number" min="5" max="55" bind:value={draft.audio.min_wpm} onchange={tune} />
              </label>
              <label>{t('decoder.maxwpm')}
                <input class="mono short" type="number" min="10" max="60" bind:value={draft.audio.max_wpm} onchange={tune} />
              </label>
              <label>{t('decoder.lockwpm')}
                <input class="mono short" type="number" min="0" max="60" bind:value={draft.audio.lock_wpm} onchange={tune} />
              </label>
            </div>
            <small class="muted">{t('decoder.speed.help')}</small>
          </div>
        {/if}

        {#if section === 'send'}
          <h3>{t('settings.sec.send')}</h3>
          <label>{t('settings.default_rst')}
            <select bind:value={draft.default_rst}>
              <option value="5NN">5NN</option>
              <option value="599">599</option>
            </select>
            <small class="muted">{t('settings.default_rst.help')}</small>
          </label>
          <small class="muted">{t('settings.send.speed')}</small>
        {/if}

        {#if section === 'view'}
          <h3>{t('settings.sec.view')}</h3>
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
          <label class="check"><input type="checkbox" bind:checked={draft.band_scope} /> {t('view.band_scope')}</label>
          <small class="muted">{t('view.band_scope.help')}</small>
          <label class="check"><input type="checkbox" bind:checked={draft.audio_waterfall} /> {t('view.audio_waterfall')}</label>
          <small class="muted">{t('view.audio_waterfall.help')}</small>

          <label>{t('view.scope_size')}
            <select bind:value={draft.scope_size}>
              <option value="small">{t('view.size.small')}</option>
              <option value="medium">{t('view.size.medium')}</option>
              <option value="large">{t('view.size.large')}</option>
            </select>
          </label>
          <label>{t('view.scope_size_contest')}
            <select bind:value={draft.scope_size_contest}>
              <option value="small">{t('view.size.small')}</option>
              <option value="medium">{t('view.size.medium')}</option>
              <option value="large">{t('view.size.large')}</option>
            </select>
          </label>

          <h3>{t('tabs.settings')}</h3>
          <label>{t('tabs.wait')}
            <input class="mono short" type="number" min="1" max="30" step="1" bind:value={draft.new_tab_seconds} />
            <small class="muted">{t('tabs.wait.help')}</small>
          </label>
          <label>{t('tabs.autoclose')}
            <input class="mono short" type="number" min="0" max="240" step="5" bind:value={draft.tabs_auto_close_minutes} />
            <small class="muted">{t('tabs.autoclose.help')}</small>
          </label>
        {/if}

        {#if section === 'contest'}
          <h3>{t('settings.sec.contest')}</h3>
          <label class="check"><input type="checkbox" checked={app.settings?.contest_mode ?? false} onchange={(e) => app.requestContest((e.currentTarget as HTMLInputElement).checked)} /> {t('contest.on')}
            <small class="muted">{t('contest.help')}</small>
          </label>
          {#if app.contestAsk}
            <div class="group" role="group" aria-label={t('contest.ask', { nr: String(app.settings?.contest_nr ?? 1) })}>
              <strong>{t('contest.ask', { nr: String(app.settings?.contest_nr ?? 1) })}</strong>
              <div class="row">
                <button class="btn primary" type="button" onclick={() => app.answerContest(false)}>{t('contest.continue')}</button>
                <button class="btn" type="button" onclick={() => app.answerContest(true)}>{t('contest.restart')}</button>
              </div>
            </div>
          {/if}
          <label>{t('contest.nr')}
            <input class="mono short" type="number" min="1" max="9999" value={app.settings?.contest_nr ?? 1} onchange={(e) => app.setContestNr(Number((e.currentTarget as HTMLInputElement).value))} />
            <small class="muted">{t('contest.nr.help')}</small>
          </label>
          <small class="muted">{t('contest.later')}</small>
          {#if app.settings}<ContestProfiles />{/if}
        {/if}

        {#if section === 'log'}
          <h3>{t('link.title')}</h3>
          <p class="muted">{t('link.intro')}</p>
          <h3>{t('link.adif.title')}</h3>
          <label class="check"><input type="checkbox" bind:checked={draft.link.adif_enabled} /> {t('link.adif.on')}</label>
          <small class="muted">{t('link.adif.help')}</small>
          <div class="row">
            <label>{t('link.host')} <input type="text" bind:value={draft.link.adif_host} disabled={!draft.link.adif_enabled} /></label>
            <label>{t('link.port')} <input type="number" min="1" max="65535" bind:value={draft.link.adif_port} disabled={!draft.link.adif_enabled} /></label>
          </div>
          <h3>{t('link.n1mm.title')}</h3>
          <label class="check"><input type="checkbox" bind:checked={draft.link.n1mm_enabled} /> {t('link.n1mm.on')}</label>
          <small class="muted">{t('link.n1mm.help')}</small>
          <label>{t('link.n1mm.targets')} <input type="text" bind:value={draft.link.n1mm_targets} disabled={!draft.link.n1mm_enabled} /></label>
          <small class="muted">{t('link.warning')}</small>
          <div class="row">
            <button class="btn" type="button" disabled={!draft.link.adif_enabled && !draft.link.n1mm_enabled} onclick={() => app.linkResend(draft)}>{t('link.resend')}</button>
          </div>
          <small class="muted">{t('link.resend.help')}</small>
          {#if app.linkResult}
            <p role="status">{app.linkResult.errors.length ? t('link.result.err', { n: app.linkResult.errors.length, list: app.linkResult.errors.join('; ') }) : t('link.result.ok', { n: app.linkResult.sent })}</p>
          {/if}

          <h3>{t('lookup.title')}</h3>
          <p class="muted">{t('lookup.intro')}</p>
          <label class="check"><input type="checkbox" bind:checked={draft.lookup.qrz_enabled} /> {t('lookup.on')}</label>
          <label>{t('lookup.user')} <input type="text" autocomplete="off" bind:value={draft.lookup.qrz_user} disabled={!draft.lookup.qrz_enabled} /></label>
          <label>{t('lookup.password')}
            <input type="password" autocomplete="new-password" placeholder={app.lookupState?.has_password ? t('lookup.password.stored') : ''} bind:value={qrzPassword} disabled={!draft.lookup.qrz_enabled} />
          </label>
          <small class="muted">{t('lookup.password.help')}</small>
          <div class="row">
            <button class="btn" type="button" disabled={!draft.lookup.qrz_enabled || !draft.lookup.qrz_user.trim() || (!qrzPassword && !app.lookupState?.has_password)} onclick={async () => { await app.lookupTestNow(draft, qrzPassword); qrzPassword = ''; }}>{t('lookup.test')}</button>
            <button class="btn" type="button" disabled={!app.lookupState?.has_password} onclick={() => app.lookupForget()}>{t('lookup.forget')}</button>
          </div>
          {#if app.lookupState?.test === 'busy'}<p role="status">{t('lookup.busy')}</p>{/if}
          {#if app.lookupState?.test === 'ok'}<p role="status">{t('lookup.ok')}</p>{/if}
          {#if app.lookupState?.test === 'ok_limited'}<p role="status">{t('lookup.ok_limited')}</p>{/if}
          {#if app.lookupState?.test === 'fail'}<p role="alert" class="warn">{app.lookupState.error}</p>{/if}
          <small class="muted">{t('lookup.privacy')}</small>
        {/if}

        {#if section === 'updates'}
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
          <h3>{t('wiz.again')}</h3>
          <p class="muted">{t('wiz.again.help')}</p>
          <div class="row"><button class="btn" type="button" onclick={() => { app.settingsOpen = false; app.wizardOpen = true; }}>{t('wiz.again')}</button></div>
        {/if}
      </div>
    </div>

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
  .dlg { background: var(--panel); border: 1px solid var(--line); border-radius: 14px; padding: 20px; width: min(820px, 96vw); height: min(720px, 92vh); display: flex; flex-direction: column; gap: 10px; }
  .body { flex: 1; min-height: 0; display: grid; grid-template-columns: 180px 1fr; gap: 16px; }
  .nav { display: flex; flex-direction: column; gap: 4px; border-right: 1px solid var(--line); padding-right: 12px; overflow: auto; }
  .nav button { text-align: left; background: none; border: 1px solid transparent; border-radius: 8px; padding: 8px 10px; color: var(--text); font-size: 15px; cursor: pointer; }
  .nav button:hover { background: var(--panel2); }
  .nav button.on { background: var(--panel2); border-color: var(--accent); color: var(--accent); font-weight: 600; }
  .pane { overflow: auto; display: flex; flex-direction: column; gap: 10px; padding-right: 4px; min-width: 0; }
  .group { display: flex; flex-direction: column; gap: 6px; padding: 8px 0; border-top: 1px solid var(--line); }
  .seg { display: inline-flex; gap: 0; }
  .seg button { background: var(--panel2); border: 1px solid var(--line); padding: 6px 14px; color: var(--text); cursor: pointer; font-size: 14px; }
  .seg button:first-child { border-radius: 8px 0 0 8px; }
  .seg button:last-child { border-radius: 0 8px 8px 0; }
  .seg button.on { border-color: var(--accent); color: var(--accent); font-weight: 600; }
  @media (max-width: 640px) { .body { grid-template-columns: 1fr; } .nav { flex-direction: row; flex-wrap: wrap; border-right: none; border-bottom: 1px solid var(--line); padding: 0 0 8px; } }
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
