<script lang="ts">
  import { onMount } from 'svelte';
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
  import { demoSignals } from '../lib/demo';

  let canvas: HTMLCanvasElement;
  // Frequency span of the live rows, for the tone line and for clicks.
  let lo = $state(200);
  let span = $state(1200);

  const toneX = $derived(((app.audio?.tone_hz ?? 700) - lo) / span);
  const live = $derived(app.waterfallLive);

  function colour(v: number, data: Uint8ClampedArray, i: number) {
    data[i] = v * 0.35;
    data[i + 1] = v * 0.8;
    data[i + 2] = Math.min(255, v * 1.1);
    data[i + 3] = 255;
  }

  /** Click on a signal: the decoder listens to that pitch. */
  function pick(e: MouseEvent) {
    if (!live) return;
    const r = canvas.getBoundingClientRect();
    const frac = (e.clientX - r.left) / r.width;
    app.setTone(lo + frac * span);
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

<div class="wf" class:live title={live ? t('waterfall.click') : ''}>
  <canvas bind:this={canvas} width="800" height="56" onclick={pick}></canvas>
  {#if live}
    <div class="tone" style:left={`${Math.min(100, Math.max(0, toneX * 100))}%`}></div>
    <span class="hz mono">{Math.round(app.audio?.tone_hz ?? 700)} Hz</span>
  {:else if app.demo}
    <div class="tone" style:left="50%"></div>
  {:else}
    <span class="off muted">{app.audio?.error ? t('waterfall.error', { error: app.audio.error }) : t('waterfall.off')}</span>
  {/if}
</div>

<style>
  .wf { position: relative; height: 56px; background: #05080b; border-bottom: 1px solid var(--line); }
  canvas { width: 100%; height: 100%; display: block; }
  .live canvas { cursor: crosshair; }
  .tone { position: absolute; top: 0; bottom: 0; border-left: 1px dashed rgba(255, 90, 90, 0.8); pointer-events: none; }
  .hz { position: absolute; right: 8px; top: 4px; font-size: 12px; color: #ff8a8a; pointer-events: none; }
  .off { position: absolute; inset: 0; display: grid; place-items: center; font-size: 13px; }
</style>
