<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
  const key = $derived(app.policy === 'live' ? 'banner.live' : app.policy === 'practice' ? 'banner.practice' : 'banner.listen');
  /** The real app still runs on the simulator: no radio has been chosen yet. */
  const noRadio = $derived(!app.demo && !!app.settings && app.settings.radio.kind === 'simulator');
</script>

<div class="banner {app.policy}" role="status">{t(key)}</div>
{#if noRadio}
  <div class="banner nora" role="status">
    {t('banner.noradio')}
    <button class="btn" onclick={() => (app.settingsOpen = true)}>{t('banner.noradio.btn')}</button>
  </div>
{/if}

<style>
  .banner { padding: 6px 16px; font-size: 14px; font-weight: 600; }
  .live { background: color-mix(in srgb, var(--live) 25%, var(--bg)); color: var(--text); border-bottom: 2px solid var(--live); }
  .listen_only { background: color-mix(in srgb, var(--listen) 18%, var(--bg)); border-bottom: 2px solid var(--listen); }
  .nora { background: color-mix(in srgb, var(--warn) 22%, var(--bg)); border-bottom: 2px solid var(--warn); display: flex; align-items: center; gap: 12px; }
  .practice { background: color-mix(in srgb, #8b6cf0 22%, var(--bg)); border-bottom: 2px solid #8b6cf0; }
</style>
