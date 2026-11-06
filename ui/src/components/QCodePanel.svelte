<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { i18n, t } from '../lib/i18n.svelte';
  import { Q_ENTRIES, entryMeaning, searchEntries, type QKind } from '../lib/qcodes';

  let query = $state('');
  let tab = $state<'all' | QKind | 'own'>('all');
  let search: HTMLInputElement;

  const tabs: Array<'all' | QKind | 'own'> = ['all', 'q', 'abbr', 'prosign', 'own'];
  let newCode = $state('');
  let newMeaning = $state('');
  async function addOwn(e: Event) {
    e.preventDefault();
    await app.addUserWord(newCode, newMeaning);
    newCode = '';
    newMeaning = '';
  }
  const rows = $derived(searchEntries(Q_ENTRIES.filter((e) => tab === 'all' || e.kind === tab), query, i18n.lang));
  const ownRows = $derived((app.settings?.user_words ?? []).filter((w) => !query.trim() || w.code.includes(query.trim().toUpperCase()) || w.meaning.toUpperCase().includes(query.trim().toUpperCase())));

  $effect(() => {
    search?.focus();
  });
</script>

<div class="scrim" role="presentation" onclick={() => (app.qOpen = false)}></div>
<aside class="panel" aria-label={t('qlib.title')}>
  <header>
    <h2>{t('qlib.title')}</h2>
    <button class="btn" onclick={() => (app.qOpen = false)} aria-label={t('qlib.close')}>×</button>
  </header>
  <input bind:this={search} class="search" type="search" placeholder={t('qlib.search')} aria-label={t('qlib.search')} bind:value={query} />
  <div class="tabs" role="tablist">
    {#each tabs as k}
      <button role="tab" aria-selected={tab === k} class:on={tab === k} onclick={() => (tab = k)}>{t(`qlib.tab.${k}`)}</button>
    {/each}
  </div>
  <p class="muted hint">{t('qlib.hint')}</p>
  {#if tab === 'own' || tab === 'all'}
    <form class="own" onsubmit={addOwn}>
      <input class="mono" placeholder={t('qlib.own.code')} aria-label={t('qlib.own.code')} bind:value={newCode} maxlength="12" />
      <input placeholder={t('qlib.own.meaning')} aria-label={t('qlib.own.meaning')} bind:value={newMeaning} />
      <button class="btn" type="submit" disabled={!newCode.trim() || !newMeaning.trim()}>{t('qlib.own.add')}</button>
    </form>
  {/if}
  <ul>
    {#if tab === 'own' || tab === 'all'}
      {#each ownRows as w (w.code)}
        <li>
          <button class="code mono" onclick={() => app.insertText(w.code)}>{w.code}</button>
          <div class="txt"><span>{w.meaning}</span><span class="muted ask">{t('qlib.own.tag')}</span></div>
          <button class="link" onclick={() => app.removeUserWord(w.code)} aria-label={t('qlib.own.remove')} title={t('qlib.own.remove')}>×</button>
        </li>
      {/each}
    {/if}
    {#each tab === 'own' ? [] : rows as e (e.kind + e.code)}
      <li>
        <button class="code mono" disabled={e.insertable === false} title={e.insertable === false ? t('qlib.noinsert') : ''} onclick={() => app.insertText(e.code)}>{e.code}</button>
        <div class="txt">
          <span>{entryMeaning(e, i18n.lang)}</span>
          {#if e.askEn}
            <span class="muted ask">{t('qlib.ask')} <button class="link mono" onclick={() => app.insertText(e.code + '?')} title={t('qlib.ask.btn')}>{e.code}?</button> {entryMeaning(e, i18n.lang, true)}</span>
          {/if}
          {#if e.pattern}<span class="muted mono pat">{e.pattern}</span>{/if}
        </div>
      </li>
    {:else}
      {#if !ownRows.length}<li class="muted">{t('qlib.none')}</li>{/if}
    {/each}
  </ul>
</aside>

<style>
  .scrim { position: fixed; inset: 0; background: rgba(0, 0, 0, 0.25); z-index: 20; }
  .panel { position: fixed; top: 0; right: 0; bottom: 0; width: min(420px, 100vw); background: var(--panel); border-left: 1px solid var(--line); z-index: 21; display: flex; flex-direction: column; gap: 8px; padding: 12px; box-shadow: -8px 0 24px rgba(0, 0, 0, 0.3); animation: slide 0.15s ease-out; }
  @keyframes slide { from { transform: translateX(30px); opacity: 0; } to { transform: none; opacity: 1; } }
  @media (prefers-reduced-motion: reduce) { .panel { animation: none; } }
  header { display: flex; justify-content: space-between; align-items: center; }
  h2 { margin: 0; font-size: 18px; }
  .search { background: var(--panel2); border: 1px solid var(--line); border-radius: 8px; padding: 8px 10px; font-size: 16px; }
  .tabs { display: flex; gap: 4px; flex-wrap: wrap; }
  .tabs button { background: var(--panel2); border: 1px solid var(--line); border-radius: 999px; padding: 4px 10px; cursor: pointer; color: inherit; }
  .tabs button.on { border-color: var(--accent); color: var(--accent); }
  .own { display: flex; gap: 6px; }
  .own input { background: var(--panel2); border: 1px solid var(--line); border-radius: 8px; padding: 6px 8px; font-size: 14px; color: var(--text); min-width: 0; }
  .own input:first-child { width: 6em; }
  .own input:nth-child(2) { flex: 1; }
  .hint { margin: 0; font-size: 13px; }
  ul { list-style: none; margin: 0; padding: 0; overflow: auto; flex: 1; display: flex; flex-direction: column; gap: 6px; }
  li { display: flex; gap: 10px; align-items: flex-start; padding: 6px 0; border-bottom: 1px solid var(--line); }
  .code { min-width: 62px; background: var(--panel2); border: 1px solid var(--line); border-radius: 8px; padding: 4px 8px; font-weight: 700; cursor: pointer; color: var(--accent); }
  .code:disabled { opacity: 0.5; cursor: not-allowed; }
  .txt { display: flex; flex-direction: column; gap: 2px; font-size: 14px; }
  .ask { font-size: 13px; }
  .pat { font-size: 12px; letter-spacing: 0.15em; }
  .link { background: none; border: none; color: var(--accent); cursor: pointer; padding: 0; }
</style>
