<script lang="ts">
  import { onMount } from 'svelte';
  import { demoSignals } from '../lib/demo';

  let canvas: HTMLCanvasElement;

  onMount(() => {
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    const W = canvas.width;
    const H = canvas.height;
    let raf = 0;
    let last = 0;
    let phase = 0;
    const draw = (now: number) => {
      raf = requestAnimationFrame(draw);
      if (now - last < 60) return;
      last = now;
      phase += 1;
      ctx.drawImage(canvas, 0, 0, W, H - 1, 0, 1, W, H - 1);
      const row = ctx.createImageData(W, 1);
      for (let x = 0; x < W; x++) {
        let v = 18 + Math.random() * 22;
        for (const s of demoSignals) {
          const cx = W / 2 + s.offsetHz * (W / 800);
          const keyed = Math.sin((phase + s.id * 7) * (s.wpm / 90)) > -0.15;
          if (keyed) v += 150 * Math.exp(-Math.pow((x - cx) / 3, 2)) * (s.snr / 14);
        }
        v = Math.min(255, v);
        const i = x * 4;
        row.data[i] = v * 0.35;
        row.data[i + 1] = v * 0.8;
        row.data[i + 2] = Math.min(255, v * 1.1);
        row.data[i + 3] = 255;
      }
      ctx.putImageData(row, 0, 0);
    };
    raf = requestAnimationFrame(draw);
    return () => cancelAnimationFrame(raf);
  });
</script>

<div class="wf" aria-hidden="true">
  <canvas bind:this={canvas} width="800" height="56"></canvas>
  <div class="centre"></div>
</div>

<style>
  .wf { position: relative; height: 56px; background: #05080b; border-bottom: 1px solid var(--line); }
  canvas { width: 100%; height: 100%; display: block; image-rendering: pixelated; }
  .centre { position: absolute; top: 0; bottom: 0; left: 50%; border-left: 1px dashed rgba(255, 90, 90, 0.7); }
</style>
