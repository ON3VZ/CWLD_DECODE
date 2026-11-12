<script lang="ts">
  import { onMount } from 'svelte';
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
  import { allProfiles, plain, recvApplies } from '../lib/contest';
  import { rates } from '../lib/contestlog';
  import { expand } from '../lib/template';
  import { slotLabel } from '../lib/texts';

  let now = $state(Math.floor(Date.now() / 1000));
  onMount(() => {
    const id = setInterval(() => (now = Math.floor(Date.now() / 1000)), 15000);
    return () => clearInterval(id);
  });

  const r = $derived(rates(app.qsoTimes, now));
  const recv = $derived(app.recvField);
  const extraOn = $derived(!!recv && recvApplies(recv, app.qso.call));
  const sug = $derived(app.suggestion);
  const hasSug = $derived(!!(sug.rst || sug.nr || sug.extra));
  const dupe = $derived(app.dupeInfo?.dupe ?? null);

  function typeCall(e: Event) {
    const v = (e.currentTarget as HTMLInputElement).value.toUpperCase();
    app.qso = { ...app.qso, call: v };
    app.callLocked = true;
    app.noteCallChange();
  }

  function keydown(e: KeyboardEvent, next?: string) {
    if (e.key === 'Enter') {
      e.preventDefault();
      void app.pressEnter();
    } else if (e.key === ' ' && next) {
      // Space moves to the next field, as in N1MM Logger+.
      e.preventDefault();
      document.getElementById(next)?.focus();
    } else if (e.key === 'Tab' && hasSug && !e.shiftKey) {
      const el = e.currentTarget as HTMLInputElement;
      if (!el.value) {
        e.preventDefault();
        app.acceptSuggestion();
      }
    }
  }

  function time(ts: number): string {
    return new Date(ts * 1000).toISOString().slice(11, 16) + ' UTC';
  }
</script>

