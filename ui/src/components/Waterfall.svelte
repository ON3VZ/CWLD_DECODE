<script lang="ts">
  import { onMount } from 'svelte';
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
  import { demoSignals } from '../lib/demo';

  /** Height in pixels: a slim strip, or taller inside the side panel. */
  let { height = 40 }: { height?: number } = $props();

  let canvas: HTMLCanvasElement;
  // Frequency span of the live rows, for the tone line and for clicks.
  let lo = $state(200);
  let span = $state(1200);

  /** 1: the whole strip. 2 and 4: a closer look around the pitch. */
  let zoom = $state(2);
  const toneHz = $derived(app.audio?.tone_hz ?? 700);
  /** Left edge of the visible part as a fraction of the full strip, centred on the pitch. */
  const left = $derived(Math.min(1 - 1 / zoom, Math.max(0, (toneHz - lo) / span - 0.5 / zoom)));
  const toneX = $derived(((toneHz - lo) / span - left) * zoom);
  const ticks = $derived.by(() => {
    const out: { x: number; hz: number }[] = [];
    const from = lo + left * span;
    const to = from + span / zoom;
    const step = zoom === 1 ? 200 : zoom === 2 ? 100 : 50;
    for (let hz = Math.ceil(from / step) * step; hz < to; hz += step) out.push({ x: ((hz - lo) / span - left) * zoom, hz });
    return out;
  });
  const live = $derived(app.waterfallLive);
  /** Edges of the decoder filter (about -3 dB), as a position in the strip. The whole width is about 1.92 times the setting. */
  const edges = $derived.by(() => {
    const bw = app.bandwidthHz;
    if (!bw || !live) return null;
    const half = 0.96 * bw;
    const x = (hz: number) => ((hz - lo) / span - left) * zoom;
    return { a: x(toneHz - half), b: x(toneHz + half), hz: Math.round(half * 2) };
  });

  /** Dark for noise, blue to white only for real signals. A curve keeps the noise quiet. */
  function colour(v: number, data: Uint8ClampedArray, i: number) {
    const x = Math.max(0, Math.min(1, (v - 70) / 185));
    const c = x * x * (3 - 2 * x);
    data[i] = 8 + c * c * 247;
    data[i + 1] = 12 + c * 215;
    data[i + 2] = 18 + c * 237;
    data[i + 3] = 255;
  }

  /** Click on a signal: the decoder listens to that pitch. */
  function pick(e: MouseEvent) {
    if (!live) return;
    const r = canvas.getBoundingClientRect();
    const frac = (e.clientX - r.left) / r.width;
    // The canvas is wider than the strip when zoomed, so this is the fraction of the full range.
    app.setTone(lo + frac * span, true);
  }

  onMount(() => {
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    let raf = 0;
    let last = 0;
    let phase = 0;
    const draw = (now: number) => {
      raf = requestAnimationFrame(draw);
      if (app.waterfallLive) {
        const rows = app.waterfallRows.splice(0);
        for (const row of rows) {
          if (canvas.width !== row.bins.length) {
            canvas.width = row.bins.length;
            ctx.fillStyle = '#05080b';
            ctx.fillRect(0, 0, canvas.width, canvas.height);
          }
          lo = row.lo_hz;
          span = row.bins.length * row.hz_per_bin;
          ctx.drawImage(canvas, 0, 0, canvas.width, canvas.height - 1, 0, 1, canvas.width, canvas.height - 1);
          const img = ctx.createImageData(canvas.width, 1);
          row.bins.forEach((v, x) => colour(v, img.data, x * 4));
          ctx.putImageData(img, 0, 0);
        }
        return;
      }
      if (!app.demo) return;
      // Demo picture for the browser version: three made-up stations.
      if (now - last < 60) return;
      last = now;
      phase += 1;
      const W = canvas.width;
      ctx.drawImage(canvas, 0, 0, W, canvas.height - 1, 0, 1, W, canvas.height - 1);
      const row = ctx.createImageData(W, 1);
      for (let x = 0; x < W; x++) {
        let v = 18 + Math.random() * 22;
        for (const s of demoSignals) {
          const cx = W / 2 + s.offsetHz * (W / 800);
          const keyed = Math.sin((phase + s.id * 7) * (s.wpm / 90)) > -0.15;
          if (keyed) v += 150 * Math.exp(-Math.pow((x - cx) / 3, 2)) * (s.snr / 14);
        }
        colour(Math.min(255, v), row.data, x * 4);
      }
      ctx.putImageData(row, 0, 0);
    };
    raf = requestAnimationFrame(draw);
    return () => cancelAnimationFrame(raf);
  });
</script>

<div class="wf" class:live style:height={`${height}px`} title={live ? t('waterfall.click') : ''}>
  <canvas
    bind:this={canvas}
    width="800"
    height={height}
    onclick={pick}
    style:width={`${zoom * 100}%`}
    style:margin-left={`${-left * zoom * 100}%`}></canvas>
  {#if live}
    {#each ticks as k (k.hz)}
      <span class="tick mono" style:left={`${k.x * 100}%`}>{k.hz}</span>
    {/each}
    {#if edges}
      <div class="edge" style:left={`${Math.min(100, Math.max(0, edges.a * 100))}%`} title={t('waterfall.filter', { hz: String(edges.hz) })}></div>
      <div class="edge" style:left={`${Math.min(100, Math.max(0, edges.b * 100))}%`} title={t('waterfall.filter', { hz: String(edges.hz) })}></div>
    {/if}
    <div class="tone" style:left={`${Math.min(100, Math.max(0, toneX * 100))}%`}></div>
    <span class="hz mono" style:left={`${Math.min(96, Math.max(0, toneX * 100 + 0.5))}%`}>{Math.round(toneHz)} Hz</span>
    <div class="zoom" role="group" aria-label={t('waterfall.zoom')}>
      {#each [1, 2, 4] as z (z)}
        <button class:on={zoom === z} onclick={() => (zoom = z)} title={t('waterfall.zoom')}>{z}x</button>
      {/each}
    </div>
  {:else if app.demo}
    <div class="tone" style:left="50%"></div>
  {:else}
    <span class="off muted">{app.audio?.error ? t('waterfall.error', { error: app.audio.error }) : t('waterfall.off')}</span>
  {/if}
</div>

<style>
  .wf { position: relative; background: #05080b; border-bottom: 1px solid var(--line); overflow: hidden; }
  canvas { height: 100%; display: block; }
  .live canvas { cursor: crosshair; }
  .tone { position: absolute; top: 0; bottom: 0; border-left: 1px dashed rgba(255, 90, 90, 0.8); pointer-events: none; }
  .edge { position: absolute; top: 0; bottom: 0; border-left: 1px solid rgba(255, 70, 70, 0.9); pointer-events: none; }
  .hz { position: absolute; top: 3px; font-size: 12px; color: #ff8a8a; pointer-events: none; text-shadow: 0 0 3px #000; }
  .tick { position: absolute; bottom: 1px; font-size: 10px; color: rgba(190, 210, 230, 0.6); pointer-events: none; transform: translateX(-50%); text-shadow: 0 0 3px #000; }
  .zoom { position: absolute; right: 6px; top: 4px; display: flex; gap: 2px; }
  .zoom button { background: rgba(5, 8, 11, 0.7); border: 1px solid var(--line); color: var(--muted); font-size: 11px; padding: 0 6px; border-radius: 4px; }
  .zoom button.on { color: var(--text); border-color: var(--accent); }
  .off { position: absolute; inset: 0; display: grid; place-items: center; font-size: 13px; }
</style>
