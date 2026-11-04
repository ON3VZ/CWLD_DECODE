<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
</script>

<nav class="tabs" aria-label={t('tabs.title')}>
  <div class="list" role="tablist">
    {#each app.sessions as s (s.id)}
      {@const live = s.id === app.cur.id}
      <div class="tab" class:on={s.id === app.shown.id} class:live>
        <button role="tab" aria-selected={s.id === app.shown.id} class="name mono" onclick={() => app.show(s.id)} title={live ? t('tabs.live.help') : t('tabs.old.help')}>
          {#if live}<span class="dot"></span>{/if}{app.title(s)}
        </button>
        {#if !live}
          <button class="x" onclick={() => app.closeConversation(s.id)} aria-label={t('tabs.close')} title={t('tabs.close')}>×</button>
        {/if}
      </div>
    {/each}
  </div>
  <button class="btn newqso" onclick={() => app.newConversation()} title={t('tabs.new.help')}>+ {t('tabs.new')}</button>
</nav>

<style>
  .tabs { display: flex; align-items: center; gap: 8px; padding: 6px 12px 0; border-bottom: 1px solid var(--line); background: var(--panel); }
  .list { display: flex; gap: 4px; overflow-x: auto; flex: 1; min-width: 0; }
  .tab { display: flex; align-items: center; border: 1px solid var(--line); border-bottom: none; border-radius: 8px 8px 0 0; background: var(--panel2); white-space: nowrap; }
  .tab.on { background: var(--bg); border-color: var(--accent); }
  .name { background: none; border: none; padding: 6px 10px; font-size: 13px; color: var(--text); }
  .x { background: none; border: none; padding: 0 8px 0 0; color: var(--muted); font-size: 16px; line-height: 1; }
  .x:hover { color: var(--text); }
  .dot { display: inline-block; width: 8px; height: 8px; border-radius: 50%; background: var(--listen); margin-right: 6px; }
  .newqso { margin-bottom: 4px; white-space: nowrap; }
</style>
