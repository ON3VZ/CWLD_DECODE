<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { t } from '../lib/i18n.svelte';
  import { allProfiles, plain } from '../lib/contest';
  import type { PracticeSettings } from '../lib/types';

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
</section>

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
</style>
