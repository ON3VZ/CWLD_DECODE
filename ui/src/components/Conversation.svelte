<script lang="ts">
  import { tick } from 'svelte';
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
  import { splitCalls } from '../lib/callsign';
  import { i18n } from '../lib/i18n.svelte';
  import { entryMeaning, lookup, usedIn } from '../lib/qcodes';
  import type { Message } from '../lib/demo';

  /** Below this confidence a character is drawn faded (see the design, 2.4). */
  const SURE = 0.9;

  let box: HTMLDivElement;
  /** The word the operator clicked and where the menu opens. */
  let pick = $state<{ word: string; x: number; y: number } | null>(null);
  const TARGETS = ['call', 'rst', 'nr', 'extra', 'name', 'qth'] as const;
  function choose(target: (typeof TARGETS)[number]) {
    if (pick) app.assignWord(pick.word, target);
    pick = null;
  }
  function openMenu(e: MouseEvent, word: string) {
    pick = { word, x: Math.min(e.clientX, window.innerWidth - 190), y: Math.min(e.clientY, window.innerHeight - 230) };
  }
  /** True while the operator is at the bottom: only then new text scrolls into view. Scrolled up to read, the view stays put. */
  let atBottom = $state(true);
  const NEAR = 48;
  function onScroll() {
    if (box) atBottom = box.scrollHeight - box.scrollTop - box.clientHeight <= NEAR;
  }
  function toBottom() {
    if (box) box.scrollTop = box.scrollHeight;
    atBottom = true;
  }
  $effect(() => {
    app.messages.length;
    app.tail;
    const last = app.messages[app.messages.length - 1];
    last?.text.length;
    tick().then(() => atBottom && toBottom());
  });
  // Another conversation is shown: start at its end.
  $effect(() => {
    app.shown.id;
    tick().then(toBottom);
  });

  type Run = { text: string; call: boolean; unsure: boolean };

  /** Splits a bubble into callsign links and runs of sure and unsure characters. */
  function runs(m: Message): Run[] {
    const out: Run[] = [];
    let pos = 0;
    for (const p of splitCalls(m.text)) {
      if (p.call || !m.conf) {
        out.push({ text: p.text, call: !!p.call, unsure: false });
      } else {
        let cur: Run | null = null;
        for (let i = 0; i < p.text.length; i++) {
          const unsure = (m.conf[pos + i] ?? 1) < SURE && p.text[i] !== ' ';
          if (!cur || cur.unsure !== unsure) {
            cur = { text: '', call: false, unsure };
            out.push(cur);
          }
          cur.text += p.text[i];
        }
      }
      pos += p.text.length;
    }
    return out;
  }

  /** Words with a known meaning get a tooltip: Q-codes, abbreviations, prosigns, own words. */
  type Piece = { text: string; tip?: string };
  function pieces(text: string): Piece[] {
    const own = app.settings?.user_words ?? [];
    return text.split(/(\s+)/).map((w) => {
      if (!w.trim()) return { text: w };
      const key = w.toUpperCase().replace(/[?.,;:!]+$/g, '');
      const o = own.find((x) => x.code.toUpperCase() === key);
      if (o) return { text: w, tip: `${key}: ${o.meaning}` };
      const e = /^\d+$/.test(key) && !['72', '73', '88'].includes(key) ? undefined : lookup(key);
      return e ? { text: w, tip: `${e.code}: ${entryMeaning(e, i18n.lang, w.endsWith('?'))}` } : { text: w };
    });
  }

  /** Everything said in this conversation, by both sides, for the list of codes used. */
  const used = $derived(usedIn(app.messages.map((m) => m.text).join(' '), app.settings?.user_words ?? []));
  const lastFrom = $derived(app.messages[app.messages.length - 1]?.from);
  const tail = $derived(app.viewingLive ? app.tail : null);
  const tailAlone = $derived(tail && tail.from !== lastFrom);
</script>

