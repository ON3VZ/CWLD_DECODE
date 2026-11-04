<script lang="ts">
  import { tick } from 'svelte';
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
  import { splitCalls } from '../lib/callsign';
  import type { Message } from '../lib/demo';

  /** Below this confidence a character is drawn faded (see the design, 2.4). */
  const SURE = 0.9;

  let box: HTMLDivElement;
  $effect(() => {
    app.messages.length;
    app.tail;
    const last = app.messages[app.messages.length - 1];
    last?.text.length;
    tick().then(() => box && (box.scrollTop = box.scrollHeight));
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

  const lastFrom = $derived(app.messages[app.messages.length - 1]?.from);
  const tailAlone = $derived(app.tail && app.tail.from !== lastFrom);
</script>

{#snippet tailChars()}
  {#if app.tail}<span class="tail">{#each app.tail.chars as c}<span title={c.alts.map(([a, p]) => `${a} ${Math.round(p * 100)}%`).join(', ')}>{c.best}</span>{/each}</span>{/if}
{/snippet}

<div class="conv" bind:this={box}>
  {#each app.messages as m, i (m.id)}
    <div class="row {m.from}">
      <div class="bubble">
        <div class="meta muted">
          {m.from === 'rx' ? (m.call ?? '') : m.from === 'tx' ? t('conversation.you') : m.from === 'paddle' ? t('conversation.paddle') : t('conversation.practice')} · {m.time}
        </div>
        <div class="txt mono">
          {#each runs(m) as r}{#if r.call}<button class="call" onclick={() => app.selectCall(r.text)}>{r.text}</button>{:else if r.unsure}<span class="unsure" title={t('conversation.unsure')}>{r.text}</span>{:else}{r.text}{/if}{/each}{#if i === app.messages.length - 1 && app.tail && !tailAlone}{@render tailChars()}{/if}
        </div>
      </div>
    </div>
  {/each}
  {#if tailAlone && app.tail}
    <div class="row {app.tail.from}">
      <div class="bubble pending"><div class="txt mono">{@render tailChars()}</div></div>
    </div>
  {/if}
</div>

<style>
  .conv { flex: 1; overflow: auto; display: flex; flex-direction: column; gap: 10px; padding: 4px; min-height: 120px; }
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
  .tail { opacity: 0.4; font-style: italic; }
</style>
