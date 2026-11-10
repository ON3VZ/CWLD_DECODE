<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
  import { allProfiles, plain } from '../lib/contest';
  import type { PracticeSettings } from '../lib/types';
  import { PADDLE_EXERCISES, KOCH, exerciseFits, kochPool, kochReady, weakest, type PaddleExercise } from '../lib/paddle';

  /** Kristof's own app for learning morse by ear (letters, Koch method). */
  const LEARN_URL = 'https://on3vz.github.io/CWLD/';

  const cfg = $derived(app.settings?.practice);
  const run = $derived(app.practiceRun);
  let now = $state(Date.now() / 1000);
  $effect(() => {
    const id = setInterval(() => (now = Date.now() / 1000), 1000);
    return () => clearInterval(id);
  });
  const clock = (s: number) => `${Math.floor(s / 60)}:${String(Math.floor(s % 60)).padStart(2, '0')}`;

  function set<K extends keyof PracticeSettings>(k: K, v: PracticeSettings[K]) {
    app.setPracticeCfg({ [k]: v } as Partial<PracticeSettings>);
  }
  const num = (e: Event) => Number((e.currentTarget as HTMLInputElement).value);

  const prog = $derived(cfg?.paddle);
  let paddleEx = $state<PaddleExercise>('chars');
  $effect(() => {
    const saved = prog?.exercise as PaddleExercise | undefined;
    if (saved && PADDLE_EXERCISES.includes(saved)) paddleEx = saved;
  });
  const recent = $derived((prog?.sessions ?? []).slice(-5).reverse());
  const weak = $derived(weakest(prog?.stats ?? []));
  const day = (ts: number) => new Date(ts * 1000).toISOString().slice(5, 16).replace('T', ' ');
  let confirmReset = $state(false);

  const koch = $derived(cfg?.koch);
  const inUse = $derived(kochPool(koch?.chars));
  const newest = $derived(inUse.length < KOCH.length ? inUse[inUse.length - 1] : '');
  const listenWeak = $derived(weakest(koch?.stats ?? []));
  const ready = $derived(kochReady(koch?.sessions ?? [], inUse.length));
  let confirmListenReset = $state(false);
  /** Click a character to use it or leave it out. At least two stay. */
  function toggle(c: string) {
    const next = inUse.includes(c) ? inUse.filter((x) => x !== c) : [...inUse, c];
    if (next.length < 2) return;
    app.setKoch({ chars: next.join('') });
  }
</script>

