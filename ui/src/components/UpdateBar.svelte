<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
</script>

{#if app.updateState === 'available' && app.updateInfo}
  <div class="update" role="status">
    <span>{t('update.available', { version: app.updateInfo.version })}</span>
    <button class="btn primary" type="button" onclick={() => app.installUpdate()}>{t('update.install')}</button>
    <button class="btn" type="button" onclick={() => (app.settingsOpen = true)}>{t('update.notes')}</button>
  </div>
{:else if app.updateState === 'downloading'}
  <div class="update" role="status">
    <span>{t('update.downloading')} {Math.round(app.updateProgress * 100)}%</span>
    <progress max="1" value={app.updateProgress}></progress>
  </div>
{/if}

<style>
  .update { display: flex; gap: 12px; align-items: center; padding: 6px 16px; font-size: 14px; background: color-mix(in srgb, var(--accent, #3b82f6) 18%, var(--bg)); border-bottom: 2px solid var(--accent, #3b82f6); }
  progress { flex: 1; max-width: 240px; }
</style>
