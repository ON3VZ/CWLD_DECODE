<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
  import { formatKm } from '../lib/dxcc';
  import { PREFIX_TABLE_VERSION } from '../lib/dxcc-data';

  const p = $derived(app.qsoPlace);
  const hasGrid = $derived(app.myGrid !== '');
  const tip = $derived.by(() => {
    if (!p) return '';
    const bits = [t('place.tip.table', { version: PREFIX_TABLE_VERSION })];
    if (p.km !== null) {
      bits.push(p.basis === 'grid' ? t('place.tip.grid', { mine: app.myGrid }) : t('place.tip.country', { mine: app.myGrid }));
      if (p.longPath !== null) bits.push(t('place.tip.long', { deg: String(p.longPath) }));
    } else if (!hasGrid) bits.push(t('place.tip.nogrid'));
    return bits.join('\n');
  });
</script>

{#if p}
  <div class="place" title={tip} aria-label={t('place.label')}>
    <b>{p.country}</b>
    <span class="muted">{p.continent}</span>
    {#if p.km !== null && p.bearing !== null}
      <span class="mono">{p.bearing}&deg;</span>
      <span class="mono">{p.basis === 'country' ? '~' : ''}{formatKm(p.km)} km</span>
    {:else if p.home}
      <span class="muted">{t('place.home')}</span>
    {:else if !hasGrid}
      <span class="muted small">{t('place.nogrid')}</span>
    {/if}
  </div>
{/if}

<style>
  .place { display: flex; flex-wrap: wrap; gap: 4px 10px; align-items: baseline; font-size: 13px; }
  .small { font-size: 12px; }
</style>
