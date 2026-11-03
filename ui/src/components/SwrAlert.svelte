<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
  let btn = $state<HTMLButtonElement>();
  $effect(() => btn?.focus());
</script>

{#if app.swrAlert}
  <div class="alert" role="alertdialog" aria-live="assertive" aria-labelledby="swr-t" aria-describedby="swr-b">
    <div class="head">
      <span class="sign" aria-hidden="true">!</span>
      <strong id="swr-t">{t('swr.alert.title')}</strong>
      <span class="val mono">{app.swrAlert.swr.toFixed(1)}</span>
    </div>
    <p id="swr-b">
      {t('swr.alert.body', { swr: app.swrAlert.swr.toFixed(1), limit: app.swrAlert.limit.toFixed(1) })}
      {#if app.swrAlert.qrx_sent}{t('swr.alert.qrx')}{/if}
    </p>
    <p class="small">{t('swr.paddle')}</p>
    <button class="btn" bind:this={btn} onclick={() => app.ackSwr()}>{t('swr.alert.ack')}</button>
  </div>
{/if}

<style>
  .alert { background: #b3141a; color: #fff; padding: 12px 16px; border-bottom: 3px solid #ff6b6b; display: flex; flex-direction: column; gap: 6px; }
  .head { display: flex; align-items: center; gap: 12px; font-size: 20px; letter-spacing: 0.06em; }
  .sign { display: grid; place-items: center; width: 30px; height: 30px; border-radius: 50%; background: #fff; color: #b3141a; font-weight: 900; }
  .val { margin-left: auto; font-size: 24px; font-weight: 700; }
  p { margin: 0; }
  .small { font-size: 13px; opacity: 0.9; }
  .btn { align-self: flex-start; background: #fff; color: #7a0c10; border-color: #fff; font-weight: 700; }
</style>
