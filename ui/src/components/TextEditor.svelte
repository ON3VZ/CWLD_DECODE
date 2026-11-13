<script lang="ts">
  import { untrack } from 'svelte';
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
  import { allProfiles, plain } from '../lib/contest';
  import type { RunMode } from '../lib/contestlog';
  import { durationSeconds } from '../lib/morse';
  import { CQ_MAX_LIMIT, CQ_PAUSE_MAX, CQ_PAUSE_MIN, checkOk, checkTemplate, chipSlots, cleanTemplate, contestSlots, knownVars, pileupSlots, previewParts, slotLabel, type Slot, type SlotGroup } from '../lib/texts';

  const focus = untrack(() => app.textEdit?.focus ?? null);
  const groupOf = (id: string | null): SlotGroup => (id?.startsWith('pileup.') ? 'pileup' : id?.startsWith('contest.') ? 'contest' : 'chip');

  let group = $state<SlotGroup>(groupOf(focus) === 'chip' && !focus && untrack(() => app.settings?.contest_mode) ? 'contest' : groupOf(focus));
  let profile = $state(untrack(() => app.settings?.contest.active ?? ''));
  let run = $state<RunMode>(untrack(() => app.runMode));
  let selected = $state<string | null>(focus);
  let tpl = $state('');
  let name = $state('');
  let loadedFor = $state<string | null>(null);
  let pendingSwitch = $state<(() => void) | null>(null);
  let area = $state<HTMLTextAreaElement | null>(null);
  let pause = $state(untrack(() => app.sendTexts.cq_pause_s));
  let max = $state(untrack(() => app.sendTexts.cq_max));
  let added = $state('');
  /** A question shown in the dialog itself (window.confirm is not used in CWLD). */
  let ask = $state<{ text: string; yes: () => Promise<void> } | null>(null);

  const texts = $derived(app.sendTexts);
  const contest = $derived(app.settings!.contest);
  const profiles = $derived(allProfiles(contest));
  const slots = $derived.by((): Slot[] => {
    if (group === 'chip') return chipSlots(texts);
    if (group === 'pileup') return pileupSlots(texts);
    const c = { ...contest, active: profile, sent_index: profile === contest.active ? contest.sent_index : 0 };
    return contestSlots(run, c, texts);
  });
  const slot = $derived(slots.find((s) => s.id === selected) ?? null);
  const known = $derived(knownVars(contest));
  const check = $derived(checkTemplate(tpl, known));
  const parts = $derived(previewParts(tpl, app.vars));
  const wpm = $derived(Math.max(5, app.status.cw_wpm || 20));
  const seconds = $derived.by(() => {
    const plainText = parts.map((p) => p.text).join('');
    try {
      return durationSeconds(plainText.replace(/[{}]/g, ''), wpm);
    } catch {
      return 0;
    }
  });
  const dirty = $derived(!!slot && (cleanTemplate(tpl) !== slot.template || name.trim() !== (slot.label || '')));

  // Load the chosen slot into the fields (once per choice, so typing is not overwritten).
  $effect(() => {
    if (slot && loadedFor !== slot.id) {
      tpl = slot.template;
      name = slot.label;
      loadedFor = slot.id;
    }
    if (!slot && slots.length && !selected) selected = slots[0].id;
  });

  /** Asks first when the current text has unsaved changes. */
  function guard(action: () => void) {
    if (dirty) pendingSwitch = action;
    else action();
  }
  function choose(id: string) {
    guard(() => {
      selected = id;
      loadedFor = null;
    });
  }
  function setGroup(g: SlotGroup) {
    guard(() => {
      group = g;
      selected = null;
      loadedFor = null;
    });
  }
  function setProfile(p: string) {
    guard(() => {
      profile = p;
      selected = null;
      loadedFor = null;
    });
  }
  function setRun(r: RunMode) {
    guard(() => {
      const key = slot?.fkey;
      run = r;
      loadedFor = null;
      selected = null;
      // Keep the same function key in the other mode.
      queueMicrotask(() => {
        const same = slots.find((s) => s.fkey === key);
        if (same) selected = same.id;
      });
    });
  }

  async function save() {
    if (!slot) return;
    await app.saveSlot(slot, tpl, name);
    loadedFor = null;
  }
  async function restore() {
    if (!slot) return;
    await app.restoreSlot(slot.id);
    loadedFor = null;
  }
  async function restoreAll() {
    const prefix = group === 'chip' ? 'chip.' : group === 'pileup' ? 'pileup.' : `contest.${profile || 'none'}.`;
    ask = {
      text: t('texts.restore.all.ask'),
      yes: async () => {
        await app.restoreTexts(prefix);
        loadedFor = null;
      },
    };
  }
  async function addChip() {
    guard(async () => {
      const id = await app.addOwnChip(t('texts.own.new'), 'TU 73');
      selected = id;
      loadedFor = null;
      added = id;
    });
  }
  async function removeChip() {
    if (!slot?.own) return;
    const id = slot.id;
    ask = {
      text: t('texts.own.delete.ask', { name: slotLabel(slot, t) }),
      yes: async () => {
        await app.removeOwnChip(id);
        selected = null;
        loadedFor = null;
      },
    };
  }

  function insertVar(v: string) {
    const token = `{${v}}`;
    const el = area;
    if (!el) {
      tpl = (tpl && !tpl.endsWith(' ') ? tpl + ' ' : tpl) + token;
      return;
    }
    const a = el.selectionStart ?? tpl.length;
    const b = el.selectionEnd ?? tpl.length;
    const before = tpl.slice(0, a);
    const sep = before && !before.endsWith(' ') ? ' ' : '';
    tpl = before + sep + token + tpl.slice(b);
    const pos = (before + sep + token).length;
    queueMicrotask(() => {
      el.focus();
      el.setSelectionRange(pos, pos);
    });
  }

  async function saveRepeat() {
    await app.setCqRepeatSettings(pause, max);
    pause = app.sendTexts.cq_pause_s;
    max = app.sendTexts.cq_max;
  }

  function close() {
    guard(() => app.closeTextEditor());
  }
