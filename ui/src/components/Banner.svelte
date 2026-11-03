<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
  const key = $derived(app.policy === 'live' ? 'banner.live' : app.policy === 'practice' ? 'banner.practice' : 'banner.listen');
</script>

<div class="banner {app.policy}" role="status">{t(key)}</div>

<style>
  .banner { padding: 6px 16px; font-size: 14px; font-weight: 600; }
  .live { background: color-mix(in srgb, var(--live) 25%, var(--bg)); color: var(--text); border-bottom: 2px solid var(--live); }
  .listen_only { background: color-mix(in srgb, var(--listen) 18%, var(--bg)); border-bottom: 2px solid var(--listen); }
  .practice { background: color-mix(in srgb, #8b6cf0 22%, var(--bg)); border-bottom: 2px solid #8b6cf0; }
</style>
