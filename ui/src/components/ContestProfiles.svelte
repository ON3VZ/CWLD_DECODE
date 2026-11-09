<script lang="ts">
  import { untrack } from 'svelte';
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
  import { expand } from '../lib/template';
  import { allProfiles, BELGIAN_PREFIXES, cutNumber, exportText, isBuiltin, plain, RESERVED_VARS, validVarName } from '../lib/contest';
  import type { ContestProfile, ContestVar } from '../lib/types';

  const profiles = $derived(allProfiles(app.settings!.contest));

  let selectedId = $state(app.settings?.contest.active || 'uba-dx-cw');
  const selected = $derived(profiles.find((p) => p.id === selectedId) ?? profiles[0]);
  const builtin = $derived(isBuiltin(selected.id));

  // The editor works on a copy that is saved with the button.
  let ed = $state<ContestProfile>(untrack(() => structuredClone($state.snapshot(selected) as ContestProfile)));
  let callsText = $state('');
  let dirty = $state(false);
  let notice = $state<string | null>(null);

  function load(p: ContestProfile) {
    ed = structuredClone($state.snapshot(p) as ContestProfile);
    callsText = ed.calls.join(', ');
    dirty = false;
  }
  untrack(() => load(selected));

  let warned = false;
  let confirmDelete = $state(false);
  function choose(id: string) {
    if (dirty && !warned) {
      warned = true;
      notice = t('contest.unsaved');
      return;
    }
    warned = false;
    confirmDelete = false;
    selectedId = id;
    const p = profiles.find((x) => x.id === id);
    if (p) load(p);
    notice = null;
  }

  const RST_SAMPLE = $derived(app.settings?.default_rst ?? '5NN');
  function preview(template: string): string {
    const nr = String(app.settings?.contest_nr ?? 1).padStart(3, '0');
    const vars = { ...app.vars, CALL: 'PA3ABC', RST: RST_SAMPLE, NR: ed.cut_numbers ? cutNumber(nr) : nr };
    const r = expand(template.toUpperCase(), vars);
    return r.missing.length ? `${r.text}  (${t('composer.missing', { vars: r.missing.join(', ') })})` : r.text;
  }

  function touch() {
    dirty = true;
  }

  async function save() {
    const p: ContestProfile = $state.snapshot(ed) as ContestProfile;
    p.calls = callsText.split(',').map((c) => c.trim().toUpperCase()).filter(Boolean);
    p.sent = p.sent.filter((s) => s.template.trim()).map((s) => ({ label: s.label.trim() || t('contest.sent.default'), template: s.template.trim().toUpperCase() }));
    if (!p.name.trim() || !p.sent.length) {
      notice = t('contest.invalid');
      return;
    }
    if (p.recv) {
      p.recv.name = p.recv.name.trim().toUpperCase();
      if (!p.recv.name) p.recv = null;
    }
    await app.saveContestProfile(p);
    ed = structuredClone(p);
    dirty = false;
    notice = t('contest.saved');
  }

  async function copy() {
    const id = await app.copyContestProfile(selected.id, t('contest.copy.name', { name: plain(selected.name, t) }));
    if (id) {
      selectedId = id;
      const p = allProfiles(app.settings!.contest).find((x) => x.id === id);
      if (p) load(p);
    }
  }

  async function create() {
    const id = await app.newContestProfile(t('contest.new.name'));
    if (id) {
      selectedId = id;
      const p = allProfiles(app.settings!.contest).find((x) => x.id === id);
      if (p) load(p);
    }
  }

  async function remove() {
    if (!confirmDelete) {
      confirmDelete = true;
      return;
    }
    confirmDelete = false;
    await app.deleteContestProfile(selected.id);
    selectedId = 'uba-dx-cw';
    load(profiles.find((x) => x.id === 'uba-dx-cw')!);
  }

  function addSent() {
    ed.sent = [...ed.sent, { label: '', template: '{CALL} {RST}' }];
    touch();
  }
  function removeSent(i: number) {
    ed.sent = ed.sent.filter((_, k) => k !== i);
    touch();
  }
  function toggleRecv(on: boolean) {
    ed.recv = on ? { name: 'SEC', label: '', kind: 'text', choices: [], only_prefixes: [] } : null;
    touch();
  }

  // ---- Own variables ----
  let vars = $state<ContestVar[]>(structuredClone($state.snapshot(app.settings!.contest.vars) as ContestVar[]));
  let varProblem = $state<string | null>(null);
  async function saveVars() {
    const next = vars.map((v) => ({ name: v.name.trim().toUpperCase(), value: v.value.trim().toUpperCase() })).filter((v) => v.name || v.value);
    const bad = next.find((v) => !validVarName(v.name));
    if (bad) {
      varProblem = RESERVED_VARS.includes(bad.name) ? t('contest.vars.reserved', { name: bad.name }) : t('contest.vars.bad', { name: bad.name || '(empty)' });
      return;
    }
    if (new Set(next.map((v) => v.name)).size !== next.length) {
      varProblem = t('contest.vars.dup');
      return;
    }
    varProblem = null;
    await app.saveContestVars(next);
  }
  function addVar() {
    vars = [...vars, { name: '', value: '' }];
  }
  function removeVar(i: number) {
    vars = vars.filter((_, k) => k !== i);
    void saveVars();
  }

  // ---- Backup ----
  let replaceSame = $state(false);
  let importMsg = $state<string | null>(null);
  function download(text: string, name: string) {
    const url = URL.createObjectURL(new Blob([text], { type: 'application/json' }));
    const a = document.createElement('a');
    a.href = url;
    a.download = name;
    document.body.appendChild(a);
    a.click();
    a.remove();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  }
  function exportOne() {
    download(exportText(app.settings!.contest, selected.id), `cwld-profile-${selected.id}.json`);
  }
  function exportAll() {
    download(exportText(app.settings!.contest), 'cwld-contest-backup.json');
  }
  async function importFile(e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    input.value = '';
    if (!file) return;
    const text = await file.text();
    const r = await app.importContest(text, replaceSame);
    if (r) {
      importMsg = t('contest.import.done', { n: String(r.added), skipped: String(r.skipped) });
      vars = structuredClone($state.snapshot(app.settings!.contest.vars) as ContestVar[]);
    }
  }