</script>

<svelte:window onkeydown={(e) => { if (e.key === 'Escape') { e.preventDefault(); if (ask) ask = null; else if (pendingSwitch) pendingSwitch = null; else close(); } }} />

<div class="scrim" role="presentation">
  <div class="dlg" role="dialog" aria-modal="true" aria-labelledby="te-title">
    <header>
      <h2 id="te-title">{t('texts.title')}</h2>
      <button class="x" onclick={close} aria-label={t('texts.close')}>×</button>
    </header>
    <p class="muted intro">{t('texts.intro')}</p>

    <div class="tabs" role="tablist">
      <button role="tab" aria-selected={group === 'chip'} class:on={group === 'chip'} onclick={() => setGroup('chip')} title={t('texts.tab.help')}>{t('texts.group.chip')}</button>
      <button role="tab" aria-selected={group === 'pileup'} class:on={group === 'pileup'} onclick={() => setGroup('pileup')} title={t('texts.tab.help')}>{t('texts.group.pileup')}</button>
      <button role="tab" aria-selected={group === 'contest'} class:on={group === 'contest'} onclick={() => setGroup('contest')} title={t('texts.tab.help')}>{t('texts.group.contest')}</button>
    </div>

    {#if group === 'contest'}
      <div class="row">
        <label class="inl">{t('contest.profile')}
          <select value={profile} onchange={(e) => setProfile((e.currentTarget as HTMLSelectElement).value)}>
            <option value="">{t('contest.profile.none')}</option>
            {#each profiles as p (p.id)}<option value={p.id}>{plain(p.name, t)}</option>{/each}
          </select>
        </label>
        <div class="seg" role="group" aria-label={t('contest.runsp')}>
          <button class:on={run === 'run'} aria-pressed={run === 'run'} onclick={() => setRun('run')}>Run</button>
          <button class:on={run === 'sp'} aria-pressed={run === 'sp'} onclick={() => setRun('sp')}>S&amp;P</button>
        </div>
      </div>
      <p class="muted small">{t('texts.contest.note')}</p>
    {/if}

    {#if ask}
      <div class="pending" role="alert">
        <span>{ask.text}</span>
        <button class="btn danger" onclick={async () => { const a = ask; ask = null; await a?.yes(); }}>{t('texts.yes')}</button>
        <button class="btn" onclick={() => (ask = null)}>{t('texts.no')}</button>
      </div>
    {/if}

    <div class="cols">
      <div class="list" role="listbox" aria-label={t('texts.list')}>
        {#each slots as s (s.id)}
          <button role="option" aria-selected={s.id === selected} class="item" class:sel={s.id === selected} onclick={() => choose(s.id)}>
            <span class="top">
              {#if s.fkey}<kbd>{s.fkey}</kbd>{/if}
              <strong>{slotLabel(s, t)}</strong>
              {#if s.own}<span class="tag">{t('texts.own')}</span>{:else if s.changed}<span class="tag ch" title={t('texts.changed.help')}>{t('texts.changed')}</span>{/if}
            </span>
            <span class="mono tpl">{s.template}</span>
          </button>
        {/each}
        <div class="listact">
          {#if group === 'chip'}<button class="btn" onclick={addChip} title={t('texts.own.add.help')}>+ {t('texts.own.add')}</button>{/if}
          <button class="link" onclick={restoreAll} title={t('texts.restore.all.help')}>{t('texts.restore.all')}</button>
        </div>
      </div>

      <div class="edit">
        {#if slot}
          {#if pendingSwitch}
            <div class="pending" role="alert">
              <span>{t('texts.unsaved')}</span>
              <button class="btn primary" onclick={async () => { const go = pendingSwitch; pendingSwitch = null; await save(); go?.(); }}>{t('texts.save')}</button>
              <button class="btn" onclick={() => { const go = pendingSwitch; pendingSwitch = null; loadedFor = null; tpl = slot!.template; name = slot!.label; go?.(); }}>{t('texts.discard')}</button>
            </div>
          {/if}
          <label>{t('texts.name')}
            <input bind:value={name} maxlength="24" placeholder={slot.labelKey ? t(slot.labelKey) : ''} />
          </label>
          <label><span>{t('texts.text')}{#if slot.fkey}&nbsp;<kbd>{slot.fkey}</kbd>{/if}</span>
            <textarea
              class="mono"
              rows="3"
              bind:this={area}
              bind:value={tpl}
              oninput={() => (tpl = tpl.toUpperCase())}
              autocomplete="off"
              spellcheck="false"></textarea>
          </label>
          <div class="vars" role="group" aria-label={t('texts.vars')}>
            <span class="muted small">{t('texts.vars')}:</span>
            {#each known as v (v)}
              <button class="var mono" onclick={() => insertVar(v)} title={t(`texts.var.${v}`) === `texts.var.${v}` ? (app.vars[v] || v) : t(`texts.var.${v}`)}>{'{'}{v}{'}'}</button>
            {/each}
          </div>
          <div class="preview" aria-live="polite">
            <span class="muted small">{t('texts.preview')}</span>
            <div class="mono pv">
              {#each parts as p, i (i)}<span class={p.kind}>{p.text}</span>{/each}
            </div>
            <span class="muted small">{t('texts.duration', { s: seconds.toFixed(1), wpm: String(wpm) })}</span>
          </div>
          {#if !checkOk(check)}
            <ul class="warns">
              {#if check.unknownVars.length}<li>{t('texts.warn.vars', { vars: check.unknownVars.map((v) => `{${v}}`).join(', ') })}</li>{/if}
              {#if check.badChars.length}<li>{t('texts.warn.chars', { chars: check.badChars.join(' ') })}</li>{/if}
              {#if check.badProsigns.length}<li>{t('texts.warn.prosigns', { list: check.badProsigns.map((p) => `<${p}>`).join(', ') })}</li>{/if}
              {#if check.braces}<li>{t('texts.warn.braces')}</li>{/if}
            </ul>
          {/if}
          {#if !slot.own && slot.def}
            <p class="muted small">{t('texts.default')}: <span class="mono">{slot.def}</span></p>
          {/if}
          {#if added === slot.id}<p class="muted small">{t('texts.own.added')}</p>{/if}
          <div class="act">
            {#if group === 'chip'}
              <button class="btn" onclick={() => app.moveChip(slot!.id, -1)} title={t('texts.move.up')} aria-label={t('texts.move.up')}>↑</button>
              <button class="btn" onclick={() => app.moveChip(slot!.id, 1)} title={t('texts.move.down')} aria-label={t('texts.move.down')}>↓</button>
            {/if}
            {#if slot.own}
              <button class="btn danger" onclick={removeChip}>{t('texts.own.delete')}</button>
            {:else}
              <button class="btn" onclick={restore} disabled={!slot.changed && !dirty} title={t('texts.restore.help')}>{t('texts.restore')}</button>
            {/if}
            <span class="grow"></span>
            <button class="btn" onclick={() => { tpl = slot!.template; name = slot!.label; }} disabled={!dirty} title={t('texts.undo.help')}>{t('texts.undo')}</button>
            <button class="btn primary" onclick={save} disabled={!dirty || !cleanTemplate(tpl)} title={t('texts.save.help')}>{t('texts.save')}</button>
          </div>
        {:else}
          <p class="muted">{t('texts.pick')}</p>
        {/if}
      </div>
    </div>

    <details class="more">
      <summary>{t('texts.repeat.title')}</summary>
      <p class="muted small">{t('texts.repeat.help')}</p>
      <div class="row">
        <label class="inl">{t('texts.repeat.pause')}
          <input type="number" min={CQ_PAUSE_MIN} max={CQ_PAUSE_MAX} bind:value={pause} onchange={saveRepeat} class="num" /> s
        </label>
        <label class="inl">{t('texts.repeat.max')}
          <input type="number" min="1" max={CQ_MAX_LIMIT} bind:value={max} onchange={saveRepeat} class="num" />
        </label>
      </div>
      <label class="check">
        <input type="checkbox" checked={texts.paddle_wait} onchange={(e) => app.setPaddleWait((e.currentTarget as HTMLInputElement).checked)} />
        {t('texts.paddlewait')}
      </label>
      <p class="muted small">{t('texts.paddlewait.help')}</p>
    </details>
  </div>
</div>

<style>
  .scrim { position: fixed; inset: 0; background: rgba(0, 0, 0, 0.6); display: grid; place-items: center; z-index: 30; }
  .dlg { background: var(--panel); border: 1px solid var(--line); border-radius: 14px; padding: 16px 18px; width: min(900px, 96vw); max-height: 94vh; overflow: auto; display: flex; flex-direction: column; gap: 10px; }
  header { display: flex; align-items: center; justify-content: space-between; }
  h2 { margin: 0; font-size: 18px; }
  .x { background: none; border: none; font-size: 22px; color: var(--text); line-height: 1; }
  .intro { margin: 0; }
  .muted { color: var(--muted); }
  .small { font-size: 12px; margin: 0; }
  .tabs { display: flex; gap: 4px; border-bottom: 1px solid var(--line); }
  .tabs button { background: none; border: none; border-bottom: 2px solid transparent; color: var(--muted); padding: 6px 12px; font-size: 14px; }
  .tabs button.on { color: var(--text); border-bottom-color: var(--accent); font-weight: 600; }
  .row { display: flex; flex-wrap: wrap; gap: 12px; align-items: center; }
  .inl { display: flex; align-items: center; gap: 6px; font-size: 13px; color: var(--muted); }
  select, input, textarea { background: var(--panel2); border: 1px solid var(--line); border-radius: 8px; padding: 6px 8px; color: var(--text); font-size: 15px; }
  .num { width: 4.5em; }
  .seg { display: inline-flex; border: 1px solid var(--line); border-radius: 8px; overflow: hidden; }
  .seg button { background: var(--panel2); border: none; color: var(--text); padding: 4px 12px; }
  .seg button.on { background: var(--accent); color: var(--bg); font-weight: 700; }
  .cols { display: grid; grid-template-columns: minmax(220px, 1fr) 2fr; gap: 14px; min-height: 300px; }
  @media (max-width: 700px) { .cols { grid-template-columns: 1fr; } }
  .list { display: flex; flex-direction: column; gap: 4px; overflow: auto; max-height: 52vh; padding-right: 4px; }
  .item { display: flex; flex-direction: column; align-items: flex-start; gap: 2px; text-align: left; background: var(--panel2); border: 1px solid var(--line); border-radius: 8px; padding: 6px 8px; color: var(--text); }
  .item.sel { border-color: var(--accent); box-shadow: 0 0 0 1px var(--accent) inset; }
  .top { display: flex; gap: 6px; align-items: center; }
  .tpl { font-size: 12px; color: var(--muted); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 100%; }
  .tag { font-size: 10px; border: 1px solid var(--line); border-radius: 999px; padding: 0 6px; color: var(--muted); }
  .tag.ch { border-color: var(--accent); color: var(--accent); }
  kbd { font: inherit; font-size: 11px; color: var(--muted); border: 1px solid var(--line); border-radius: 4px; padding: 0 4px; }
  .listact { display: flex; gap: 10px; align-items: center; margin-top: 6px; flex-wrap: wrap; }
  .link { background: none; border: none; color: var(--accent); text-decoration: underline; font-size: 12px; padding: 0; }
  .edit { display: flex; flex-direction: column; gap: 8px; }
  .edit label { display: flex; flex-direction: column; gap: 3px; font-size: 12px; color: var(--muted); }
  textarea { font-size: 17px; resize: vertical; text-transform: uppercase; }
  .vars { display: flex; flex-wrap: wrap; gap: 4px; align-items: center; }
  .var { background: var(--panel2); border: 1px solid var(--line); border-radius: 999px; padding: 1px 8px; font-size: 12px; color: var(--text); }
  .preview { display: flex; flex-direction: column; gap: 3px; background: var(--bg); border: 1px dashed var(--line); border-radius: 8px; padding: 6px 10px; }
  .pv { font-size: 16px; white-space: pre-wrap; word-break: break-word; min-height: 1.4em; }
  .pv .var { background: none; border: none; padding: 0; font-size: inherit; color: var(--accent); font-weight: 600; border-radius: 0; }
  .pv .missing { color: var(--warn); text-decoration: underline dotted; }
  .warns { margin: 0; padding-left: 18px; color: var(--warn); font-size: 13px; }
  .pending { display: flex; gap: 8px; align-items: center; border: 1px solid var(--warn); border-radius: 8px; padding: 6px 10px; font-size: 13px; }
  .pending span { flex: 1; }
  .act { display: flex; gap: 6px; align-items: center; flex-wrap: wrap; }
  .grow { flex: 1; }
  .more { border-top: 1px solid var(--line); padding-top: 8px; display: flex; flex-direction: column; gap: 6px; }
  .more summary { cursor: pointer; font-size: 14px; }
  .check { display: flex; gap: 8px; align-items: center; font-size: 14px; }
</style>
