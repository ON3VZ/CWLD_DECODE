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
  import LogPanel from './components/LogPanel.svelte';
  import PracticePanel from './components/PracticePanel.svelte';
  import PaddleTrainer from './components/PaddleTrainer.svelte';
  import ListenTrainer from './components/ListenTrainer.svelte';
  import LogEdit from './components/LogEdit.svelte';
  import ExportDialog from './components/ExportDialog.svelte';
  import LogBook from './components/LogBook.svelte';
  import Wizard from './components/Wizard.svelte';
  import TextEditor from './components/TextEditor.svelte';

  onMount(() => {
    createApi().then((api) => app.init(api));
    // CQ repeat only runs while the operator is at the station: minimizing CWLD stops it.
    const vis = () => {
      if (document.visibilityState === 'hidden') app.stopCqRepeat('hidden');
    };
    document.addEventListener('visibilitychange', vis);
    return () => {
      document.removeEventListener('visibilitychange', vis);
      app.dispose();
    };
  });

  function isRepeatKey(e: KeyboardEvent): boolean {
    return e.altKey && !e.ctrlKey && !e.metaKey && (e.key === 'r' || e.key === 'R' || e.code === 'KeyR');
  }

  function onKey(e: KeyboardEvent) {
    // The text editor has its own keys, Esc included (it asks first when a change is not saved).
    if (app.textEdit) return;
    // Alt+R starts or stops CQ repeat. Any other key stops it, and still does what it always does.
    if (isRepeatKey(e)) {
      e.preventDefault();
      if (!app.settingsOpen && !app.confirmLive && !app.wizardOpen) void app.toggleCqRepeat();
      return;
    }
    if (app.cqRepeat && !['Shift', 'Control', 'Alt', 'Meta', 'AltGraph', 'CapsLock'].includes(e.key)) app.stopCqRepeat('key');
    if (e.key === 'Escape') {
      if (app.wizardOpen) return;
      if (app.confirmLive) app.confirmLive = false;
      else if (app.settingsOpen) app.settingsOpen = false;
      else if (app.closeAllAsk) app.closeAllAsk = 0;
      else if (app.contestAsk) app.contestAsk = false;
      else if (app.editing) app.editing = null;
      else if (app.exportOpen) app.exportOpen = false;
      else if (app.bookOpen) app.bookOpen = false;
      else if (app.qOpen) app.qOpen = false;
      else app.stop();
      return;
    }
    if ((e.ctrlKey || e.metaKey) && (e.key === 'q' || e.key === 'Q')) {
      e.preventDefault();
      app.qOpen = !app.qOpen;
      return;
    }
    const contest = !!app.settings?.contest_mode;
    if (contest && !app.settingsOpen && !app.confirmLive && /^F[1-8]$/.test(e.key) && !e.ctrlKey) {
      e.preventDefault();
      void app.pressContestKey(e.key);
      return;
    }
    if (contest && e.ctrlKey && (e.key === 'r' || e.key === 'R')) {
      e.preventDefault();
      app.setRunMode(app.runMode === 'run' ? 'sp' : 'run');
      return;
    }
    if (contest && e.ctrlKey && (e.key === 'w' || e.key === 'W')) {
      e.preventDefault();
      app.clearEntry();
      return;
    }
    if (!contest && (e.key === 'F1' || e.key === 'F2' || e.key === 'F3' || e.key === 'F4')) {
      e.preventDefault();
      if (!app.settingsOpen && !app.confirmLive) void app.pressPileup(e.key);
      return;
    }
    // Strongest and AFC: F6 and F7, in contest mode Ctrl+F6 and Ctrl+F7 (the plain keys send messages there).
    if (e.key === 'F6' && (!contest || e.ctrlKey)) {
      e.preventDefault();
      void app.goStrongest();
      return;
    }
    if (e.key === 'F7' && (!contest || e.ctrlKey)) {
      e.preventDefault();
      void app.setAfc(!(app.settings?.audio.afc ?? true));
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
        {#if app.paddleEx}
          <PaddleTrainer />
        {:else if app.listenEx}
          <ListenTrainer />
        {:else}
          <Conversation />
          <Composer />
        {/if}
      </section>
      <aside class="side">
        {#if app.policy === 'practice' || app.practiceScore}<PracticePanel />{/if}
        <QsoPanel />
        {#if app.settings.signals_panel}
          <SignalsPanel />
        {/if}
        {#if app.settings.audio_waterfall}<AudioPanel />{/if}
        <LogPanel />
      </aside>
    </div>
    {#if app.settings.radio.kind === 'simulator'}
      <footer class="demo muted">{t('demo.note')}</footer>
    {/if}
  </div>
  {#if app.confirmLive}<ConfirmLive />{/if}
  {#if app.settingsOpen}<SettingsDialog />{/if}
  {#if app.wizardOpen}<Wizard />{/if}
  {#if app.qOpen}<QCodePanel />{/if}
  {#if app.bookOpen}<LogBook />{/if}
  {#if app.exportOpen}<ExportDialog />{/if}
  {#if app.editing}<LogEdit />{/if}
  {#if app.textEdit}<TextEditor />{/if}
  {#if app.closeAllAsk}
    <div class="scrim" role="presentation">
      <div class="ask" role="alertdialog" aria-modal="true" aria-labelledby="ca-title">
        <h2 id="ca-title">{t('tabs.closeall')}</h2>
        <p>{t('tabs.closeall.ask', { n: String(app.closeAllAsk) })}</p>
        <div class="act">
          <button class="btn" onclick={() => (app.closeAllAsk = 0)}>{t('tabs.cancel')}</button>
          <button class="btn danger" onclick={() => app.closeAllNow()}>{t('tabs.closeall.yes')}</button>
        </div>
      </div>
    </div>
  {/if}
  {#if app.contestAsk && !app.settingsOpen}
    <div class="scrim" role="presentation">
      <div class="ask" role="alertdialog" aria-modal="true" aria-labelledby="ct-title">
        <h2 id="ct-title">{t('contest.chip')}</h2>
        <p>{t('contest.ask', { nr: String(app.settings.contest_nr) })}</p>
        <div class="act">
          <button class="btn primary" onclick={() => app.answerContest(false)}>{t('contest.continue')}</button>
          <button class="btn" onclick={() => app.answerContest(true)}>{t('contest.restart')}</button>
        </div>
      </div>
    </div>
  {/if}
{:else}
  <div class="loading muted">CWLD</div>
{/if}

<style>
  .shell { display: flex; flex-direction: column; height: 100%; min-height: 0; }
  .main { flex: 1; display: grid; grid-template-columns: 1fr 300px; gap: 12px; padding: 12px; min-height: 0; overflow-y: auto; }
  .center { display: flex; flex-direction: column; min-height: min-content; gap: 10px; }
  .side { display: flex; flex-direction: column; gap: 12px; min-height: 0; overflow: auto; }
  .demo { padding: 4px 16px 8px; font-size: 12px; }
  .scrim { position: fixed; inset: 0; background: rgba(0, 0, 0, 0.6); display: grid; place-items: center; z-index: 30; }
  .ask { background: var(--panel); border: 1px solid var(--line); border-radius: 14px; padding: 18px; width: min(420px, 92vw); display: flex; flex-direction: column; gap: 10px; }
  .ask h2 { margin: 0; font-size: 18px; }
  .ask p { margin: 0; }
  .act { display: flex; gap: 8px; justify-content: flex-end; }
  .loading { display: grid; place-items: center; height: 100%; }
  .views { display: flex; gap: 14px; padding: 3px 12px; border-bottom: 1px solid var(--line); }
  .views .link { background: none; border: none; color: var(--accent); font-size: 12px; cursor: pointer; padding: 0; }
  @media (max-width: 800px) {
    .main { grid-template-columns: 1fr; grid-auto-rows: min-content; align-content: start; overflow: auto; }
    .center, .side { min-height: min-content; flex-shrink: 0; overflow: visible; }
  }
</style>