<div class="ce" class:test={app.testMode}>
  <div class="bar">
    <button class="btn chip contest on" aria-pressed="true" title={t('contest.chip.help')} onclick={() => app.requestContest(false)}>{t('contest.chip')} &times;</button>
    <div class="seg" role="group" aria-label={t('contest.runsp')}>
      <button class:on={app.runMode === 'run'} aria-pressed={app.runMode === 'run'} title={t('contest.run.help')} onclick={() => app.setRunMode('run')}>Run</button>
      <button class:on={app.runMode === 'sp'} aria-pressed={app.runMode === 'sp'} title={t('contest.sp.help')} onclick={() => app.setRunMode('sp')}>S&amp;P</button>
    </div>
    <span class="fk" title={t('contest.runsp.key')}>Ctrl+R</span>
    <button class="btn testbtn" class:on={app.testMode} aria-pressed={app.testMode} title={t('contest.test.help')} onclick={() => app.setTestMode(!app.testMode)}>{t('contest.test')}</button>
    {#if app.testMode}
      <button class="link" onclick={() => app.clearTestLog()} title={t('contest.test.clear.help')}>{t('contest.test.clear')}</button>
    {/if}
    <select class="profile" aria-label={t('contest.profile')} title={t('contest.profile')} value={app.settings?.contest.active ?? ''} onchange={(e) => app.selectContestProfile((e.currentTarget as HTMLSelectElement).value)}>
      <option value="">{t('contest.profile.none')}</option>
      {#each allProfiles(app.settings!.contest) as p (p.id)}
        <option value={p.id}>{plain(p.name, t)}</option>
      {/each}
    </select>
    {#if app.contestProfile && app.contestProfile.sent.length > 1}
      <select class="profile" aria-label={t('contest.sent')} title={t('contest.sent')} value={app.settings?.contest.sent_index ?? 0} onchange={(e) => app.setContestSent(Number((e.currentTarget as HTMLSelectElement).value))}>
        {#each app.contestProfile.sent as s, i (i)}
          <option value={i}>{plain(s.label, t)}</option>
        {/each}
      </select>
    {/if}
  </div>

  {#if app.contestNote}
    <div class="note" role="status">
      <span>{app.contestNote}</span>
      <button class="btn" type="button" onclick={() => (app.contestNote = null)} aria-label={t('contest.note.close')}>×</button>
    </div>
  {/if}

  <div class="entry">
    <label>{t('qso.call')}
      <input id="ce-call" class="mono call" class:isdupe={!!dupe} value={app.qso.call} oninput={typeCall} onkeydown={(e) => keydown(e, 'ce-rst')} autocomplete="off" autocapitalize="characters" />
    </label>
    <label>{t('contest.rcvd.rst')}
      <input id="ce-rst" class="mono w60" bind:value={app.entry.rst} placeholder={sug.rst ?? ''} onkeydown={(e) => keydown(e, app.serialUsed ? 'ce-nr' : extraOn ? 'ce-extra' : undefined)} autocomplete="off" />
    </label>
    {#if app.serialUsed}
      <label>{t('contest.rcvd.nr')}
        <input id="ce-nr" class="mono w70" bind:value={app.entry.nr} placeholder={sug.nr ?? ''} onkeydown={(e) => keydown(e, extraOn ? 'ce-extra' : undefined)} autocomplete="off" inputmode="numeric" />
      </label>
    {/if}
    {#if recv}
      <label class:off={!extraOn}>{plain(recv.label, t)}
        <input id="ce-extra" class="mono w90" list="ce-known" bind:value={app.entry.extra} disabled={!extraOn} placeholder={sug.extra ?? ''} onkeydown={(e) => keydown(e)} autocomplete="off" autocapitalize="characters" />
      </label>
      <datalist id="ce-known">{#each app.knownExtra as k}<option value={k}></option>{/each}</datalist>
    {/if}
    {#if app.serialUsed}
      <div class="mine"><span class="lbl">{t('contest.mynr')}</span>
        <span class="nr mono" class:testnr={app.testMode}>{app.serialText}</span>
      </div>
    {/if}
    <button class="btn primary logbtn" onclick={() => app.logQso()} disabled={!app.qso.call.trim()} title={t('contest.log.help')}>{t('contest.log')}</button>
  <div class="info" aria-live="polite">
    {#if dupe}
      <strong class="dupe">{t('contest.dupe')}</strong>
      <span class="muted">{t('contest.dupe.at', { time: time(dupe.ts), band: dupe.band })}</span>
    {:else if app.dupeInfo?.other_bands.length}
      <span class="muted">{t('contest.worked.other', { bands: app.dupeInfo.other_bands.join(', ') })}</span>
    {/if}
    <span class="muted">{app.band || '-'}</span>
    <span class="muted">{t('contest.count', { n: String(app.qsoCount) })}</span>
    <span class="muted" title={t('contest.rate.help')}>{t('contest.rate', { r10: String(r.rate10), r60: String(r.rate60) })}</span>
    {#if app.testMode}<strong class="testtag">{t('contest.test.on')}</strong>{/if}
  </div>
  </div>

  {#if hasSug}
    <button class="sug" onclick={() => app.acceptSuggestion()} title={t('contest.sug.help')}>
      <span class="muted">{t('contest.sug')}</span>
      <span class="mono">{[sug.rst, sug.nr, sug.extra].filter(Boolean).join(' ')}</span>
      <span class="fk">Tab</span>
    </button>
  {/if}

  <div class="keys" role="group" aria-label={t('contest.keys')}>
    {#each app.contestKeyList as k (k.key)}
      {@const miss = expand(k.template, app.vars).missing}
      <button class="btn key" class:changed={k.changed} aria-disabled={!app.canSend} title={`${expand(k.template, app.vars).text}\n${t('texts.rightclick')}`} onclick={() => app.canSend && app.pressContestKey(k.key)} oncontextmenu={(e) => { e.preventDefault(); app.openTextEditor(k.id); }}>
        <span class="fk">{k.key}</span>
        <span>{slotLabel(k, t)}</span>
        {#if miss.length}<span class="dotwarn" title={t('composer.missing', { vars: miss.join(', ') })}></span>{/if}
      </button>
    {/each}
    <button class="btn key edit" onclick={() => app.openTextEditor(app.contestKeyList[0]?.id ?? null)} title={t('texts.open.help')} aria-label={t('texts.open.help')}>
      <span class="fk">&nbsp;</span><span>✎</span>
    </button>
  </div>
</div>

<style>
  .ce { display: flex; flex-direction: column; gap: 6px; }
  .ce.test { outline: 1px dashed var(--warn); outline-offset: 4px; border-radius: 8px; }
  .bar { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; }
  .seg { display: inline-flex; border: 1px solid var(--line); border-radius: 8px; overflow: hidden; }
  .seg button { background: var(--panel2); border: none; color: var(--text); padding: 4px 12px; }
  .seg button.on { background: var(--accent); color: var(--bg); font-weight: 700; }
  .fk { font-size: 10px; color: var(--muted); opacity: 0.75; letter-spacing: 0.04em; }
  .testbtn.on { background: var(--warn); color: var(--bg); border-color: var(--warn); font-weight: 700; }
  .link { background: none; border: none; color: var(--accent); text-decoration: underline; font-size: 12px; padding: 0; }
  .profile { background: var(--panel2); border: 1px solid var(--line); border-radius: 8px; padding: 4px 8px; color: var(--text); max-width: 16em; }
  .note { display: flex; align-items: center; gap: 8px; font-size: 12px; background: color-mix(in srgb, var(--warn) 18%, var(--bg)); border: 1px solid var(--warn); border-radius: 8px; padding: 4px 10px; }
  .note span { flex: 1; }
  .info { display: flex; flex-wrap: wrap; gap: 2px 12px; align-items: baseline; font-size: 12px; margin-left: auto; max-width: 22em; justify-content: flex-end; }
  .contest.on { border-color: var(--accent); color: var(--accent); border-radius: 999px; }
  .dupe { color: var(--live); font-size: 15px; letter-spacing: 0.06em; }
  .testtag { color: var(--warn); letter-spacing: 0.08em; }
  .entry { display: flex; flex-wrap: wrap; gap: 10px; align-items: flex-end; }
  .entry label { display: flex; flex-direction: column; font-size: 11px; color: var(--muted); gap: 2px; }
  .mine { display: flex; flex-direction: column; font-size: 11px; color: var(--muted); gap: 2px; }
  .entry label.off { opacity: 0.4; }
  .entry input { background: var(--panel2); border: 1px solid var(--line); border-radius: 8px; padding: 4px 8px; font-size: 17px; color: var(--text); }
  .entry input::placeholder { color: var(--muted); opacity: 0.55; }
  .call { width: 9.5em; text-transform: uppercase; }
  .call.isdupe { border-color: var(--live); }
  .w60 { width: 4.2em; } .w70 { width: 5em; } .w90 { width: 6.5em; }
  .nr { font-size: 20px; font-weight: 700; color: var(--accent); padding: 5px 4px; }
  .nr.testnr { color: var(--warn); }
  .logbtn { align-self: flex-end; }
  .sug { display: inline-flex; gap: 8px; align-items: baseline; align-self: flex-start; background: var(--panel2); border: 1px dashed var(--line); border-radius: 999px; padding: 3px 12px; color: var(--text); font-size: 14px; }
  .keys { display: grid; grid-template-columns: repeat(auto-fit, minmax(6em, 1fr)); gap: 6px; }
  .key { display: flex; flex-direction: column; align-items: center; gap: 0; padding: 3px 6px 5px; position: relative; font-size: 14px; }
  .key .fk { align-self: flex-end; line-height: 1; }
  .key.changed .fk { color: var(--accent); opacity: 1; }
  .key.edit { color: var(--muted); }
  .dotwarn { position: absolute; top: 6px; left: 6px; width: 7px; height: 7px; border-radius: 50%; background: var(--warn); }
</style>
