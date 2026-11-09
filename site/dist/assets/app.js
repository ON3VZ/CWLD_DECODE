/* CWLD website: hero waterfall, typed demo, morse toy and the latest release lookup. No tracking, no third parties except the GitHub API call for the download. */
(function () {
  'use strict';
  var reduce = window.matchMedia && window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  var REPO = 'ON3VZ/CWLD_DECODE';

  // Menu button (small screens)
  var mb = document.querySelector('.menu-btn');
  if (mb) mb.addEventListener('click', function () {
    var nav = document.querySelector('nav.main');
    var open = nav.classList.toggle('open');
    mb.setAttribute('aria-expanded', open ? 'true' : 'false');
  });

  // ---------- Morse table ----------
  var MORSE = { A: '.-', B: '-...', C: '-.-.', D: '-..', E: '.', F: '..-.', G: '--.', H: '....', I: '..', J: '.---', K: '-.-', L: '.-..', M: '--', N: '-.', O: '---', P: '.--.', Q: '--.-', R: '.-.', S: '...', T: '-', U: '..-', V: '...-', W: '.--', X: '-..-', Y: '-.--', Z: '--..', 0: '-----', 1: '.----', 2: '..---', 3: '...--', 4: '....-', 5: '.....', 6: '-....', 7: '--...', 8: '---..', 9: '----.', '/': '-..-.', '?': '..--..', '.': '.-.-.-', ',': '--..--', '=': '-...-' };

  // ---------- Hero waterfall ----------
  var cv = document.getElementById('wf');
  if (cv && cv.getContext) {
    var ctx = cv.getContext('2d');
    var W, H, cols = 160, rowsH = 2, img, sigs = [], t0 = 0;
    var resize = function () {
      var r = cv.getBoundingClientRect();
      W = cv.width = Math.max(320, Math.floor(r.width / 3));
      H = cv.height = Math.max(160, Math.floor(r.height / 3));
      cols = W;
      ctx.fillStyle = '#05080c';
      ctx.fillRect(0, 0, W, H);
    };
    resize();
    window.addEventListener('resize', resize);
    var mk = function () {
      var word = ['CQ CQ DE ON3VZ K', 'TEST DL1ABC', 'QRZ? 73', 'DE PA3XYZ K', 'UR 5NN', 'TNX FER QSO'][Math.floor(Math.random() * 6)];
      var bits = [];
      var unit = 2 + Math.floor(Math.random() * 2);
      word.split('').forEach(function (c) {
        if (c === ' ') { bits.push([0, unit * 4]); return; }
        (MORSE[c] || '').split('').forEach(function (e) { bits.push([1, e === '.' ? unit : unit * 3]); bits.push([0, unit]); });
        bits.push([0, unit * 2]);
      });
      var flat = [];
      bits.forEach(function (b) { for (var i = 0; i < b[1]; i++) flat.push(b[0]); });
      return { x: 0.08 + Math.random() * 0.84, w: 1 + Math.random() * 1.6, hue: Math.random() < 0.7 ? 155 : 195, amp: 0.5 + Math.random() * 0.5, flat: flat, i: -Math.floor(Math.random() * 80) };
    };
    for (var k = 0; k < 5; k++) sigs.push(mk());
    var draw = function () {
      ctx.drawImage(cv, 0, rowsH);
      ctx.fillStyle = '#05080c';
      ctx.fillRect(0, 0, W, rowsH);
      var row = ctx.createImageData(W, rowsH);
      for (var x = 0; x < W; x++) {
        var v = 14 + Math.random() * 22;
        for (var s = 0; s < sigs.length; s++) {
          var g = sigs[s];
          if (g.i >= 0 && g.i < g.flat.length && g.flat[g.i]) {
            var d = (x - g.x * W) / (g.w * 1.4);
            v += 190 * g.amp * Math.exp(-d * d);
          }
        }
        v = Math.min(255, v);
        for (var r = 0; r < rowsH; r++) {
          var o = (r * W + x) * 4;
          var f = v / 255;
          row.data[o] = Math.floor(10 + 40 * f * f);
          row.data[o + 1] = Math.floor(30 + 225 * f);
          row.data[o + 2] = Math.floor(40 + 150 * f * f);
          row.data[o + 3] = 255;
        }
      }
      ctx.putImageData(row, 0, 0);
      for (var q = 0; q < sigs.length; q++) {
        sigs[q].i++;
        if (sigs[q].i > sigs[q].flat.length + 60) sigs[q] = mk();
      }
    };
    if (reduce) { for (var n = 0; n < H / rowsH; n++) draw(); }
    else {
      var last = 0;
      var loop = function (ts) {
        if (ts - last > 55) { draw(); last = ts; }
        requestAnimationFrame(loop);
      };
      requestAnimationFrame(loop);
    }
  }

  // ---------- Typed conversation demo ----------
  var demo = document.getElementById('demo');
  if (demo) {
    var script = JSON.parse(demo.getAttribute('data-script'));
    var meter = demo.querySelector('.meter i');
    var feed = demo.querySelector('.feed');
    var run = function () {
      feed.innerHTML = '';
      var idx = 0;
      var next = function () {
        if (idx >= script.length) { setTimeout(run, 4500); return; }
        var m = script[idx++];
        var b = document.createElement('div');
        b.className = 'bub' + (m.me ? ' me' : '');
        b.innerHTML = '<span class="who"></span><span class="txt caret"></span>';
        b.querySelector('.who').textContent = m.who;
        feed.appendChild(b);
        var txt = b.querySelector('.txt');
        var i = 0;
        var tick = function () {
          i++;
          var s = m.text.slice(0, i);
          // The first callsign is highlighted once it is complete.
          txt.innerHTML = s.replace(/[<>&]/g, function (c) { return { '<': '&lt;', '>': '&gt;', '&': '&amp;' }[c]; }).replace(/\b([A-Z0-9]{1,2}\d[A-Z]{1,4})\b(?= )/g, '<span class="call">$1</span>');
          if (meter) meter.style.width = (m.me ? 100 : 55 + Math.min(40, i * 2)) + '%';
          if (i < m.text.length) setTimeout(tick, reduce ? 0 : 55 + Math.random() * 70);
          else { txt.classList.remove('caret'); setTimeout(next, reduce ? 0 : 900); }
        };
        tick();
      };
      next();
    };
    run();
  }

  // ---------- Morse toy ----------
  var toy = document.getElementById('toy');
  if (toy) {
    var inp = toy.querySelector('input[type=text]');
    var out = toy.querySelector('.morse');
    var wpmEl = toy.querySelector('input[type=range]');
    var wpmOut = toy.querySelector('.wpm');
    var btn = toy.querySelector('button');
    var stopLabel = btn.getAttribute('data-stop');
    var playLabel = btn.textContent;
    var AC = window.AudioContext || window.webkitAudioContext;
    var actx = null, osc = null, timers = [];
    var render = function () {
      out.innerHTML = '';
      inp.value.toUpperCase().split('').forEach(function (c) {
        if (c === ' ') { var sp = document.createElement('span'); sp.style.width = '24px'; out.appendChild(sp); return; }
        var p = MORSE[c];
        if (!p) return;
        var el = document.createElement('span');
        el.className = 'ch';
        el.dataset.c = c;
        var pat = document.createElement('span');
        pat.className = 'pat';
        p.split('').forEach(function (e) { var s = document.createElement('i'); s.className = e === '.' ? 'dit' : 'dah'; pat.appendChild(s); });
        var lab = document.createElement('small');
        lab.textContent = c;
        el.appendChild(pat); el.appendChild(lab);
        out.appendChild(el);
      });
    };
    var stop = function () {
      timers.forEach(clearTimeout); timers = [];
      if (osc) { try { osc.stop(); } catch (e) {} osc = null; }
      btn.textContent = playLabel;
      out.querySelectorAll('.on').forEach(function (e) { e.classList.remove('on'); });
    };
    var play = function () {
      if (!AC) return;
      if (osc) { stop(); return; }
      actx = actx || new AC();
      var wpm = +wpmEl.value, unit = 1.2 / wpm;
      var g = actx.createGain(); g.gain.value = 0; g.connect(actx.destination);
      osc = actx.createOscillator(); osc.frequency.value = 650; osc.connect(g);
      var t = actx.currentTime + 0.08;
      var start = t;
      var chars = inp.value.toUpperCase().split('');
      chars.forEach(function (c) {
        if (c === ' ') { t += unit * 4; return; }
        var p = MORSE[c]; if (!p) return;
        var ct = t;
        timers.push(setTimeout((function (ch, tt) { return function () { out.querySelectorAll('.ch').forEach(function (e) { e.classList.remove('on'); }); }; })(c, ct), (ct - actx.currentTime) * 1000));
        p.split('').forEach(function (e) {
          var d = e === '.' ? unit : unit * 3;
          g.gain.setTargetAtTime(0.22, t, 0.004); g.gain.setTargetAtTime(0, t + d, 0.004);
          t += d + unit;
        });
        t += unit * 2;
      });
      osc.start(start);
      osc.stop(t + 0.2);
      btn.textContent = stopLabel;
      osc.onended = function () { osc = null; btn.textContent = playLabel; };
    };
    inp.addEventListener('input', function () { stop(); render(); });
    wpmEl.addEventListener('input', function () { wpmOut.textContent = wpmEl.value; });
    btn.addEventListener('click', play);
    wpmOut.textContent = wpmEl.value;
    render();
  }

  // ---------- Latest release ----------
  var dl = document.querySelectorAll('[data-dl]');
  if (dl.length && window.fetch) {
    var fmt = function (n) { return (n / 1048576).toFixed(1) + ' MB'; };
    fetch('https://api.github.com/repos/' + REPO + '/releases/latest', { headers: { Accept: 'application/vnd.github+json' } })
      .then(function (r) { if (!r.ok) throw new Error(r.status); return r.json(); })
      .then(function (rel) {
        var a = (rel.assets || []).filter(function (x) { return /-setup\.exe$/i.test(x.name); })[0];
        var v = (rel.tag_name || '').replace(/^v/, '');
        var lang = document.documentElement.lang;
        var date = rel.published_at ? new Date(rel.published_at).toLocaleDateString(lang === 'nl' ? 'nl-BE' : 'en-GB', { year: 'numeric', month: 'long', day: 'numeric' }) : '';
        document.querySelectorAll('[data-version]').forEach(function (e) { e.textContent = v; });
        document.querySelectorAll('[data-date]').forEach(function (e) { e.textContent = date; });
        document.querySelectorAll('[data-notes]').forEach(function (e) { e.href = rel.html_url; });
        if (a) {
          document.querySelectorAll('[data-file]').forEach(function (e) { e.textContent = a.name; });
          document.querySelectorAll('[data-size]').forEach(function (e) { e.textContent = fmt(a.size); });
          document.querySelectorAll('a[data-dl]').forEach(function (e) { e.href = a.browser_download_url; });
          var hash = (a.digest || '').replace(/^sha256:/, '');
          document.querySelectorAll('[data-hash]').forEach(function (e) {
            e.textContent = hash || e.getAttribute('data-nohash');
          });
          document.querySelectorAll('[data-name]').forEach(function (e) { e.textContent = a.name; });
        }
        document.querySelectorAll('[data-live]').forEach(function (e) { e.hidden = false; });
      })
      .catch(function () { document.querySelectorAll('[data-fail]').forEach(function (e) { e.hidden = false; }); });
  }
})();
