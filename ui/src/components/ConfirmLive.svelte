<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
  let okBtn: HTMLButtonElement;
  $effect(() => okBtn?.focus());
</script>

<div class="scrim" role="presentation">
  <div class="dlg" role="alertdialog" aria-modal="true" aria-labelledby="cl-title" aria-describedby="cl-body">
    <h2 id="cl-title">{t('mode.confirm.title')}</h2>
    <p id="cl-body">{t('mode.confirm.body')}</p>
    <div class="act">
      <button class="btn" onclick={() => (app.confirmLive = false)}>{t('mode.confirm.cancel')}</button>
      <button class="btn danger" bind:this={okBtn} onclick={() => app.confirmLiveSwitch()}>{t('mode.confirm.ok')}</button>
    </div>
  </div>
</div>

<style>
  .scrim { position: fixed; inset: 0; background: rgba(0, 0, 0, 0.6); display: grid; place-items: center; z-index: 10; }
  .dlg { background: var(--panel); border: 2px solid var(--live); border-radius: 14px; padding: 20px; max-width: 420px; }
  h2 { margin: 0 0 8px; }
  p { margin: 0 0 16px; }
  .act { display: flex; justify-content: flex-end; gap: 8px; }
</style>
