<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
  import { ADIF_CONTEST_ID_SUGGESTIONS, allProfiles } from '../lib/contest';
  import { csvText, COLUMNS, DEFAULT_COLUMNS, loadColumns, saveColumns, sortRows, summary, type SortKey } from '../lib/logbook';
  import type { LogFilter, LogQso } from '../lib/types';

  const BANDS = ['160m', '80m', '60m', '40m', '30m', '20m', '17m', '15m', '12m', '10m', '6m', '2m'];

  // ---- Filters (all of them go to the core, so the export holds exactly what is shown) ----
  let search = $state('');
  let from = $state('');
  let to = $state('');
  let band = $state('');
  let contest = $state('');
  let activation = $state<number | null>(null);
  let notExported = $state(false);
  let testLog = $state(false);

  let rows = $state<LogQso[]>([]);
  /** All QSOs of the log, for the choices in the contest and activation lists. */
  let all = $state<LogQso[]>([]);
  let loading = $state(false);
  let sortKey = $state<SortKey>('ts');
  let sortUp = $state(false);
  let visible = $state<string[]>(loadColumns());
  let pickCols = $state(false);
  let problem = $state<string | null>(null);
  let result = $state<{ path: string; count: number } | null>(null);

  /** "2026-11-08" (a day, UTC) to seconds since 1970; `end`: the end of that day. */
  function day(v: string, end: boolean): number | null {
    if (!v) return null;
    const ms = Date.parse(`${v}T${end ? '23:59:59' : '00:00:00'}Z`);
    return Number.isNaN(ms) ? null : Math.floor(ms / 1000);
  }

  function filter(): LogFilter {
    return {
      search: search.trim(),
      from_ts: day(from, false),
      to_ts: day(to, true),
      band,
      contest,
      activation,
      not_exported: notExported,
      test: testLog,
      limit: 0,
    };
  }

  let timer: ReturnType<typeof setTimeout> | null = null;
  async function load() {
    loading = true;
    try {
      rows = await app.bookList(filter());
    } catch (e) {
      problem = say(e);
    } finally {
      loading = false;
    }
  }
  /** Typing in the search box waits a moment, the other filters act at once. */
  function soon(ms = 250) {
    if (timer) clearTimeout(timer);
    timer = setTimeout(load, ms);
  }

  $effect(() => {
    // Read every filter, so a change of any of them loads again.
    void [search, from, to, band, contest, activation, notExported, testLog];
    result = null;
    soon(search ? 250 : 0);
  });

  $effect(() => {
    void testLog;
    app.bookList({ test: testLog, limit: 0 }).then((r) => (all = r)).catch(() => {});
  });

  // After an edit or a delete in the edit window, show the new state.
  $effect(() => {
    void app.bookVersion;
    soon(0);
    app.bookList({ test: testLog, limit: 0 }).then((r) => (all = r)).catch(() => {});
  });

  const contests = $derived.by(() => {
    const names = new Map(app.settings ? allProfiles(app.settings.contest).map((p) => [p.id, p.name]) : []);
    return [...new Set(all.map((q) => q.contest).filter(Boolean))].sort().map((id) => ({ id, name: names.get(id) ?? id }));
  });
  const activations = $derived.by(() => {
    const seen = new Map<number, string>();
    for (const q of all) {
      if (!q.activation_id || seen.has(q.activation_id)) continue;
      const refs = [q.my_wwff, q.my_pota, q.my_sota].filter(Boolean).join(' ');
      const d = new Date(q.ts * 1000).toISOString().slice(0, 10);
      seen.set(q.activation_id, `${refs || q.my_grid || '#' + q.activation_id} (${d})`);
    }
    return [...seen.entries()].sort((a, b) => b[0] - a[0]);
  });

  const shown = $derived(sortRows(rows, sortKey, sortUp));
  const cols = $derived(COLUMNS.filter((c) => visible.includes(c.key)));
  const sum = $derived(summary(rows));
  const anyFilter = $derived(!!(search.trim() || from || to || band || contest || activation !== null || notExported));

  function sortBy(k: SortKey) {
    if (sortKey === k) sortUp = !sortUp;
    else {
      sortKey = k;
      sortUp = k !== 'ts';
    }
  }

  function toggleCol(k: string) {
    visible = visible.includes(k) ? visible.filter((x) => x !== k) : COLUMNS.map((c) => c.key).filter((x) => x === k || visible.includes(x));
    saveColumns(visible);
  }

  function clearFilters() {
    search = '';
    from = '';
    to = '';
    band = '';
    contest = '';
    activation = null;
    notExported = false;
  }

  function say(e: unknown): string {
    const f = e as { code?: string; message?: string };
    const key = f?.code ? `error.${f.code}` : '';
    const text = key ? t(key) : '';
    return text && text !== key ? text : String(f?.message ?? e);
  }

  /** Writes exactly the QSOs of this filter to an ADIF file, after the same check as the export window. */
  async function exportShown() {
    problem = null;
    result = null;
    const f = filter();
    try {
      const issues = await app.checkLog(f);
      const errors = issues.filter((i) => i.severity === 'error');
      if (errors.length) {
        problem = t('book.export.errors', { n: errors.length, list: errors.slice(0, 5).map((i) => `${t(`check.${i.code}`)} ${i.detail}`).join('; ') });
        return;
      }
      const r = await app.exportLog(f, ADIF_CONTEST_ID_SUGGESTIONS[contest] ?? '');
      result = { path: r.path, count: r.count };
      if (r.path === '(browser)') {
        const a = document.createElement('a');
        a.href = URL.createObjectURL(new Blob([r.text], { type: 'text/plain' }));
        a.download = 'cwld-log.adi';
        a.click();
      }
      void load();
    } catch (e) {
      problem = say(e);
    }
  }

  /** The QSOs as shown (same filter, same order) with the chosen columns, as a CSV file. Not marked as exported. */
  async function exportCsv() {
    problem = null;
    result = null;
    try {
      const text = csvText(shown, visible, (k) => t(`book.col.${k}`));
      const stamp = new Date().toISOString().slice(0, 16).replace(/[-:]/g, '').replace('T', '-');
      const name = `cwld-logbook-${stamp}.csv`;
      const path = await app.exportCsvFile(name, text);
      result = { path, count: shown.length };
      if (path === '(browser)') {
        const a = document.createElement('a');
        a.href = URL.createObjectURL(new Blob([text], { type: 'text/csv;charset=utf-8' }));
        a.download = name;
        a.click();
      }
    } catch (e) {
      problem = say(e);
    }
  }

  const close = () => (app.bookOpen = false);