{#snippet tailChars()}
  {#if tail}<span class="tail">{#each tail.chars as c}<span title={c.alts.map(([a, p]) => `${a} ${Math.round(p * 100)}%`).join(', ')}>{c.best}</span>{/each}</span>{/if}
{/snippet}

<div class="convwrap">
{#if app.decodeHold}
  <div class="hold muted" role="status" title={t(`hold.${app.decodeHold.hold}.help`)}>{t(`hold.${app.decodeHold.hold}`, { mode: app.decodeHold.mode })}</div>
{/if}
<div class="conv" bind:this={box} onscroll={onScroll}>
  {#each app.messages as m, i (m.id)}
    <div class="row {m.from}">
      <div class="bubble">
        <div class="meta muted">
          {m.from === 'rx' ? (m.call ?? '') : m.from === 'tx' ? t('conversation.you') : m.from === 'paddle' ? t('conversation.paddle') : t('conversation.practice')} · {m.time}
        </div>
        <div class="txt mono">
          {#if m.from === 'rx' && app.hideRx}<span class="hidden" title={t('practice.hidden')}>{'· '.repeat(Math.min(24, Math.ceil(m.text.length / 2)))}</span>{:else}{#each runs(m) as r}{#if r.call}<button class="call" title={app.placeText(r.text) || undefined} onclick={() => app.selectCall(r.text)}>{r.text}</button>{:else if r.unsure}<span class="unsure" title={t('conversation.unsure')}>{r.text}</span>{:else}{#each pieces(r.text) as p}{#if !p.text.trim()}{p.text}{:else if m.from === 'rx'}<button class="w" class:known={!!p.tip} title={p.tip ?? t('conversation.word.help')} onclick={(e) => openMenu(e, p.text)}>{p.text}</button>{:else if p.tip}<abbr class="known" title={p.tip}>{p.text}</abbr>{:else}{p.text}{/if}{/each}{/if}{/each}{#if i === app.messages.length - 1 && tail && !tailAlone}{@render tailChars()}{/if}{/if}
        </div>
      </div>
    </div>
  {/each}
  {#if tailAlone && tail && !(tail.from === 'rx' && app.hideRx)}
    <div class="row {tail.from}">
      <div class="bubble pending"><div class="txt mono">{@render tailChars()}</div></div>
    </div>
  {/if}
</div>
{#if pick}
  <div class="scrimmenu" role="presentation" onclick={() => (pick = null)}></div>
  <div class="menu" role="menu" style="left:{pick.x}px; top:{pick.y}px">
    <div class="mh mono">{pick.word}</div>
    {#each TARGETS as tg}<button role="menuitem" onclick={() => choose(tg)}>{t(`conversation.as.${tg}`)}</button>{/each}
  </div>
{/if}
{#if used.length && !app.settings?.contest_mode}
  <details class="used">
    <summary>{t('conv.used', { n: String(used.length) })}</summary>
    <ul>
      {#each used as u (u.code)}
        <li><button class="code mono" onclick={() => app.insertText(u.code)} title={t('qlib.hint')}>{u.code}</button> <span>{u.own ?? (u.entry ? entryMeaning(u.entry, i18n.lang) : '')}</span></li>
      {/each}
    </ul>
  </details>
{/if}
</div>

<style>
  .hold { font-size: 13px; border: 1px dashed var(--line); border-radius: 10px; padding: 4px 10px; background: var(--panel); }
  .hidden { color: var(--muted); letter-spacing: 0.1em; }
  .convwrap { flex: 1; min-height: 190px; display: flex; flex-direction: column; gap: 6px; }
  .known { text-decoration: underline dotted; text-underline-offset: 3px; cursor: help; }
  .used { font-size: 13px; background: var(--panel); border: 1px solid var(--line); border-radius: 10px; padding: 4px 10px; }
  .used summary { cursor: pointer; color: var(--muted); }
  .used ul { list-style: none; margin: 6px 0 2px; padding: 0; display: grid; grid-template-columns: repeat(auto-fill, minmax(230px, 1fr)); gap: 4px 12px; max-height: 120px; overflow: auto; }
  .used li { display: flex; gap: 6px; align-items: baseline; }
  .used .code { background: var(--panel2); border: 1px solid var(--line); border-radius: 6px; padding: 0 6px; color: var(--accent); font-weight: 700; cursor: pointer; }
  .conv { flex: 1; overflow: auto; display: flex; flex-direction: column; gap: 10px; padding: 4px; min-height: 170px; }
  .row { display: flex; }
  .row.tx, .row.practice, .row.paddle { justify-content: flex-end; }
  .bubble { max-width: 78%; padding: 8px 12px; border-radius: 14px; background: var(--rx); }
  .tx .bubble { background: var(--tx); border-bottom-right-radius: 4px; }
  .paddle .bubble { background: var(--tx); border: 1.5px dashed var(--accent); border-bottom-right-radius: 4px; }
  .practice .bubble { background: var(--practice); border-bottom-right-radius: 4px; }
  .rx .bubble { border-bottom-left-radius: 4px; }
  .bubble.pending { opacity: 0.85; }
  .meta { font-size: 12px; margin-bottom: 2px; }
  .txt { font-size: 17px; word-break: break-word; }
  .call { background: none; border: none; padding: 0; color: var(--call); font: inherit; font-weight: 700; text-decoration: underline dotted; }
  .call:hover { text-decoration: underline; }
  .unsure { opacity: 0.45; }
  .w { background: none; border: none; padding: 0; margin: 0; color: inherit; font: inherit; cursor: pointer; border-radius: 3px; }
  .w:hover { background: color-mix(in srgb, var(--accent) 22%, transparent); }
  .scrimmenu { position: fixed; inset: 0; z-index: 40; }
  .menu { position: fixed; z-index: 41; background: var(--panel); border: 1px solid var(--line); border-radius: 10px; padding: 6px; display: flex; flex-direction: column; min-width: 170px; box-shadow: 0 6px 24px rgba(0,0,0,0.5); }
  .menu .mh { padding: 2px 8px 6px; color: var(--muted); }
  .menu button { background: none; border: none; text-align: left; padding: 6px 8px; color: var(--text); border-radius: 6px; cursor: pointer; }
  .menu button:hover { background: var(--panel2); }
  .tail { opacity: 0.4; font-style: italic; }
</style>
