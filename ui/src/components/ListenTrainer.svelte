<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';

  const e = $derived(app.listenEx);
  const r = $derived(e?.result ?? null);
  let input: HTMLInputElement | undefined = $state();
  $effect(() => {
    if (e && !e.result) input?.focus();
  });
  function key(ev: KeyboardEvent) {
    if (ev.key === 'Enter') {
      ev.preventDefault();
      if (e && !e.result) void app.listenSubmit();
      else app.listenNext();
    } else if (ev.key === 'Escape') {
      app.listenStop();
    }
  }
</script>

{#if e}
  <section class="trainer card" aria-label={t('koch.listen.title')}>
    <div class="head">
      <h2>{t('koch.listen.title')}</h2>
      <button class="link" onclick={() => app.listenStop()}>{t('koch.listen.stop')}</button>
    </div>
    <p class="muted small">{t('koch.listen.how')}</p>
    <div class="row">
      <button class="btn" onclick={() => app.listenPlay()} disabled={!!r}>{e.playing ? t('koch.listen.playing') : t('koch.listen.again')}</button>
      <small class="muted">{t('koch.listen.pc')}</small>
    </div>
    {#if r}
      <div class="target mono" aria-label={t('paddle.target')}>
        {#each r.marks as m}<span class="c" class:bad={!m.ok} title={!m.ok ? t('koch.listen.typed', { got: m.got || '-' }) : ''}>{m.ch}{#if !m.ok}<small class="got">{m.got || '-'}</small>{/if}</span>{/each}
      </div>
      <p class="big">{Math.round(r.accuracy * 100)}% <span class="muted small">{t('koch.listen.right')}</span></p>
      <div class="row">
        <button class="btn primary" onclick={() => app.listenNext()}>{t('paddle.next')}</button>
      </div>
    {:else}
      <input
        bind:this={input}
        class="mono type"
        type="text"
        autocomplete="off"
        autocapitalize="characters"
        spellcheck="false"
        placeholder={t('koch.listen.placeholder')}
        aria-label={t('koch.listen.placeholder')}
        bind:value={e.typed}
        onkeydown={key}
      />
      <div class="row">
        <button class="btn primary" disabled={!e.typed.trim()} onclick={() => app.listenSubmit()}>{t('koch.listen.check')}</button>
      </div>
    {/if}
  </section>
{/if}

<style>
  .trainer { display: flex; flex-direction: column; gap: 10px; flex-shrink: 0; min-height: min-content; }
  .head { display: flex; justify-content: space-between; align-items: center; gap: 8px; }
  .row { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
  .type { font-size: 28px; letter-spacing: 0.12em; text-transform: uppercase; padding: 8px 10px; background: var(--panel2); border: 1px solid var(--line); border-radius: 10px; color: var(--text); }
  .target { font-size: 32px; letter-spacing: 0.12em; line-height: 1.6; word-break: break-word; padding: 8px 0; }
  .c { position: relative; display: inline-block; }
  .c.bad { color: var(--bad, #c0392b); text-decoration: underline wavy; }
  .c .got { position: absolute; left: 0; top: 100%; font-size: 12px; letter-spacing: 0; color: var(--muted); }
  .big { font-size: 26px; font-weight: 600; margin: 0; }
</style>