<section class="card" aria-label={t('practice.title')}>
  <div class="head">
    <h2>{t('practice.title')}</h2>
    <a class="link" href={LEARN_URL} target="_blank" rel="noreferrer" title={t('practice.learn.help')}>{t('practice.learn')}</a>
  </div>

  {#if cfg && !run}
    <label>{t('practice.exercise')}
      <select value={cfg.exercise} onchange={(e) => set('exercise', (e.currentTarget as HTMLSelectElement).value as PracticeSettings['exercise'])}>
        <option value="they_cq">{t('practice.ex.they_cq')}</option>
        <option value="you_cq">{t('practice.ex.you_cq')}</option>
        <option value="contest">{t('practice.ex.contest')}</option>
      </select>
    </label>
    {#if cfg.exercise === 'contest'}
      <label>{t('contest.profile')}
        <select value={app.settings?.contest.active ?? ''} onchange={(e) => app.selectContestProfile((e.currentTarget as HTMLSelectElement).value)}>
          <option value="">{t('contest.profile.none')}</option>
          {#each allProfiles(app.settings!.contest) as p (p.id)}<option value={p.id}>{plain(p.name, t)}</option>{/each}
        </select>
      </label>
      <div class="two">
        <label>{t('practice.pileup')}<input class="mono" type="number" min="1" max="8" value={cfg.pileup} onchange={(e) => set('pileup', Math.min(8, Math.max(1, num(e) || 1)))} /></label>
        <label>{t('practice.minutes')}<input class="mono" type="number" min="0" max="120" step="5" value={cfg.minutes} onchange={(e) => set('minutes', Math.min(120, Math.max(0, num(e) || 0)))} /></label>
      </div>
    {/if}
    <div class="two">
      <label>{t('practice.wpm_min')}<input class="mono" type="number" min="5" max="60" value={cfg.wpm_min} onchange={(e) => set('wpm_min', Math.min(60, Math.max(5, num(e) || 5)))} /></label>
      <label>{t('practice.wpm_max')}<input class="mono" type="number" min="5" max="60" value={cfg.wpm_max} onchange={(e) => set('wpm_max', Math.min(60, Math.max(5, num(e) || 5)))} /></label>
    </div>
    <label>{t('practice.strength')}
      <select value={cfg.strength} onchange={(e) => set('strength', (e.currentTarget as HTMLSelectElement).value as PracticeSettings['strength'])}>
        <option value="weak">{t('practice.strength.weak')}</option>
        <option value="medium">{t('practice.strength.medium')}</option>
        <option value="strong">{t('practice.strength.strong')}</option>
      </select>
    </label>
    <label class="check"><input type="checkbox" checked={cfg.noise} onchange={(e) => set('noise', (e.currentTarget as HTMLInputElement).checked)} /> {t('practice.noise')}</label>
    <label class="check"><input type="checkbox" checked={cfg.qsb} onchange={(e) => set('qsb', (e.currentTarget as HTMLInputElement).checked)} /> {t('practice.qsb')}</label>
  {/if}

  {#if cfg}
    <label class="check" title={t('practice.help.help')}><input type="checkbox" checked={cfg.decoder_help} onchange={(e) => set('decoder_help', (e.currentTarget as HTMLInputElement).checked)} /> {t('practice.help')}</label>
    <label>{t('practice.volume')}<input type="range" min="0" max="1" step="0.05" value={cfg.volume} onchange={(e) => set('volume', num(e))} /></label>
  {/if}

  {#if run}
    <div class="running">
      <span class="dot"></span>
      <span>{t(`practice.ex.${run.exercise}`)}</span>
      <span class="mono">{run.endsAt ? clock(Math.max(0, run.endsAt - now)) : clock(Math.max(0, now - run.startedTs))}</span>
    </div>
    <button class="btn" onclick={() => app.stopExercise()}>{t('practice.stop')}</button>
  {:else}
    <p class="muted small">{t('practice.af')}</p>
    <button class="btn primary" disabled={!app.settings} onclick={() => app.startExercise()}>{t('practice.start')}</button>
  {/if}
  {#if cfg}
    <label class="check" title={t('practice.quiet.help')}><input type="checkbox" checked={cfg.quiet_radio} onchange={(e) => set('quiet_radio', (e.currentTarget as HTMLInputElement).checked)} /> {t('practice.quiet')}</label>
    {#if app.radioQuiet}<small class="muted">{t('paddle.quiet')}</small>{/if}
    {#if app.radioControls && app.status.connected}
      {@const c = app.radioControls}
      <small class="muted" title={t('practice.controls.help')}>{t('practice.controls', {
        af: c.af_level === null ? t('practice.controls.none') : `${Math.round((c.af_level / 255) * 100)}%`,
        sql: c.squelch === null ? t('practice.controls.none') : c.squelch >= 255 ? t('practice.controls.closed') : `${Math.round((c.squelch / 255) * 100)}%`,
        bk: c.break_in === null ? t('practice.controls.none') : t(`practice.controls.bk${c.break_in}`),
      })}</small>
    {/if}
  {/if}
</section>

{#if cfg && !run && app.policy === 'practice'}
  <section class="card" aria-label={t('paddle.title')}>
    <div class="head"><h2>{t('paddle.title')}</h2></div>
    <p class="muted small">{t('paddle.intro')}</p>
    <label>{t('practice.exercise')}
      <select bind:value={paddleEx}>
        {#each PADDLE_EXERCISES as k (k)}<option value={k}>{t(`paddle.ex.${k}`)}</option>{/each}
      </select>
    </label>
    {#if !exerciseFits(paddleEx, koch?.chars)}<small class="muted">{t('koch.fallback')}</small>{/if}
    {#if paddleEx === 'weak' && weak.length < 2}<small class="muted">{t('paddle.weak.none')}</small>{/if}
    {#if app.paddleEx}
      <button class="btn" onclick={() => app.paddleStop()}>{t('paddle.stop')}</button>
    {:else}
      <button class="btn primary" onclick={() => app.paddleStart(paddleEx)}>{t('paddle.start')}</button>
    {/if}
    {#if weak.length}
      <div class="small"><span class="muted">{t('paddle.weak')}</span> {#each weak as w (w.ch)}<span class="chip mono" title={t('paddle.weak.rate', { errors: String(w.errors), tries: String(w.tries) })}>{w.ch}</span>{/each}</div>
    {/if}
    {#if recent.length}
      <table class="small">
        <thead><tr><th>{t('paddle.when')}</th><th>{t('paddle.score')}</th><th>{t('paddle.wpm')}</th></tr></thead>
        <tbody>{#each recent as s (s.ts)}<tr><td class="mono">{day(s.ts)}</td><td class="mono">{s.score}</td><td class="mono">{Math.round(s.wpm)}</td></tr>{/each}</tbody>
      </table>
      {#if confirmReset}
        <div class="row"><button class="btn" onclick={() => { app.paddleReset(); confirmReset = false; }}>{t('paddle.reset.yes')}</button><button class="link" onclick={() => (confirmReset = false)}>{t('paddle.reset.no')}</button></div>
      {:else}
        <button class="link small" onclick={() => (confirmReset = true)}>{t('paddle.reset')}</button>
      {/if}
    {/if}
  </section>
{/if}

{#if cfg && koch && !run}
  <section class="card" aria-label={t('koch.title')}>
    <div class="head"><h2>{t('koch.title')}</h2></div>
    <p class="muted small">{t('koch.intro')}</p>
    <label>{t('koch.count', { n: String(inUse.length), total: String(KOCH.length) })}
      <input type="range" min="2" max={KOCH.length} value={inUse.length} oninput={(e) => app.setKochCount(num(e))} aria-label={t('koch.count.help')} />
    </label>
    <div class="chars" role="group" aria-label={t('koch.chars.help')}>
      {#each [...KOCH] as c (c)}
        <button class="ch mono" class:on={inUse.includes(c)} class:new={c === newest} aria-pressed={inUse.includes(c)} title={c === newest ? t('koch.newest') : t('koch.toggle')} onclick={() => toggle(c)}>{c}</button>
      {/each}
    </div>
    <small class="muted">{t('koch.chars.hint')}</small>
    {#if ready}<p class="advice small" role="status">{t('koch.ready', { next: KOCH[inUse.length] })}
      <button class="btn" onclick={() => app.setKochCount(inUse.length + 1)}>{t('koch.add', { next: KOCH[inUse.length] })}</button></p>{/if}
    <h3>{t('koch.listen.title')}</h3>
    <p class="muted small">{t('koch.listen.intro')}</p>
    <label>{t('koch.listen.kind')}
      <select value={koch.listen} onchange={(e) => app.setKoch({ listen: (e.currentTarget as HTMLSelectElement).value as 'groups' | 'words' })}>
        <option value="groups">{t('koch.listen.groups')}</option>
        <option value="words">{t('koch.listen.words')}</option>
      </select>
    </label>
    {#if koch.listen === 'words' && !exerciseFits('words', koch.chars)}<small class="muted">{t('koch.fallback')}</small>{/if}
    <div class="two">
      <label>{t('koch.listen.char_wpm')}<input class="mono" type="number" min="10" max="40" value={koch.char_wpm} onchange={(e) => app.setKoch({ char_wpm: num(e) || 20 })} /></label>
      <label title={t('koch.listen.eff.help')}>{t('koch.listen.eff_wpm')}<input class="mono" type="number" min="5" max={koch.char_wpm} value={koch.eff_wpm} onchange={(e) => app.setKoch({ eff_wpm: num(e) || 12 })} /></label>
    </div>
    <label>{t('koch.listen.tone')}<input class="mono" type="number" min="400" max="900" step="10" value={koch.tone_hz} onchange={(e) => app.setKoch({ tone_hz: num(e) || 650 })} /></label>
    {#if app.listenEx}
      <button class="btn" onclick={() => app.listenStop()}>{t('koch.listen.stop')}</button>
    {:else}
      <button class="btn primary" onclick={() => app.listenStart()}>{t('koch.listen.start')}</button>
    {/if}
    {#if listenWeak.length}
      <div class="small"><span class="muted">{t('paddle.weak')}</span> {#each listenWeak as w (w.ch)}<span class="chip mono" title={t('paddle.weak.rate', { errors: String(w.errors), tries: String(w.tries) })}>{w.ch}</span>{/each}</div>
    {/if}
    {#if koch.sessions.length}
      {@const last = koch.sessions.slice(-5).reverse()}
      <table class="small">
        <thead><tr><th>{t('paddle.when')}</th><th>{t('koch.listen.right')}</th><th>{t('koch.listen.n')}</th></tr></thead>
        <tbody>{#each last as s (s.ts)}<tr><td class="mono">{day(s.ts)}</td><td class="mono">{Math.round(s.accuracy * 100)}%</td><td class="mono">{s.n}</td></tr>{/each}</tbody>
      </table>
      {#if confirmListenReset}
        <div class="row"><button class="btn" onclick={() => { app.listenReset(); confirmListenReset = false; }}>{t('paddle.reset.yes')}</button><button class="link" onclick={() => (confirmListenReset = false)}>{t('paddle.reset.no')}</button></div>
      {:else}
        <button class="link small" onclick={() => (confirmListenReset = true)}>{t('paddle.reset')}</button>
      {/if}
    {/if}
  </section>
{/if}

{#if app.practiceScore}
  {@const s = app.practiceScore}
  <div class="scrim" role="presentation">
    <div class="dlg" role="dialog" aria-modal="true" aria-labelledby="score-title">
      <h2 id="score-title">{t('practice.score.title')}</h2>
      <div class="nums">
        <div><span class="big">{s.correct}</span><span class="muted">{t('practice.score.correct', { n: String(s.logged) })}</span></div>
        <div><span class="big">{s.rate}</span><span class="muted">{t('practice.score.rate')}</span></div>
        <div><span class="big">{Math.round(s.minutes)}</span><span class="muted">{t('practice.score.minutes')}</span></div>
      </div>
      {#if s.busted || s.wrongExchange || s.missed.length}
        <p class="small">{t('practice.score.summary', { busted: String(s.busted), wrong: String(s.wrongExchange), missed: String(s.missed.length) })}</p>
      {/if}
      {#if s.lines.length}
        <ul class="lines">
          {#each s.lines as l}
            <li class:bad={!l.ok}>
              <span class="mono">{l.call}</span>
              {#if l.truth}<span class="small">{t('practice.score.was', { call: l.truth })}</span>{/if}
              {#if !l.ok}<span class="small">{l.problems.map((p) => t(`practice.problem.${p}`)).join(', ')}</span>{:else}<span class="small ok">{t('practice.score.ok')}</span>{/if}
            </li>
          {/each}
        </ul>
      {:else}
        <p class="muted small">{t('practice.score.none')}</p>
      {/if}
      {#if s.missed.length}<p class="small">{t('practice.score.missed', { calls: s.missed.join(' ') })}</p>{/if}
      <div class="act">
        <button class="btn primary" onclick={() => (app.practiceScore = null)}>{t('practice.score.close')}</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .chip { display: inline-block; background: var(--panel2); border: 1px solid var(--line); border-radius: 999px; padding: 0 8px; margin: 2px 2px 0 0; }
  table { border-collapse: collapse; width: 100%; }
  th, td { text-align: left; padding: 2px 6px 2px 0; border-bottom: 1px solid var(--line); }
  .row { display: flex; gap: 8px; align-items: center; }
  .card { background: var(--panel); border: 1px solid #8b6cf0; border-radius: 12px; padding: 10px 12px; display: flex; flex-direction: column; gap: 6px; }
  .head { display: flex; align-items: baseline; gap: 8px; }
  h2 { margin: 0; font-size: 13px; letter-spacing: 0.06em; text-transform: uppercase; color: var(--muted); flex: 1; }
  .link { color: var(--accent); font-size: 12px; }
  label { display: flex; flex-direction: column; font-size: 12px; color: var(--muted); gap: 2px; }
  label.check { flex-direction: row; align-items: center; gap: 6px; font-size: 13px; color: var(--text); }
  input:not([type='checkbox']):not([type='range']), select { background: var(--panel2); border: 1px solid var(--line); border-radius: 8px; padding: 5px 8px; font-size: 14px; color: var(--text); }
  .two { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
  .small { font-size: 12px; }
  p { margin: 0; }
  .running { display: flex; align-items: center; gap: 8px; font-weight: 600; }
  .running .mono { margin-left: auto; font-size: 18px; }
  .dot { width: 10px; height: 10px; border-radius: 50%; background: #8b6cf0; animation: pulse 1.2s infinite; }
  @keyframes pulse { 50% { opacity: 0.3; } }
  .scrim { position: fixed; inset: 0; background: rgba(0, 0, 0, 0.6); display: grid; place-items: center; z-index: 30; }
  .dlg { background: var(--panel); border: 1px solid var(--line); border-radius: 14px; padding: 18px; width: min(520px, 94vw); max-height: 88vh; overflow: auto; display: flex; flex-direction: column; gap: 10px; }
  .dlg h2 { font-size: 18px; color: var(--text); text-transform: none; letter-spacing: normal; flex: none; }
  .nums { display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px; }
  .nums div { display: flex; flex-direction: column; align-items: center; background: var(--panel2); border-radius: 10px; padding: 8px; }
  .big { font-size: 28px; font-weight: 800; }
  .nums .muted { font-size: 12px; text-align: center; }
  .lines { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 4px; }
  .lines li { display: flex; gap: 10px; align-items: baseline; }
  .lines li.bad { color: var(--warn); }
  .ok { color: var(--listen); }
  .act { display: flex; justify-content: flex-end; }
  h3 { margin: 6px 0 0; font-size: 13px; }
  .chars { display: flex; flex-wrap: wrap; gap: 4px; }
  .ch { min-width: 28px; height: 30px; border-radius: 8px; border: 1px solid var(--line); background: var(--panel2); color: var(--muted); font-size: 15px; }
  .ch.on { background: var(--accent); color: #fff; border-color: var(--accent); }
  .ch.new { outline: 2px solid #8b6cf0; outline-offset: 1px; }
  .advice { background: var(--panel2); border-radius: 8px; padding: 6px 8px; display: flex; flex-direction: column; gap: 6px; align-items: flex-start; }
</style>
