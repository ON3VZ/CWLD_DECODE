<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
  import { offsetLabel } from '../lib/split';

  /** Offsets one click away. On CW a pile-up nearly always listens one to a few kHz up. */
  const QUICK = ['UP 1', 'UP 2', 'UP 5', 'DWN 1'];
  /** Steps for moving the transmit frequency inside a pile-up, in Hz. */
  const NUDGE = [-500, -100, 100, 500];

  let text = $state('');
  let failed = $state(false);

  const s = $derived(app.split);
  const rx = $derived(app.status.frequency_hz);
  const khz = (hz: number) => (hz / 1000).toFixed(2);
  const label = $derived(s.on && s.tx_hz !== null ? (app.splitInfo?.label ?? offsetLabel(rx, s.tx_hz)) : '');
  const canSet = $derived(app.status.connected && !app.status.transmitting);

  async function set(value: string) {
    failed = !(await app.splitFromText(value));
    if (!failed) text = '';
  }

  function close() {
    app.splitOpen = false;
    app.splitEnded = null;
    failed = false;
  }

  function key(e: KeyboardEvent) {
    if (e.key === 'Enter' && text.trim()) {
      e.preventDefault();
      void set(text);
    }
    // Esc is not handled here on purpose: it stays the emergency stop everywhere.
  }
</script>

{#if s.on || app.splitOpen}
  <div class="split" class:on={s.on} role="status" aria-label={t('split.title')}>
    {#if s.on}
      <span class="tag">SPLIT</span>
      <span class="what">
        {t('split.on.listen')} <b class="mono">{khz(rx)}</b>
        <span class="arrow" aria-hidden="true">→</span>
        {t('split.on.transmit')} <b class="mono tx">{s.tx_hz !== null ? khz(s.tx_hz) : '?'}</b> kHz
        {#if label}<span class="lbl mono">{label}</span>{/if}
        {#if app.splitInfo?.call}<span class="muted">{app.splitInfo.call}</span>{/if}
      </span>
      {#if app.splitInfo?.toHz}
        <span class="muted">{t('split.on.range', { to: khz(app.splitInfo.toHz) })}</span>
      {/if}
      {#if app.splitInfo?.assumed}
        <span class="warn">{t('split.on.assumed')}</span>
      {/if}
      {#if s.tx_hz === null}
        <span class="warn">{t('split.on.unknown')}</span>
      {/if}
      <span class="grow"></span>
      {#if s.tx_hz !== null}
        <span class="nudge" title={t('split.nudge.help')}>
          <span class="muted">TX</span>
          {#each NUDGE as d (d)}
            <button class="btn sm mono" disabled={!canSet} onclick={() => app.splitNudge(d)}>{d > 0 ? '+' : '−'}{Math.abs(d) / 1000}</button>
          {/each}
        </span>
      {/if}
      <input class="mono in" bind:value={text} onkeydown={key} placeholder={t('split.input.change')} aria-label={t('split.input.label')} title={t('split.input.help')} disabled={!canSet} />
      <button class="btn off" onclick={() => app.splitOff()} title={t('split.off.help')}>{t('split.off')}</button>
    {:else}
      <span class="tag idle">SPLIT</span>
      {#if app.splitEnded}
        <span class="warn">{t(`split.ended.${app.splitEnded}`)}</span>
      {:else}
        <span>{t('split.manual.lead')}</span>
      {/if}
      <!-- svelte-ignore a11y_autofocus -->
      <input class="mono in" bind:value={text} onkeydown={key} placeholder={t('split.input.placeholder')} aria-label={t('split.input.label')} title={t('split.input.help')} disabled={!canSet} autofocus={!app.splitEnded} />
      <button class="btn primary" disabled={!canSet || !text.trim()} onclick={() => set(text)} title={t('split.set.help')}>{t('split.set')}</button>
      {#each QUICK as q (q)}
        <button class="btn sm mono" disabled={!canSet} onclick={() => set(q)} title={t('split.quick.help')}>{q}</button>
      {/each}
      <span class="muted hint">{t('split.manual.hint')}</span>
      <span class="grow"></span>
      <button class="btn sm" onclick={close} aria-label={t('split.close')} title={t('split.close')}>✕</button>
    {/if}
    {#if failed && app.error}<span class="err" role="alert">{app.error}</span>{/if}
  </div>
{/if}

<style>
  .split { display: flex; align-items: center; gap: 10px; flex-wrap: wrap; padding: 6px 16px; font-size: 14px; background: var(--panel); border-bottom: 1px solid var(--line); }
  .split.on { background: color-mix(in srgb, var(--warn) 20%, var(--bg)); border-bottom: 2px solid var(--warn); }
  .tag { font-weight: 800; letter-spacing: 0.08em; font-size: 12px; padding: 2px 8px; border-radius: 6px; background: var(--warn); color: #1a1205; }
  .tag.idle { background: var(--panel2); color: var(--muted); border: 1px solid var(--line); }
  .what { display: inline-flex; align-items: baseline; gap: 6px; flex-wrap: wrap; }
  .what b { font-size: 16px; }
  .what .tx { color: var(--warn); }
  .arrow { color: var(--muted); }
  .lbl { padding: 1px 7px; border-radius: 999px; border: 1px solid var(--warn); font-size: 12px; font-weight: 700; }
  .warn { color: var(--warn); font-weight: 600; }
  .grow { flex: 1; }
  .nudge { display: inline-flex; align-items: center; gap: 4px; }
  .in { width: 14em; background: var(--panel2); border: 1px solid var(--line); border-radius: 6px; padding: 3px 8px; color: var(--text); font-size: 13px; }
  .in:focus { border-color: var(--accent); outline: none; }
  .btn.sm { padding: 2px 8px; font-size: 12px; }
  .btn.off { background: var(--warn); border-color: var(--warn); color: #1a1205; font-weight: 700; }
  .hint { font-size: 12px; }
  .err { color: var(--live); font-weight: 600; flex-basis: 100%; }
</style>
