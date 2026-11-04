<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
  import Waterfall from './Waterfall.svelte';
</script>

<section class="card">
  <div class="head">
    <h2>{t('audio.panel.title')}</h2>
    <button class="link" onclick={() => app.setView('audio_waterfall', false)}>{t('audio.panel.hide')}</button>
  </div>
  <div class="frame"><Waterfall height={110} /></div>
  <small class="muted">{t('audio.panel.help')}</small>
</section>

<style>
  .card { background: var(--panel); border: 1px solid var(--line); border-radius: 12px; padding: 12px; display: flex; flex-direction: column; gap: 6px; }
  .head { display: flex; align-items: baseline; justify-content: space-between; gap: 8px; }
  h2 { margin: 0; font-size: 15px; letter-spacing: 0.06em; text-transform: uppercase; color: var(--muted); }
  .frame { border-radius: 8px; overflow: hidden; border: 1px solid var(--line); }
  .link { background: none; border: none; color: var(--accent); font-size: 12px; cursor: pointer; padding: 0; }
  small { font-size: 11px; }
</style>
