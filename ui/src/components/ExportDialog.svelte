<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
  import { ADIF_CONTEST_ID_SUGGESTIONS } from '../lib/contest';
  import type { CheckIssue, LogFilter } from '../lib/types';

  /** Which part of the log: the real log, one contest run, or the test log. */
  let scope = $state<'all' | 'activation' | 'since' | 'contest' | 'test'>(app.activation ? 'activation' : 'all');
  let issues = $state<CheckIssue[] | null>(null);
  let from = $state('');
  let to = $state('');
  let band = $state('');
  let contestId = $state(ADIF_CONTEST_ID_SUGGESTIONS[app.settings?.contest.active ?? ''] ?? '');
  let result = $state<{ path: string; count: number } | null>(null);
  let busy = $state(false);

  const BANDS = ['160m', '80m', '60m', '40m', '30m', '20m', '17m', '15m', '12m', '10m', '6m', '2m'];

  /** "2026-11-08T14:30" typed as UTC to seconds since 1970. */
  function ts(v: string): number | null {
    if (!v) return null;
    const ms = Date.parse(v + ':00Z');
    return Number.isNaN(ms) ? null : Math.floor(ms / 1000);
  }

  function filter(): LogFilter {
    const f: LogFilter = { from_ts: ts(from), to_ts: ts(to), band, test: scope === 'test' };
    if (scope === 'activation' && app.activation) f.activation = app.activation.id;
    if (scope === 'since') f.not_exported = true;
    if (scope === 'contest') {
      f.contest = app.contestId;
      f.session = app.contestSession;
    }
    return f;
  }

  const errors = $derived(issues?.filter((i) => i.severity === 'error') ?? []);
  const warnings = $derived(issues?.filter((i) => i.severity === 'warning') ?? []);
  /** The grid that will be in the file, in large letters so a wrong one is seen at once. */
  const gridShown = $derived(app.activation?.grid || app.profile?.grid || app.settings?.station.locator || '');

  async function check() {
    busy = true;
    result = null;
    try {
      issues = await app.checkLog(filter());
    } catch (e) {
      app.error = String((e as { message?: string })?.message ?? e);
    } finally {
      busy = false;
    }
  }

  async function run() {
    busy = true;
    try {
      const r = await app.exportLog(filter(), contestId);
      result = { path: r.path, count: r.count };
      if (r.path === '(browser)') {
        // The browser stand-in has no CWLD folder: offer the file as a download.
        const a = document.createElement('a');
        a.href = URL.createObjectURL(new Blob([r.text], { type: 'text/plain' }));
        a.download = 'cwld-log.adi';
        a.click();
      }
    } catch (e) {
      app.error = String((e as { message?: string })?.message ?? e);
    } finally {
      busy = false;
    }
  }
</script>

<div class="scrim" role="presentation">
  <div class="dlg" role="dialog" aria-modal="true" aria-labelledby="ex-title">
    <h2 id="ex-title">{t('export.title')}</h2>
    <p class="muted">{t('export.intro')}</p>
    <label>{t('export.scope')}
      <select bind:value={scope} onchange={() => (issues = null)}>
        <option value="activation" disabled={!app.activation}>{t('export.scope.activation')}</option>
        <option value="since">{t('export.scope.since')}</option>
        <option value="all">{t('export.scope.all')}</option>
        <option value="contest" disabled={!app.settings?.contest_mode}>{t('export.scope.contest')}</option>
        <option value="test">{t('export.scope.test')}</option>
      </select>
    </label>
    <div class="grid">
      <label>{t('export.from')}<input type="datetime-local" bind:value={from} /></label>
      <label>{t('export.to')}<input type="datetime-local" bind:value={to} /></label>
    </div>
    <p class="muted small">{t('export.utc')}</p>
    <label>{t('export.band')}
      <select bind:value={band}>
        <option value="">{t('export.band.all')}</option>
        {#each BANDS as b}<option value={b}>{b}</option>{/each}
      </select>
    </label>
    <label>{t('export.contestid')}
      <input class="mono" bind:value={contestId} />
      <span class="muted small">{t('export.contestid.help')}</span>
    </label>
    <div class="gridbox" title={t('export.grid')}>
      <span class="muted small">{t('export.grid')}</span>
      <span class="biggrid mono">{gridShown || '?'}</span>
    </div>
    {#if issues}
      {#if !issues.length}<p class="ok" role="status">{t('export.check.ok')}</p>{/if}
      {#if errors.length}
        <div class="issues bad" role="alert">
          <strong>{t('export.check.errors', { n: String(errors.length) })}</strong>
          <ul>{#each errors.slice(0, 8) as i}<li>{t(`check.${i.code}`)} {i.detail}</li>{/each}</ul>
        </div>
      {/if}
      {#if warnings.length}
        <div class="issues warn">
          <strong>{t('export.check.warnings', { n: String(warnings.length) })}</strong>
          <ul>{#each warnings.slice(0, 8) as i}<li>{t(`check.${i.code}`)} {i.detail}</li>{/each}</ul>
        </div>
      {/if}
    {/if}
    {#if result}
      <p class="ok" role="status">{t('export.done', { n: String(result.count), path: result.path })}</p>
    {/if}
    <div class="act">
      <button class="btn" onclick={() => (app.exportOpen = false)}>{t('export.close')}</button>
      <button class="btn" disabled={busy} onclick={check}>{t('export.check')}</button>
      <button class="btn primary" disabled={busy || issues === null || errors.length > 0} title={issues === null ? t('export.check.first') : ''} onclick={run}>{t('export.go')}</button>
    </div>
  </div>
</div>

<style>
  .scrim { position: fixed; inset: 0; background: rgba(0, 0, 0, 0.6); display: grid; place-items: center; z-index: 30; }
  .dlg { background: var(--panel); border: 1px solid var(--line); border-radius: 14px; padding: 18px; width: min(520px, 94vw); display: flex; flex-direction: column; gap: 10px; }
  h2 { margin: 0; font-size: 18px; }
  p { margin: 0; }
  .small { font-size: 12px; }
  .grid { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
  label { display: flex; flex-direction: column; font-size: 12px; color: var(--muted); gap: 2px; }
  input, select { background: var(--panel2); border: 1px solid var(--line); border-radius: 8px; padding: 6px 8px; font-size: 15px; color: var(--text); }
  .gridbox { display: flex; align-items: baseline; gap: 12px; }
  .biggrid { font-size: 34px; font-weight: 800; letter-spacing: 0.08em; }
  .issues { border-radius: 8px; padding: 8px 10px; font-size: 13px; }
  .issues ul { margin: 4px 0 0 18px; padding: 0; }
  .issues.bad { border: 1px solid var(--live); color: var(--live); }
  .issues.warn { border: 1px solid var(--warn); color: var(--warn); }
  .ok { color: var(--listen); word-break: break-all; }
  .act { display: flex; gap: 8px; justify-content: flex-end; }
</style>
