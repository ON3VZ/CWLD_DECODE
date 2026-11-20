<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';

  // The station every text with {CALL} goes to: always the live QSO, also when an older tab is shown.
  const target = $derived(app.callTarget);
  const kind = $derived(!target.call ? 'none' : target.locked ? 'locked' : 'auto');
  const help = $derived(
    kind === 'none' ? t('callbadge.none') : kind === 'locked' ? `${t('callbadge.locked')}\n${t('callbadge.unlock')}` : `${t('callbadge.auto')}\n${t('callbadge.lock')}`,
  );
</script>

<div class="callbadge {kind}" class:flash={app.callFlash && kind === 'auto'} role="status" aria-live="polite">
  <span class="lbl">{t('callbadge.to')}</span>
  <button type="button" class="who mono" title={help} aria-label={`${t('callbadge.to')} ${target.call || t('callbadge.nobody')}. ${help}`} disabled={kind === 'none'} onclick={() => app.toggleLiveLock()}>
    {#if kind === 'locked'}<span class="lock" aria-hidden="true">🔒</span>{/if}
    {target.call || t('callbadge.nobody')}
  </button>
  <span class="hint muted">{kind === 'none' ? t('callbadge.none') : kind === 'locked' ? t('callbadge.locked') : t('callbadge.auto')}</span>
</div>

<style>
  .callbadge { display: flex; align-items: center; gap: 10px; margin-bottom: 6px; }
  .lbl { font-size: 13px; color: var(--muted); text-transform: uppercase; letter-spacing: 0.04em; }
  .who {
    font-size: 22px; font-weight: 700; padding: 2px 14px; border-radius: 8px; border: 2px solid var(--line);
    background: var(--panel); color: var(--text); cursor: pointer; min-width: 9ch; text-align: center;
  }
  .who:disabled { cursor: default; }
  .locked .who { border-color: var(--accent); color: var(--accent); }
  .auto .who { border-color: var(--call); color: var(--call); border-style: dashed; }
  .none .who { color: var(--muted); font-weight: 400; font-size: 16px; }
  .flash .who { animation: callflash 0.5s ease-in-out 4; }
  @keyframes callflash { 50% { background: var(--call); color: var(--bg); } }
  .lock { font-size: 16px; margin-right: 4px; }
  .hint { font-size: 13px; }
  @media (prefers-reduced-motion: reduce) { .flash .who { animation: none; outline: 3px solid var(--call); } }
</style>
