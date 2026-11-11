<script lang="ts">
  import { onMount } from 'svelte';
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
  import type { ClusterSpot } from '../lib/types';

  let spectrum: HTMLCanvasElement;
  let fall: HTMLCanvasElement;
  let hoverHz = $state<number | null>(null);
  let hoverX = $state(0);
  let areaW = $state(600);
  /** Seconds since 1970, moved on every 15 s so old spots fade out without new data. */
  let nowSec = $state(Math.floor(Date.now() / 1000));

  const lo = $derived(app.scopeLo);
  const hi = $derived(app.scopeHi);
  const span = $derived(Math.max(1, hi - lo));
  const vfo = $derived(app.status.frequency_hz);
  const vfoX = $derived((vfo - lo) / span);
  /** A clear step for the frequency scale: about 6 to 10 labels over the width. */
  const ticks = $derived.by(() => {
    if (!app.scopeLive) return [] as { x: number; khz: string }[];
    const steps = [1000, 2000, 5000, 10_000, 20_000, 50_000, 100_000];
    const step = steps.find((s) => span / s <= 10) ?? 100_000;
    const out: { x: number; khz: string }[] = [];
    for (let f = Math.ceil(lo / step) * step; f <= hi; f += step) {
      out.push({ x: (f - lo) / span, khz: (f / 1000).toFixed(step < 10_000 ? 1 : 0) });
    }
    return out;
  });

  /** Dark for noise, blue to white for strong signals. The radio sends 0 to 160. */
  function colour(v: number, data: Uint8ClampedArray, i: number) {
    const x = Math.max(0, Math.min(1, (v - 30) / 110));
    const c = x * x * (3 - 2 * x);
    data[i] = 8 + c * c * 247;
    data[i + 1] = 12 + c * 215;
    data[i + 2] = 18 + c * 237;
    data[i + 3] = 255;
  }

  function freqAt(e: MouseEvent): number {
    const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
    return lo + ((e.clientX - r.left) / r.width) * span;
  }

  function move(e: MouseEvent) {
    if (!app.scopeLive) return;
    hoverHz = freqAt(e);
    const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
    hoverX = (e.clientX - r.left) / r.width;
  }

  function click(e: MouseEvent) {
    if (!app.scopeLive) return;
    app.tuneTo(freqAt(e));
  }

  onMount(() => {
    const sctx = spectrum.getContext('2d');
    const fctx = fall.getContext('2d');
    if (!sctx || !fctx) return;
    let raf = 0;
    const draw = () => {
      raf = requestAnimationFrame(draw);
      const rows = app.scopeRows.splice(0);
      for (const row of rows) {
        const n = row.data.length;
        if (fall.width !== n) {
          fall.width = n;
          spectrum.width = n;
          fctx.fillStyle = '#05080b';
          fctx.fillRect(0, 0, fall.width, fall.height);
        }
        // Waterfall: scroll down one line, new line on top.
        fctx.drawImage(fall, 0, 0, n, fall.height - 1, 0, 1, n, fall.height - 1);
        const img = fctx.createImageData(n, 1);
        row.data.forEach((v, x) => colour(v, img.data, x * 4));
        fctx.putImageData(img, 0, 0);
        // Spectrum line of the newest sweep.
        const H = spectrum.height;
        sctx.fillStyle = '#05080b';
        sctx.fillRect(0, 0, n, H);
        sctx.beginPath();
        sctx.moveTo(0, H);
        row.data.forEach((v, x) => sctx.lineTo(x, H - (v / 160) * (H - 2)));
        sctx.lineTo(n, H);
        sctx.closePath();
        sctx.fillStyle = 'rgba(80, 170, 230, 0.35)';
        sctx.fill();
        sctx.strokeStyle = 'rgba(150, 215, 255, 0.9)';
        sctx.lineWidth = 1;
        sctx.stroke();
      }
    };
    raf = requestAnimationFrame(draw);
    const clock = setInterval(() => (nowSec = Math.floor(Date.now() / 1000)), 15_000);
    return () => {
      cancelAnimationFrame(raf);
      clearInterval(clock);
    };
  });
  /** The chosen size: one setting outside contest mode and one in contest mode. */
  const size = $derived((app.settings?.contest_mode ? app.settings?.scope_size_contest : app.settings?.scope_size) ?? 'medium');

  // ---- DX cluster labels ----
  const ROW_PX = 13;
  const PRIORITY = { new: 0, other_band: 1, worked: 2 } as const;
  const rowsMax = $derived(size === 'small' ? 1 : size === 'large' ? 4 : 3);
  const cluster = $derived(app.settings?.cluster);
  const showLabels = $derived(!!cluster?.show_labels && app.clusterStatus.state !== 'off');
  /** Spots in view with a row each. A crowded stretch shows the stations that are not in the log first. */
  const placed = $derived.by(() => {
    if (!app.scopeLive || !showLabels || !cluster) return { items: [] as Array<ClusterSpot & { x: number; row: number }>, hidden: 0 };
    const maxAge = Math.max(1, cluster.max_age_minutes) * 60;
    const inView = app.clusterSpots
      .filter((s) => s.freq_hz >= lo && s.freq_hz <= hi && nowSec - s.ts <= maxAge)
      .sort((a, b) => PRIORITY[a.status] - PRIORITY[b.status] || b.ts - a.ts);
    const rows: Array<Array<[number, number]>> = Array.from({ length: rowsMax }, () => []);
    const items: Array<ClusterSpot & { x: number; row: number }> = [];
    for (const s of inView) {
      const x = ((s.freq_hz - lo) / span) * areaW;
      const end = x + s.call.length * 6.7 + 10;
      const row = rows.findIndex((r) => r.every(([a, b]) => end + 3 <= a || x >= b + 3));
      if (row < 0) continue;
      rows[row].push([x, end]);
      items.push({ ...s, x: (s.freq_hz - lo) / span, row });
    }
    return { items, hidden: inView.length - items.length };
  });

  function spotTitle(s: ClusterSpot): string {
    const mins = Math.max(0, Math.round((nowSec - s.ts) / 60));
    const extra = [s.snr !== null ? `${s.snr} dB` : '', s.wpm !== null ? `${s.wpm} WPM` : ''].filter(Boolean).join(', ');
    return t('spot.title', { call: s.call, khz: (s.freq_hz / 1000).toFixed(1), spotter: s.spotter, extra: extra ? ` (${extra})` : '', mins, status: t(`spot.status.${s.status}`) });
  }

  function spotClick(e: MouseEvent, hz: number) {
    e.stopPropagation();
    app.tuneTo(hz);
  }

  const cs = $derived(app.clusterStatus);
  const clusterText = $derived(
    cs.state === 'online'
      ? t('cluster.bar.online', { node: cs.node, n: placed.items.length + placed.hidden })
      : cs.state === 'connecting'
        ? t('cluster.bar.connecting', { node: cs.node })
        : cs.state === 'offline'
          ? t('cluster.bar.offline', { node: cs.node })
          : '',
  );
