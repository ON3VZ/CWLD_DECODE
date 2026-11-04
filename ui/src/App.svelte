<script lang="ts">
  import { onMount } from 'svelte';
  import { createApi } from './lib/api';
  import { app } from './lib/app.svelte';
  import { t } from './lib/i18n.svelte';
  import StatusBar from './components/StatusBar.svelte';
  import Banner from './components/Banner.svelte';
  import UpdateBar from './components/UpdateBar.svelte';
  import BandScope from './components/BandScope.svelte';
  import SwrAlert from './components/SwrAlert.svelte';
  import AudioPanel from './components/AudioPanel.svelte';
  import SessionTabs from './components/SessionTabs.svelte';
  import Conversation from './components/Conversation.svelte';
  import Composer from './components/Composer.svelte';
  import QsoPanel from './components/QsoPanel.svelte';
  import SignalsPanel from './components/SignalsPanel.svelte';
  import ConfirmLive from './components/ConfirmLive.svelte';
  import SettingsDialog from './components/SettingsDialog.svelte';
  import QCodePanel from './components/QCodePanel.svelte';

  onMount(() => {
    createApi().then((api) => app.init(api));
    return () => app.dispose();
  });

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      if (app.confirmLive) app.confirmLive = false;
      else if (app.settingsOpen) app.settingsOpen = false;
      else if (app.qOpen) app.qOpen = false;
      else app.stop();
      return;
    }
    if (e.key === 'F1') {
      e.preventDefault();
      app.qOpen = !app.qOpen;
      return;
    }
    const tag = (e.target as HTMLElement | null)?.tagName;
    if ((e.key === 's' || e.key === 'S') && tag !== 'INPUT' && tag !== 'TEXTAREA' && tag !== 'SELECT' && !e.ctrlKey && !e.metaKey) {
      app.toggleSignals();
    }
  }
</script>

<svelte:window onkeydown={onKey} />

{#if app.ready && app.settings}
  <div class="shell">
    <StatusBar />
    <SwrAlert />
    <Banner />
    <UpdateBar />
    {#if app.settings.band_scope}<BandScope />{/if}
    {#if !app.settings.band_scope || !app.settings.audio_waterfall}
      <div class="views">
        {#if !app.settings.band_scope}<button class="link" onclick={() => app.setView('band_scope', true)}>{t('scope.show')}</button>{/if}
        {#if !app.settings.audio_waterfall}<button class="link" onclick={() => app.setView('audio_waterfall', true)}>{t('scope.audio.show')}</button>{/if}
      </div>
    {/if}
    <SessionTabs />
    <div class="main" class:with-signals={app.settings.signals_panel}>
      <section class="center" aria-label={t('conversation.title')}>
        <Conversation />
        <Composer />
      </section>
      <aside class="side">
        <QsoPanel />
        {#if app.settings.signals_panel}
          <SignalsPanel />
        {/if}
        {#if app.settings.audio_waterfall}<AudioPanel />{/if}
      </aside>
    </div>
    {#if app.settings.radio.kind === 'simulator'}
      <footer class="demo muted">{t('demo.note')}</footer>
    {/if}
  </div>
  {#if app.confirmLive}<ConfirmLive />{/if}
  {#if app.settingsOpen}<SettingsDialog />{/if}
  {#if app.qOpen}<QCodePanel />{/if}
{:else}
  <div class="loading muted">CWLD</div>
{/if}

<style>
  .shell { display: flex; flex-direction: column; height: 100%; min-height: 0; }
  .main { flex: 1; display: grid; grid-template-columns: 1fr 300px; gap: 12px; padding: 12px; min-height: 0; }
  .center { display: flex; flex-direction: column; min-height: 0; gap: 10px; }
  .side { display: flex; flex-direction: column; gap: 12px; min-height: 0; overflow: auto; }
  .demo { padding: 4px 16px 8px; font-size: 12px; }
  .loading { display: grid; place-items: center; height: 100%; }
  .views { display: flex; gap: 14px; padding: 3px 12px; border-bottom: 1px solid var(--line); }
  .views .link { background: none; border: none; color: var(--accent); font-size: 12px; cursor: pointer; padding: 0; }
  @media (max-width: 800px) {
    .main { grid-template-columns: 1fr; overflow: auto; }
  }
</style>