</script>

<div class="scrim" role="presentation">
  <div class="book" role="dialog" aria-modal="true" aria-labelledby="book-title">
    <header>
      <h2 id="book-title">{t('book.title')}{#if testLog} <span class="testtag">{t('contest.test.on')}</span>{/if}</h2>
      <button class="btn" onclick={close} title={t('book.close.help')}>{t('book.close')}</button>
    </header>

    <div class="filters">
      <input class="search" type="search" placeholder={t('book.search')} aria-label={t('book.search')} bind:value={search} />
      <label>{t('book.from')} <input type="date" bind:value={from} /></label>
      <label>{t('book.to')} <input type="date" bind:value={to} /></label>
      <label>{t('book.band')}
        <select bind:value={band}>
          <option value="">{t('book.all')}</option>
          {#each BANDS as b}<option value={b}>{b}</option>{/each}
        </select>
      </label>
      <label>{t('book.contest')}
        <select bind:value={contest}>
          <option value="">{t('book.all')}</option>
          {#each contests as c (c.id)}<option value={c.id}>{c.name}</option>{/each}
        </select>
      </label>
      <label>{t('book.activation')}
        <select bind:value={activation}>
          <option value={null}>{t('book.all')}</option>
          {#each activations as [id, label] (id)}<option value={id}>{label}</option>{/each}
        </select>
      </label>
      <label class="check"><input type="checkbox" bind:checked={notExported} /> {t('book.not_exported')}</label>
      <label class="check"><input type="checkbox" bind:checked={testLog} /> {t('book.test_log')}</label>
      {#if anyFilter}<button class="link" onclick={clearFilters}>{t('book.clear')}</button>{/if}
    </div>

    <div class="tools">
      <span class="muted">{t('book.summary', { n: sum.qsos, calls: sum.calls, bands: sum.bands })}{#if loading} …{/if}</span>
      <button class="btn" onclick={() => (pickCols = !pickCols)} aria-expanded={pickCols} title={t('book.columns.help')}>{t('book.columns')}</button>
      <button class="btn" disabled={!rows.length} onclick={exportCsv} title={t('book.csv.help')}>{t('book.csv')}</button>
      <button class="btn primary" disabled={!rows.length} onclick={exportShown} title={t('book.export.help')}>{t('book.export', { n: rows.length })}</button>
    </div>
    {#if pickCols}
      <div class="colpick">
        {#each COLUMNS as c (c.key)}
          <label class="check"><input type="checkbox" checked={visible.includes(c.key)} onchange={() => toggleCol(c.key)} /> {t(`book.col.${c.key}`)}</label>
        {/each}
        <button class="link" onclick={() => { visible = [...DEFAULT_COLUMNS]; saveColumns(visible); }}>{t('book.columns.reset')}</button>
      </div>
    {/if}
    {#if problem}<div class="msg bad" role="alert">{problem}</div>{/if}
    {#if result}
      <div class="msg ok" role="status">
        {t('export.done', { n: String(result.count), path: result.path })}
        {#if result.path !== '(browser)'}<button class="link" onclick={() => app.openExportFolder().catch((e) => (problem = say(e)))}>{t('export.open_folder')}</button>{/if}
      </div>
    {/if}

    <div class="tablewrap">
      {#if !rows.length && !loading}
        <p class="muted empty">{anyFilter ? t('book.none_filtered') : t('book.none')}</p>
      {:else}
        <table>
          <thead>
            <tr>
              {#each cols as c (c.key)}
                <th class:num={c.num} aria-sort={sortKey === c.sort ? (sortUp ? 'ascending' : 'descending') : 'none'}>
                  {#if c.sort}<button class="th" onclick={() => sortBy(c.sort!)}>{t(`book.col.${c.key}`)}{#if sortKey === c.sort}<span class="arrow">{sortUp ? '▲' : '▼'}</span>{/if}</button>{:else}{t(`book.col.${c.key}`)}{/if}
                </th>
              {/each}
            </tr>
          </thead>
          <tbody>
            {#each shown as q (q.id)}
              <tr onclick={() => (app.editing = { ...q })} title={t('book.row.help')}>
                {#each cols as c (c.key)}<td class:num={c.num} class:mono={c.mono}>{c.value(q)}</td>{/each}
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </div>
  </div>
</div>

<style>
  .scrim { position: fixed; inset: 0; background: rgba(0, 0, 0, 0.6); display: grid; place-items: center; z-index: 25; }
  .book { background: var(--bg-2, var(--card, #111820)); color: var(--text); border: 1px solid var(--line); border-radius: 12px; width: min(1400px, calc(100vw - 32px)); height: calc(100vh - 48px); display: flex; flex-direction: column; gap: 10px; padding: 16px; box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5); }
  header { display: flex; align-items: center; justify-content: space-between; gap: 12px; }
  h2 { margin: 0; font-size: 18px; }
  .filters { display: flex; flex-wrap: wrap; gap: 8px 14px; align-items: center; font-size: 13px; }
  .filters label { display: inline-flex; align-items: center; gap: 6px; }
  .search { min-width: 220px; flex: 1 1 220px; }
  .tools { display: flex; align-items: center; gap: 10px; flex-wrap: wrap; font-size: 13px; }
  .tools .muted { flex: 1; }
  .colpick { display: flex; flex-wrap: wrap; gap: 6px 16px; padding: 8px 10px; border: 1px solid var(--line); border-radius: 8px; font-size: 13px; }
  .msg { padding: 6px 10px; border-radius: 6px; font-size: 13px; }
  .msg.bad { border: 1px solid var(--bad, #c0392b); color: var(--bad, #c0392b); }
  .msg.ok { border: 1px solid var(--line); }
  .msg .link { margin-left: 10px; }
  .tablewrap { flex: 1; overflow: auto; border: 1px solid var(--line); border-radius: 8px; min-height: 0; }
  table { border-collapse: collapse; width: 100%; font-size: 13px; }
  thead th { position: sticky; top: 0; background: var(--bg-2, var(--card, #111820)); text-align: left; font-weight: 600; padding: 6px 8px; border-bottom: 1px solid var(--line); white-space: nowrap; z-index: 1; }
  th.num, td.num { text-align: right; }
  .th { background: none; border: none; color: inherit; font: inherit; cursor: pointer; padding: 0; }
  .arrow { margin-left: 4px; font-size: 10px; opacity: 0.8; }
  td { padding: 4px 8px; border-bottom: 1px solid rgba(140, 160, 180, 0.12); white-space: nowrap; max-width: 260px; overflow: hidden; text-overflow: ellipsis; }
  tbody tr { cursor: pointer; }
  tbody tr:hover { background: rgba(120, 160, 200, 0.08); }
  .empty { padding: 20px; }
  .testtag { font-size: 11px; padding: 1px 6px; border-radius: 6px; border: 1px solid var(--warn, #b9770e); color: var(--warn, #b9770e); vertical-align: middle; }
  .check { display: inline-flex; align-items: center; gap: 5px; }
  .link { background: none; border: none; color: var(--accent); cursor: pointer; padding: 0; font-size: 13px; }
  @media (max-width: 700px) {
    .book { width: calc(100vw - 16px); height: calc(100vh - 16px); padding: 10px; }
  }
</style>