</script>

<section class="scope" aria-label={t('scope.title')}>
  <div
    class="area"
    class:small={size === 'small'}
    class:large={size === 'large'}
    class:live={app.scopeLive}
    role="presentation"
    onmousemove={move}
    onmouseleave={() => (hoverHz = null)}
    onclick={click}
    bind:clientWidth={areaW}
    title={app.scopeLive ? t('scope.click') : ''}>
    <canvas class="spectrum" bind:this={spectrum} width="475" height="55"></canvas>
    <canvas class="fall" bind:this={fall} width="475" height="110"></canvas>
    {#if app.scopeLive}
      {#if vfoX >= 0 && vfoX <= 1}
        <div class="vfo" style:left={`${vfoX * 100}%`}></div>
      {/if}
      {#each ticks as k (k.khz)}
        <span class="tick mono" style:left={`${k.x * 100}%`}>{k.khz}</span>
      {/each}
      {#each placed.items as sp (`${sp.call}|${sp.band}|${sp.freq_hz}`)}
        <div class="pin {sp.status}" style:left={`${sp.x * 100}%`} style:top={`${2 + sp.row * ROW_PX}px`}></div>
        <button class="spot {sp.status}" style:left={`${sp.x * 100}%`} style:top={`${2 + sp.row * ROW_PX}px`} title={spotTitle(sp)} onclick={(e) => spotClick(e, sp.freq_hz)} onmousemove={(e) => e.stopPropagation()}>{sp.call}</button>
      {/each}
      {#if hoverHz !== null}
        <div class="hover" style:left={`${hoverX * 100}%`}></div>
        <span class="hoverlabel mono" style:left={`${Math.min(88, Math.max(0, hoverX * 100 + 0.5))}%`}>{(hoverHz / 1000).toFixed(2)} kHz</span>
      {/if}
    {:else}
      <span class="off muted">{app.scopeOutOfRange ? t('scope.out_of_range') : t('scope.waiting')}</span>
    {/if}
  </div>
  <div class="bar">
    <span class="left muted">{t('scope.title')}{#if app.scopeLive}: {t('scope.range', { lo: (lo / 1000).toFixed(1), hi: (hi / 1000).toFixed(1) })}{/if}</span>
    {#if cs.state === 'off'}
      <button class="link" title={t('cluster.scope.connect.help')} onclick={() => app.clusterConnect()}>{t('cluster.scope.connect')}</button>
    {:else}
      <span class="cl muted" title={cs.state === 'offline' && cs.reason ? cs.reason : cs.software}><i class="dot {cs.state}"></i>{clusterText}{#if placed.hidden > 0} · {t('cluster.hidden', { n: placed.hidden })}{/if}</span>
      <button class="link" onclick={() => app.setClusterLabels(!cluster?.show_labels)}>{cluster?.show_labels ? t('cluster.labels.hide') : t('cluster.labels.show')}</button>
    {/if}
    <button class="link" onclick={() => app.setView('band_scope', false)}>{t('scope.hide')}</button>
  </div>
</section>

<style>
  .scope { background: #05080b; border-bottom: 1px solid var(--line); }
  .area { position: relative; height: 165px; overflow: hidden; }
  .area.small { height: 90px; }
  .area.small .spectrum { height: 32px; }
  .area.small .fall { height: 58px; }
  .area.small .tick { top: 20px; }
  .area.large { height: 230px; }
  .area.large .spectrum { height: 80px; }
  .area.large .fall { height: 150px; }
  .area.large .tick { top: 66px; }
  .area.live { cursor: crosshair; }
  canvas { display: block; width: 100%; image-rendering: pixelated; }
  .spectrum { height: 55px; border-bottom: 1px solid rgba(120, 150, 180, 0.25); }
  .fall { height: 110px; }
  .vfo { position: absolute; top: 0; bottom: 0; border-left: 2px solid rgba(255, 90, 90, 0.85); pointer-events: none; }
  .hover { position: absolute; top: 0; bottom: 0; border-left: 1px dashed rgba(255, 255, 255, 0.6); pointer-events: none; }
  .hoverlabel { position: absolute; top: 4px; font-size: 12px; color: #fff; text-shadow: 0 0 3px #000; pointer-events: none; }
  .tick { position: absolute; top: 42px; font-size: 10px; color: rgba(190, 210, 230, 0.75); transform: translateX(-50%); pointer-events: none; text-shadow: 0 0 3px #000; }
  .off { position: absolute; inset: 0; display: grid; place-items: center; font-size: 13px; padding: 0 16px; text-align: center; }
  .bar { display: flex; gap: 14px; align-items: center; padding: 3px 12px; font-size: 12px; border-top: 1px solid var(--line); }
  .bar .left { flex: 1; }
  .cl { display: inline-flex; align-items: center; gap: 5px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 55%; }
  .dot { width: 7px; height: 7px; border-radius: 50%; background: #888; display: inline-block; flex: none; }
  .dot.online { background: #4ade80; }
  .dot.connecting { background: #fbbf24; }
  .dot.offline { background: #f87171; }
  .spot { position: absolute; z-index: 2; height: 12px; padding: 0 3px 0 4px; border: none; border-left: 2px solid currentColor; background: rgba(5, 8, 11, 0.74); font: 600 11px/12px var(--mono, ui-monospace, monospace); cursor: pointer; white-space: nowrap; }
  .spot:hover { background: rgba(5, 8, 11, 0.95); outline: 1px solid currentColor; }
  .pin { position: absolute; bottom: 0; width: 0; border-left: 1px solid currentColor; opacity: 0.45; pointer-events: none; z-index: 1; }
  .new { color: #6ee7a0; }
  .other_band { color: #fbbf24; }
  .worked { color: #f87171; }
  .spot.worked { text-decoration: line-through; opacity: 0.85; }
  .link { background: none; border: none; color: var(--accent); font-size: 12px; cursor: pointer; padding: 0; }
</style>