</script>

<div class="cp">
  <div class="list" role="listbox" aria-label={t('contest.profiles.title')}>
    <h4>{t('contest.profiles.title')}</h4>
    <div class="opts">
    {#each profiles as p (p.id)}
      <button type="button" role="option" aria-selected={p.id === selected.id} class:on={p.id === selected.id} onclick={() => choose(p.id)}>
        {plain(p.name, t)}{#if isBuiltin(p.id)} <small class="muted">{t('contest.builtin.tag')}</small>{/if}
      </button>
    {/each}
    </div>
    <div class="row">
      <button class="btn" type="button" onclick={create}>{t('contest.new')}</button>
      <button class="btn" type="button" onclick={copy}>{t('contest.copy')}</button>
    </div>
  </div>

  <div class="editor">
    {#if builtin}<p class="muted">{t('contest.builtin.hint')}</p>{/if}
    <label>{t('contest.name')}
      <input value={plain(ed.name, t)} disabled={builtin} oninput={(e) => { ed.name = (e.currentTarget as HTMLInputElement).value; touch(); }} />
    </label>
    <label>{t('contest.calls')}
      <input class="mono" bind:value={callsText} disabled={builtin} oninput={touch} />
      <small class="muted">{t('contest.calls.help')}</small>
    </label>
    <label>{t('contest.note.label')}
      <textarea rows="2" value={plain(ed.note, t)} disabled={builtin} oninput={(e) => { ed.note = (e.currentTarget as HTMLTextAreaElement).value; touch(); }}></textarea>
    </label>

    <h4>{t('contest.sent.title')}</h4>
    <small class="muted">{t('contest.sent.help')}</small>
    {#each ed.sent as s, i (i)}
      <div class="sent">
        <input class="lab" placeholder={t('contest.sent.label')} aria-label={t('contest.sent.label')} value={plain(s.label, t)} disabled={builtin} oninput={(e) => { s.label = (e.currentTarget as HTMLInputElement).value; touch(); }} />
        <input class="mono tpl" placeholder="{'{CALL} {RST} {NR}'}" aria-label={t('contest.sent.template')} bind:value={s.template} disabled={builtin} oninput={touch} />
        {#if !builtin && ed.sent.length > 1}<button class="btn" type="button" onclick={() => removeSent(i)} aria-label={t('contest.sent.remove')}>×</button>{/if}
        <div class="prev mono" aria-live="polite"><span class="muted">{t('contest.sent.preview')}</span> {preview(s.template)}</div>
      </div>
    {/each}
    {#if !builtin}<button class="btn" type="button" onclick={addSent}>{t('contest.sent.add')}</button>{/if}

    <label class="check"><input type="checkbox" bind:checked={ed.cut_numbers} disabled={builtin} onchange={touch} /> {t('contest.cut')}
      <small class="muted">{t('contest.cut.help')}</small>
    </label>

    <h4>{t('contest.recv.title')}</h4>
    <small class="muted">{t('contest.recv.help')}</small>
    <label class="check"><input type="checkbox" checked={!!ed.recv} disabled={builtin} onchange={(e) => toggleRecv((e.currentTarget as HTMLInputElement).checked)} /> {t('contest.recv.on')}</label>
    {#if ed.recv}
      {@const r = ed.recv}
      <div class="recv">
        <label>{t('contest.recv.name')} <input class="mono short" bind:value={r.name} disabled={builtin} oninput={touch} maxlength="12" /></label>
        <label>{t('contest.recv.label')} <input bind:value={r.label} disabled={builtin} oninput={touch} /></label>
        <label>{t('contest.recv.kind')}
          <select bind:value={r.kind} disabled={builtin} onchange={touch}>
            <option value="text">{t('contest.recv.kind.text')}</option>
            <option value="number">{t('contest.recv.kind.number')}</option>
            <option value="choice">{t('contest.recv.kind.choice')}</option>
          </select>
        </label>
        {#if r.kind === 'choice'}
          <label>{t('contest.recv.choices')} <input class="mono" value={r.choices.join(', ')} disabled={builtin} oninput={(e) => { r.choices = (e.currentTarget as HTMLInputElement).value.split(',').map((x) => x.trim().toUpperCase()).filter(Boolean); touch(); }} /></label>
        {/if}
        <label>{t('contest.recv.prefixes')}
          <input class="mono" value={r.only_prefixes.join(', ')} disabled={builtin} oninput={(e) => { r.only_prefixes = (e.currentTarget as HTMLInputElement).value.split(',').map((x) => x.trim().toUpperCase()).filter(Boolean); touch(); }} />
          <small class="muted">{t('contest.recv.prefixes.help')}</small>
        </label>
        {#if !builtin}<button class="btn" type="button" onclick={() => { r.only_prefixes = [...BELGIAN_PREFIXES]; touch(); }}>{t('contest.recv.prefixes.be')}</button>{/if}
      </div>
    {/if}

    {#if !builtin}
      <div class="row">
        <button class="btn primary" type="button" onclick={save} disabled={!dirty}>{t('contest.save')}</button>
        <button class="btn danger" type="button" onclick={remove} onblur={() => (confirmDelete = false)}>{confirmDelete ? t('contest.delete.confirm', { name: ed.name }) : t('contest.delete')}</button>
      </div>
    {/if}
    {#if notice}<p class="muted" role="status">{notice}</p>{/if}
  </div>

  <div class="vars">
    <h4>{t('contest.vars.title')}</h4>
    <small class="muted">{t('contest.vars.help')}</small>
    {#each vars as v, i (i)}
      <div class="var">
        <input class="mono name" placeholder={t('contest.vars.name')} aria-label={t('contest.vars.name')} bind:value={v.name} maxlength="16" onchange={saveVars} />
        <input class="mono" placeholder={t('contest.vars.value')} aria-label={t('contest.vars.value')} bind:value={v.value} onchange={saveVars} />
        <button class="btn" type="button" onclick={() => removeVar(i)} aria-label={t('contest.vars.remove')}>×</button>
      </div>
    {/each}
    <button class="btn" type="button" onclick={addVar}>{t('contest.vars.add')}</button>
    {#if varProblem}<p class="bad" role="alert">{varProblem}</p>{/if}
  </div>

  <div class="backup">
    <h4>{t('contest.backup.title')}</h4>
    <small class="muted">{t('contest.backup.help')}</small>
    <div class="row">
      <button class="btn" type="button" onclick={exportOne}>{t('contest.export.profile')}</button>
      <button class="btn" type="button" onclick={exportAll}>{t('contest.export.all')}</button>
      <label class="btn file">{t('contest.import')}<input type="file" accept="application/json,.json" onchange={importFile} hidden /></label>
    </div>
    <label class="check"><input type="checkbox" bind:checked={replaceSame} /> {t('contest.import.replace')}</label>
    {#if importMsg}<p class="muted" role="status">{importMsg}</p>{/if}
  </div>
</div>

<style>
  .cp { display: flex; flex-direction: column; gap: 14px; }
  h4 { margin: 8px 0 2px; font-size: 15px; }
  .list { display: flex; flex-direction: column; gap: 4px; }
  .opts { display: flex; flex-direction: column; gap: 4px; max-height: 240px; overflow: auto; }
  .list small { margin-left: 8px; }
  .list button[role='option'] { text-align: left; background: var(--panel2); border: 1px solid var(--line); border-radius: 8px; padding: 6px 10px; color: inherit; cursor: pointer; }
  .list button.on { border-color: var(--accent); color: var(--accent); }
  .row { display: flex; gap: 8px; flex-wrap: wrap; align-items: center; }
  .editor, .vars, .backup { display: flex; flex-direction: column; gap: 8px; }
  label { display: flex; flex-direction: column; gap: 3px; font-size: 14px; }
  label.check { flex-direction: row; flex-wrap: wrap; align-items: center; gap: 6px; }
  input, select, textarea { background: var(--panel2); border: 1px solid var(--line); border-radius: 8px; padding: 6px 8px; color: var(--text); font-size: 14px; min-width: 0; }
  input:disabled, textarea:disabled, select:disabled { opacity: 0.7; }
  .short { width: 8em; }
  .sent { display: grid; grid-template-columns: 1fr 2fr auto; gap: 6px; align-items: center; }
  .prev { grid-column: 1 / -1; font-size: 13px; padding: 0 2px 6px; }
  .recv { display: flex; flex-direction: column; gap: 6px; padding-left: 12px; border-left: 2px solid var(--line); }
  .var { display: grid; grid-template-columns: 8em 1fr auto; gap: 6px; }
  .bad { color: #ff8a8a; margin: 0; }
  .file { cursor: pointer; }
</style>
