<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
  import { defaultChips, pileupChips } from '../lib/demo';
  import ContestEntry from './ContestEntry.svelte';
  import { expand } from '../lib/template';

  const canSend = $derived(app.canSend);
  const busy = $derived(app.status.transmitting || app.practiceBusy);

  function chipMissing(template: string): string[] {
    return expand(template, app.vars).missing;
  }
  function submit(e: Event) {
    e.preventDefault();
    app.send(app.draft);
  }
</script>

<div class="composer">
  {#if !app.settings?.contest_mode}
  <div class="chips" role="group" aria-label="Chips">
    {#each defaultChips as c (c.id)}
      {@const missing = chipMissing(c.template)}
      <button class="btn chip" disabled={!canSend} title={expand(c.template, app.vars).text} onclick={() => app.sendChip(c.template)}>
        {t(c.key)}{#if missing.length}<span class="dotwarn" title={t('composer.missing', { vars: missing.join(', ') })}></span>{/if}
      </button>
    {/each}
  </div>
  {/if}

  {#if app.settings?.contest_mode}
    <ContestEntry />
  {/if}
  {#if !app.settings?.contest_mode}
  <div class="chips pile" role="group" aria-label={t('pileup.title')}>
    {#if !app.settings?.contest_mode}<span class="muted label">{t('pileup.title')}</span>{/if}
    {#each app.settings?.contest_mode ? [] : pileupChips as c (c.id)}
      {@const missing = chipMissing(c.template)}
      <button class="btn chip" disabled={!canSend} title={expand(c.template, app.vars).text} onclick={() => app.sendChip(c.template)}>
        <kbd>{c.fkey}</kbd> {t(c.key)}{#if missing.length}<span class="dotwarn" title={t('composer.missing', { vars: missing.join(', ') })}></span>{/if}
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
    <button class="btn" type="button" onclick={() => (app.qOpen = !app.qOpen)} title={`${t('qlib.open.hint')} (Ctrl+Q)`} aria-label={t('qlib.open.hint')} aria-pressed={app.qOpen}>Q</button>
    <button class="btn danger" type="button" disabled={!busy && app.policy !== 'live'} onclick={() => app.stop()}>{t('composer.stop')} (Esc)</button>
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
