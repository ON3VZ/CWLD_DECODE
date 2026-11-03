<script lang="ts">
  import { tick } from 'svelte';
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
  import { splitCalls } from '../lib/callsign';

  let box: HTMLDivElement;
  $effect(() => {
    app.messages.length;
    tick().then(() => box && (box.scrollTop = box.scrollHeight));
  });
</script>

<div class="conv" bind:this={box}>
  {#each app.messages as m (m.id)}
    <div class="row {m.from}">
      <div class="bubble">
        <div class="meta muted">
          {m.from === 'rx' ? (m.call ?? '') : m.from === 'tx' ? t('conversation.you') : m.from === 'paddle' ? t('conversation.paddle') : t('conversation.practice')} · {m.time}
        </div>
        <div class="txt mono">
          {#each splitCalls(m.text) as p}{#if p.call}<button class="call" onclick={() => app.selectCall(p.text)}>{p.text}</button>{:else}{p.text}{/if}{/each}
        </div>
      </div>
    </div>
  {/each}
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
  .meta { font-size: 12px; margin-bottom: 2px; }
  .txt { font-size: 17px; word-break: break-word; }
  .call { background: none; border: none; padding: 0; color: var(--call); font: inherit; font-weight: 700; text-decoration: underline dotted; }
  .call:hover { text-decoration: underline; }
</style>
