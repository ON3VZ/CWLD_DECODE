<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
  import { slotLabel } from '../lib/texts';
  import ContestEntry from './ContestEntry.svelte';
  import { expand } from '../lib/template';

  const canSend = $derived(app.canSend);
  const busy = $derived(app.status.transmitting || app.practiceBusy);

  function chipMissing(template: string): string[] {
    return expand(template, app.vars).missing;
  }
  /** Right click on a chip or key opens its text. The buttons are never really disabled, else a right click does nothing. */
  function editText(e: MouseEvent, id: string) {
    e.preventDefault();
    app.openTextEditor(id);
  }
  function tip(template: string): string {
    return `${expand(template, app.vars).text}\n${t('texts.rightclick')}`;
  }
  const cqSlot = $derived(app.cqRepeatSlot);
  function submit(e: Event) {
    e.preventDefault();
    app.send(app.draft);
  }
</script>

<div class="composer">
  {#if !app.settings?.contest_mode}
  <div class="chips" role="group" aria-label="Chips">
    {#each app.chipList as c (c.id)}
      {@const missing = chipMissing(c.template)}
      <button class="btn chip" class:own={c.own} aria-disabled={!canSend} title={tip(c.template)} onclick={() => canSend && app.sendChip(c.template)} oncontextmenu={(e) => editText(e, c.id)}>
        {slotLabel(c, t)}{#if missing.length}<span class="dotwarn" title={t('composer.missing', { vars: missing.join(', ') })}></span>{/if}
      </button>
    {/each}
    <button class="btn chip edit" onclick={() => app.openTextEditor(null)} title={t('texts.open.help')} aria-label={t('texts.open.help')}>✎</button>
  </div>
  {/if}

  {#if app.settings?.contest_mode}
    <ContestEntry />
  {/if}
  {#if !app.settings?.contest_mode}
  <div class="chips pile" role="group" aria-label={t('pileup.title')}>
    {#if !app.settings?.contest_mode}<span class="muted label">{t('pileup.title')}</span>{/if}
    {#each app.settings?.contest_mode ? [] : app.pileupList as c (c.id)}
      {@const missing = chipMissing(c.template)}
      <button class="btn chip" aria-disabled={!canSend} title={tip(c.template)} onclick={() => canSend && app.sendChip(c.template)} oncontextmenu={(e) => editText(e, c.id)}>
        <kbd>{c.fkey}</kbd> {slotLabel(c, t)}{#if missing.length}<span class="dotwarn" title={t('composer.missing', { vars: missing.join(', ') })}></span>{/if}
      </button>
    {/each}
    <button class="btn chip contest" class:on={app.settings?.contest_mode} aria-pressed={app.settings?.contest_mode ?? false} title={t('contest.chip.help')} onclick={() => app.requestContest(!app.settings?.contest_mode)}>
      {t('contest.chip')}{#if app.settings?.contest_mode && app.serialUsed}&nbsp;<span class="mono">{app.serialText}</span>{/if}
    </button>
  </div>
  {/if}

  <form class="line" onsubmit={submit}>
    <input
      class="mono"
      bind:value={app.draft}
      placeholder={!app.viewingLive ? t('composer.placeholder.old') : canSend ? t('composer.placeholder') : app.swrAlert ? t('composer.placeholder.swr') : t('composer.placeholder.listen')}
      aria-label={t('composer.placeholder')}
      autocomplete="off"
      autocapitalize="characters"
      oninput={() => (app.draft = app.draft.toUpperCase())} />
    <button class="btn primary" type="submit" disabled={!canSend || !app.draft.trim()}>{t('composer.send')}</button>
    <button
      class="btn cqrep"
      class:on={!!app.cqRepeat}
      type="button"
      disabled={!app.cqRepeat && (!canSend || !cqSlot)}
      aria-pressed={!!app.cqRepeat}
      onclick={() => app.toggleCqRepeat()}
      oncontextmenu={(e) => cqSlot && editText(e, cqSlot.id)}
      title={cqSlot ? `${t('cq.repeat.help', { s: String(app.sendTexts.cq_pause_s), max: String(app.sendTexts.cq_max) })}\n${expand(cqSlot.template, app.vars).text}` : t('cq.repeat.none')}>
      {app.cqRepeat ? t('cq.repeat.stop') : t('cq.repeat')} (Alt+R)
    </button>
    <button class="btn" type="button" onclick={() => (app.qOpen = !app.qOpen)} title={`${t('qlib.open.hint')} (Ctrl+Q)`} aria-label={t('qlib.open.hint')} aria-pressed={app.qOpen}>Q</button>
    {#if app.cqHome && Math.abs(app.cqHome.hz - app.status.frequency_hz) >= 100}
      <button class="btn" type="button" disabled={!app.status.connected} onclick={() => app.returnToCq()} title={t('cq.freq.help')}>{t('cq.freq', { khz: (app.cqHome.hz / 1000).toFixed(1) })} (Alt+Q)</button>
    {/if}
    <button class="btn danger" type="button" disabled={!busy && app.policy !== 'live'} onclick={() => app.stop()} title={t('composer.stop.help')}>{t('composer.stop')} (Esc)</button>
    <button class="btn danger" type="button" disabled={!canSend} onclick={() => app.stopAndQrx()} title={t('composer.stopqrx.help')}>{t('composer.stopqrx')} (Esc Esc)</button>
    <button class="btn" type="button" disabled={!canSend} onclick={() => app.sendError()} title={t('composer.error.help')}>{t('composer.error')}</button>
    {#if app.settings?.contest_mode}
    <label class="wpm inline">
      {t('composer.wpm')}
      <input
        type="range" min="5" max="60"
        value={app.status.cw_wpm}
        onchange={(e) => app.setSpeed(Number((e.currentTarget as HTMLInputElement).value))} />
      <span class="mono">{app.status.cw_wpm} WPM</span>
    </label>
    {/if}
  </form>

  {#if app.cqRepeat}
    <div class="cqline" role="status" aria-live="polite">
      {#if app.cqRepeat.phase === 'pause'}
        <span>{t('cq.repeat.listening', { s: String(app.cqSecondsLeft), n: String(app.cqRepeat.count), max: String(app.cqRepeat.max) })}</span>
      {:else}
        <span>{t('cq.repeat.calling', { n: String(app.cqRepeat.count), max: String(app.cqRepeat.max) })}</span>
      {/if}
      <span class="muted">{t('cq.repeat.stops')}</span>
    </div>
  {:else if app.cqStopped}
    <div class="cqline off" role="status">
      <span class="muted">{t(`cq.repeat.stopped.${app.cqStopped}`)}</span>
      <button class="x" onclick={() => (app.cqStopped = null)} aria-label={t('warning.close')}>×</button>
    </div>
  {/if}

  {#if app.held.length}
    <div class="queue held" aria-label={t('paddle.wait.title')}>
      <span class="muted">{t('paddle.wait.title')}:</span>
      {#each app.held as block}<span class="block mono">{block}</span>{/each}
      <span class="muted">{t('paddle.wait.help')}</span>
    </div>
  {/if}

  {#if app.queue.length}
    <div class="queue" aria-label={t('queue.title')}>
      <span class="muted">{t('queue.title')}:</span>
      {#each app.queue as block}<span class="block mono">{block.trim()}</span>{/each}
    </div>
  {/if}

  {#if app.warning}
    <div class="warn" role="alert">
      <span>{t(`warning.${app.warning}`) === `warning.${app.warning}` ? app.warning : t(`warning.${app.warning}`)}</span>
      <button class="x" onclick={() => app.clearWarning()} aria-label={t('warning.close')}>×</button>
    </div>
  {/if}

  <div class="foot">
    {#if !app.settings?.contest_mode}
    <label class="wpm">
      {t('composer.wpm')}
      <input
        type="range" min="5" max="60"
        value={app.status.cw_wpm}
        onchange={(e) => app.setSpeed(Number((e.currentTarget as HTMLInputElement).value))} />
      <span class="mono">{app.status.cw_wpm} WPM</span>
    </label>
    {/if}
    {#if busy}<span class="muted">{t('composer.sending')}...</span>{/if}
    {#if app.error}<span class="err" role="alert">{app.error}</span>{/if}
    {#if app.notice && !app.error}<span class="muted">{app.notice}</span>{/if}
  </div>
</div>

<style>
  .composer { display: flex; flex-direction: column; gap: 8px; background: var(--panel); border: 1px solid var(--line); border-radius: 12px; padding: 10px; }
  .chips { display: flex; flex-wrap: wrap; gap: 6px; }
  .chip { border-radius: 999px; position: relative; }
  .pile { align-items: center; }
  .label { font-size: 12px; text-transform: uppercase; letter-spacing: 0.06em; }
  kbd { font: inherit; font-size: 11px; color: var(--muted); border: 1px solid var(--line); border-radius: 4px; padding: 0 4px; margin-right: 2px; }
  .contest.on { border-color: var(--accent); color: var(--accent); }
  .chip.own { border-style: dashed; }
  .chip.edit { color: var(--muted); padding-left: 10px; padding-right: 10px; }
  .cqrep { white-space: nowrap; }
  .cqrep.on { background: var(--warn); color: var(--bg); border-color: var(--warn); font-weight: 700; }
  .cqline { display: flex; gap: 12px; align-items: center; flex-wrap: wrap; font-size: 14px; border: 1px solid var(--warn); border-radius: 8px; padding: 4px 10px; }
  .cqline.off { border-color: var(--line); justify-content: space-between; }
  .held .block { border-color: var(--warn); }
  .dotwarn { display: inline-block; width: 7px; height: 7px; border-radius: 50%; background: var(--warn); margin-left: 6px; }
  .line { display: flex; gap: 8px; }
  .line input { flex: 1; background: var(--panel2); border: 1px solid var(--line); border-radius: 8px; padding: 8px 10px; font-size: 17px; }
  .foot { display: flex; gap: 14px; align-items: center; flex-wrap: wrap; font-size: 14px; }
  .wpm { display: flex; align-items: center; gap: 8px; }
  .wpm.inline { font-size: 13px; white-space: nowrap; }
  .wpm.inline input[type='range'] { width: 90px; }
  .line .wpm input { flex: none; padding: 0; border: none; background: none; }
  .queue { display: flex; flex-wrap: wrap; gap: 6px; align-items: center; font-size: 13px; }
  .block { background: var(--panel2); border: 1px dashed var(--line); border-radius: 6px; padding: 1px 8px; }
  .warn { display: flex; justify-content: space-between; gap: 8px; background: color-mix(in srgb, var(--warn) 20%, var(--panel)); border: 1px solid var(--warn); border-radius: 8px; padding: 6px 10px; font-size: 14px; }
  .x { background: none; border: none; font-size: 18px; line-height: 1; color: var(--text); }
  .err { color: var(--live); font-weight: 600; }
</style>
