<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';

  const ex = $derived(app.paddleEx);
  const r = $derived(ex?.result ?? null);
  const fmt = (v: number | null, d = 1) => (v === null || !Number.isFinite(v) ? '-' : v.toFixed(d));
  /** Rows of the timing table: measured, ideal, and whether it is close enough. */
  const rows = $derived(
    r
      ? [
          { key: 'dash_dot', v: r.timing.dashDot, ideal: 3, ok: r.timing.dashDot === null || Math.abs(r.timing.dashDot - 3) <= 0.4 },
          { key: 'element_gap', v: r.timing.elementGap, ideal: 1, ok: r.timing.elementGap === null || Math.abs(r.timing.elementGap - 1) <= 0.35 },
          { key: 'char_gap', v: r.timing.charGap, ideal: 3, ok: r.timing.charGap === null || (r.timing.charGap >= 2.5 && r.timing.charGap <= 4.5) },
          { key: 'word_gap', v: r.timing.wordGap, ideal: 7, ok: r.timing.wordGap === null || (r.timing.wordGap >= 5.5 && r.timing.wordGap <= 10) },
        ]
      : [],
  );
</script>

{#if ex}
  <section class="trainer card" aria-label={t('paddle.title')}>
    <div class="head">
      <h2>{t('paddle.title')} · {t(`paddle.ex.${ex.exercise}`)}</h2>
      <button class="link" onclick={() => app.paddleStop()}>{t('paddle.stop')}</button>
    </div>

    {#if !app.audio?.running}
      <p class="warn small" role="alert">{t('paddle.no_audio')}</p>
    {:else if !ex.heard && !r}
      <p class="muted small">{t('paddle.waiting')}</p>
    {/if}
    {#if app.radioQuiet}<p class="muted small">{t('paddle.quiet')}</p>{/if}
    {#if app.radioControls?.af_level === 0}<p class="warn small" role="alert">{t('paddle.af_zero')}</p>{/if}

    <div class="target mono" aria-label={t('paddle.target')}>
      {#if r}
        {#each r.marks as m}<span class="ch" class:bad={!m.ok && m.ch !== ' '} class:space={m.ch === ' '} title={!m.ok && m.ch !== ' ' ? t('paddle.got', { got: m.got || '-' }) : ''}>{m.ch === ' ' ? ' ' : m.ch}{#if !m.ok && m.ch !== ' '}<small class="got">{m.got || '-'}</small>{/if}</span>{/each}
      {:else}
        {ex.target}
      {/if}
    </div>
    <div class="heard"><span class="muted small">{t('paddle.heard')}</span> <span class="mono">{ex.got.trim() || '…'}</span></div>

    <div class="row">
      {#if !r}
        <button class="btn primary" disabled={!ex.got.trim()} onclick={() => app.paddleFinish()} title={t('paddle.done.help')}>{t('paddle.done')}</button>
        <button class="btn" onclick={() => app.paddleRetry()}>{t('paddle.retry')}</button>
      {:else}
        <button class="btn primary" onclick={() => app.paddleNext()}>{t('paddle.next')}</button>
        <button class="btn" onclick={() => app.paddleRetry()}>{t('paddle.retry')}</button>
      {/if}
    </div>

    {#if r}
      <div class="nums">
        <div><span class="big">{r.score}</span><span class="muted small">{t('paddle.score')}</span></div>
        <div><span class="big">{Math.round(r.accuracy * 100)}%</span><span class="muted small">{t('paddle.accuracy')}</span></div>
        <div><span class="big">{fmt(r.timing.wpm, 0)}</span><span class="muted small">{t('paddle.wpm')}</span></div>
        <div><span class="big">{r.timing.score}</span><span class="muted small">{t('paddle.timing')}</span></div>
      </div>
      {#if r.timing.elements >= 4}
        <table class="small">
          <thead><tr><th>{t('paddle.t.what')}</th><th>{t('paddle.t.measured')}</th><th>{t('paddle.t.ideal')}</th></tr></thead>
          <tbody>
            {#each rows as row (row.key)}
              <tr class:bad={!row.ok}><td title={t(`paddle.t.${row.key}.help`)}>{t(`paddle.t.${row.key}`)}</td><td class="mono">{fmt(row.v)}</td><td class="mono">{row.ideal}</td></tr>
            {/each}
          </tbody>
        </table>
      {/if}
      <ul class="tips small">
        {#each r.timing.tips as tip}<li>{t(tip)}</li>{/each}
      </ul>
    {/if}
  </section>
{/if}

<style>
  .trainer { display: flex; flex-direction: column; gap: 10px; flex-shrink: 0; min-height: min-content; }
  .head { display: flex; justify-content: space-between; align-items: center; gap: 8px; }
  .target { font-size: 32px; letter-spacing: 0.12em; line-height: 1.6; word-break: break-word; padding: 8px 0; }
  .ch { position: relative; display: inline-block; }
  .ch.bad { color: var(--bad, #c0392b); text-decoration: underline wavy; }
  .ch .got { position: absolute; left: 0; top: 100%; font-size: 12px; letter-spacing: 0; color: var(--muted); }
  .heard { font-size: 15px; margin-top: 6px; }
  .row { display: flex; gap: 8px; flex-wrap: wrap; }
  .nums { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 8px; }
  .nums > div { display: flex; flex-direction: column; align-items: flex-start; }
  .big { font-size: 26px; font-weight: 600; }
  table { border-collapse: collapse; width: 100%; max-width: 460px; }
  th, td { text-align: left; padding: 3px 8px 3px 0; border-bottom: 1px solid var(--line); }
  tr.bad td { color: var(--bad, #c0392b); }
  .tips { margin: 0; padding-left: 18px; }
  .warn { color: var(--warn, #b9770e); }
  @media (max-width: 600px) {
    .target { font-size: 24px; }
    .nums { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  }
</style>
