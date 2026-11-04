<script lang="ts">
  import { onMount } from 'svelte';
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';

  let spectrum: HTMLCanvasElement;
  let fall: HTMLCanvasElement;
  let hoverHz = $state<number | null>(null);
  let hoverX = $state(0);

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
    return () => cancelAnimationFrame(raf);
  });
</script>

<section class="scope" aria-label={t('scope.title')}>
  <div
    class="area"
    class:live={app.scopeLive}
    role="presentation"
    onmousemove={move}
    onmouseleave={() => (hoverHz = null)}
    onclick={click}
    title={app.scopeLive ? t('scope.click') : ''}>
    <canvas class="spectrum" bind:this={spectrum} width="475" height="70"></canvas>
    <canvas class="fall" bind:this={fall} width="475" height="140"></canvas>
    {#if app.scopeLive}
      {#if vfoX >= 0 && vfoX <= 1}
        <div class="vfo" style:left={`${vfoX * 100}%`}></div>
      {/if}
      {#each ticks as k (k.khz)}
        <span class="tick mono" style:left={`${k.x * 100}%`}>{k.khz}</span>
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
    <span class="muted">{t('scope.title')}{#if app.scopeLive}: {t('scope.range', { lo: (lo / 1000).toFixed(1), hi: (hi / 1000).toFixed(1) })}{/if}</span>
    <button class="link" onclick={() => app.setView('band_scope', false)}>{t('scope.hide')}</button>
  </div>
</section>

<style>
  .scope { background: #05080b; border-bottom: 1px solid var(--line); }
  .area { position: relative; height: 210px; overflow: hidden; }
  .area.live { cursor: crosshair; }
  canvas { display: block; width: 100%; image-rendering: pixelated; }
  .spectrum { height: 70px; border-bottom: 1px solid rgba(120, 150, 180, 0.25); }
  .fall { height: 140px; }
  .vfo { position: absolute; top: 0; bottom: 0; border-left: 2px solid rgba(255, 90, 90, 0.85); pointer-events: none; }
  .hover { position: absolute; top: 0; bottom: 0; border-left: 1px dashed rgba(255, 255, 255, 0.6); pointer-events: none; }
  .hoverlabel { position: absolute; top: 4px; font-size: 12px; color: #fff; text-shadow: 0 0 3px #000; pointer-events: none; }
  .tick { position: absolute; top: 56px; font-size: 10px; color: rgba(190, 210, 230, 0.75); transform: translateX(-50%); pointer-events: none; text-shadow: 0 0 3px #000; }
  .off { position: absolute; inset: 0; display: grid; place-items: center; font-size: 13px; padding: 0 16px; text-align: center; }
  .bar { display: flex; gap: 14px; align-items: center; padding: 3px 12px; font-size: 12px; border-top: 1px solid var(--line); }
  .bar span { flex: 1; }
  .link { background: none; border: none; color: var(--accent); font-size: 12px; cursor: pointer; padding: 0; }
</style>
