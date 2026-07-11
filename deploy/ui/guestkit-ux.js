/* GuestKit UX layer — event-bus toasts, ⌘K command palette, dock magnify,
   activity log, skeletons. Vanilla, token-driven, reduced-motion aware.
   Decoupled from the app via window CustomEvents (gk:toast). */
(function () {
  'use strict';
  var reduce = matchMedia('(prefers-reduced-motion: reduce)').matches;
  var esc = function (s) { return (window.escapeHtml ? window.escapeHtml(String(s == null ? '' : s)) : String(s == null ? '' : s)); };
  function el(tag, cls, html) { var e = document.createElement(tag); if (cls) e.className = cls; if (html != null) e.innerHTML = html; return e; }
  function ready(fn) { if (document.readyState !== 'loading') fn(); else document.addEventListener('DOMContentLoaded', fn); }

  /* ── Activity log — resurrects feed() by providing #activityFeed ── */
  function mountActivityLog() {
    if (document.getElementById('activityFeed')) return;
    var wrap = el('div', 'gk-activity-wrap');
    wrap.innerHTML =
      '<button type="button" class="gk-activity-toggle" aria-label="Activity log">◰ <span id="gkActivityCount">0</span></button>' +
      '<div class="gk-activity-panel" hidden><header>Activity <button type="button" class="gk-activity-close" aria-label="Close">✕</button></header><ul id="activityFeed" class="gk-activity"></ul></div>';
    document.body.appendChild(wrap);
    var panel = wrap.querySelector('.gk-activity-panel');
    var ul = wrap.querySelector('#activityFeed');
    var count = wrap.querySelector('#gkActivityCount');
    wrap.querySelector('.gk-activity-toggle').addEventListener('click', function () { panel.hidden = !panel.hidden; });
    wrap.querySelector('.gk-activity-close').addEventListener('click', function () { panel.hidden = true; });
    // keep the counter live
    new MutationObserver(function () { count.textContent = ul.children.length; }).observe(ul, { childList: true });
  }

  /* ── Rich toast bus ── */
  var ICONS = { ok: '✓', success: '✓', err: '✕', error: '✕', warn: '⚠', info: 'ℹ' };
  var recentToast = {};
  function mountToasts() {
    var stack = document.getElementById('toastStack');
    if (!stack) { stack = el('div', 'toast-stack'); stack.id = 'toastStack'; stack.setAttribute('aria-live', 'polite'); document.body.appendChild(stack); }
    window.addEventListener('gk:toast', function (ev) {
      var d = ev.detail || {}; var type = d.type || 'ok'; var msg = d.msg || '';
      var key = type + '|' + msg; var now = ev.timeStamp || performance.now();
      if (recentToast[key] && now - recentToast[key] < 3000) return; // dedup
      recentToast[key] = now;
      var t = el('div', 'toast toast--rich ' + type);
      t.innerHTML = '<span class="toast__ic">' + (ICONS[type] || 'ℹ') + '</span>' +
        '<span class="toast__msg">' + esc(msg) + '</span>' +
        (d.action ? '<button type="button" class="toast__act">' + esc(d.action.label) + '</button>' : '') +
        '<button type="button" class="toast__x" aria-label="Dismiss">✕</button>';
      stack.appendChild(t);
      while (stack.children.length > 5) stack.firstChild.remove();
      var kill = function () { t.classList.add('toast--out'); setTimeout(function () { t.remove(); }, 220); };
      t.querySelector('.toast__x').addEventListener('click', kill);
      if (d.action) t.querySelector('.toast__act').addEventListener('click', function () { try { d.action.run(); } catch (e) {} kill(); });
      setTimeout(kill, type === 'err' || type === 'error' ? 6500 : 4200);
    });
  }
  // public helper for other scripts
  window.gkToast = function (msg, type, action) { window.dispatchEvent(new CustomEvent('gk:toast', { detail: { msg: msg, type: type || 'ok', action: action } })); };

  /* ── Dock magnify + ripple ── */
  function mountDock() {
    var dock = document.getElementById('commandDock');
    if (!dock) return;
    dock.classList.remove('dock-hidden'); // reveal for discoverability
    var inner = dock.querySelector('.mac-dock-inner') || dock;
    var items = function () { return inner.querySelectorAll('.mac-dock-item'); };
    if (!reduce) {
      dock.addEventListener('mousemove', function (e) {
        items().forEach(function (it) {
          var r = it.getBoundingClientRect();
          var dist = Math.abs(e.clientX - (r.left + r.width / 2));
          var scale = Math.max(1, 1.65 - dist / 78);
          it.style.setProperty('--dock-scale', scale.toFixed(3));
        });
      });
      dock.addEventListener('mouseleave', function () { items().forEach(function (it) { it.style.setProperty('--dock-scale', 1); }); });
    }
    inner.addEventListener('click', function (e) {
      var it = e.target.closest('.mac-dock-item'); if (!it || reduce) return;
      var r = it.getBoundingClientRect();
      var rip = el('span', 'mac-dock-ripple');
      rip.style.left = (e.clientX - r.left) + 'px'; rip.style.top = (e.clientY - r.top) + 'px';
      it.appendChild(rip); rip.addEventListener('animationend', function () { rip.remove(); });
    });
    // tooltip on disabled items
    items().forEach(function (it) { if (it.disabled) it.title = 'Select a disk first'; });
  }

  /* ── Command palette (⌘K) ── */
  function fuzzy(q, text) {
    q = q.toLowerCase(); text = (text || '').toLowerCase();
    if (!q) return 1;
    if (text.startsWith(q)) return 100;
    var idx = text.indexOf(q); if (idx >= 0) return 80 - idx * 0.5;
    var ti = 0, run = 0, best = 0;
    for (var qi = 0; qi < q.length; qi++) {
      var f = text.indexOf(q[qi], ti);
      if (f < 0) return 0;
      run = f === ti ? run + 1 : 1; best = Math.max(best, run); ti = f + 1;
    }
    return 45 + best * 5;
  }
  var THEMES = ['carbon', 'phosphor', 'solaris', 'abyss'];
  var RECENT_KEY = 'gk.palette.recent';
  function recents() { try { return JSON.parse(localStorage.getItem(RECENT_KEY) || '[]'); } catch (e) { return []; } }
  function pushRecent(id) { try { var r = recents().filter(function (x) { return x !== id; }); r.unshift(id); localStorage.setItem(RECENT_KEY, JSON.stringify(r.slice(0, 6))); } catch (e) {} }

  function buildCommands() {
    var cmds = [];
    var st = window.state || {};
    // Actions on the selected disk
    var actions = [['analyze', '⚡ Analyze (full)', function () { window.runFullAnalysis && window.runFullAnalysis(); }],
      ['inspect', 'Fingerprint', function () { window.runAction && window.runAction('inspect'); }],
      ['doctor', 'Boot Doctor', function () { window.runAction && window.runAction('doctor'); }],
      ['migration-plan', 'Migration Plan', function () { window.runAction && window.runAction('migration-plan'); }],
      ['provision', 'Create VM', function () { window.runAction && window.runAction('provision'); }],
      ['convert', 'Convert Disk', function () { window.GuestKitConsole && window.GuestKitConsole.showConvertStudio && window.GuestKitConsole.showConvertStudio(); }]];
    actions.forEach(function (a) { cmds.push({ id: 'act:' + a[0], cat: 'Action', label: a[1], hint: st.selectedVm ? st.selectedVm.name : 'select a disk', run: a[2] }); });
    // Disks
    (st.vms || []).forEach(function (v) {
      cmds.push({ id: 'disk:' + v.id, cat: 'Disk', label: v.name || v.id, hint: (v.format || '') + ' · ' + (window.fmtBytes ? window.fmtBytes(v.size_bytes) : ''), run: function () { window.selectVm && window.selectVm(v); } });
    });
    // Themes
    THEMES.forEach(function (t) { cmds.push({ id: 'theme:' + t, cat: 'Theme', label: 'Theme · ' + t.charAt(0).toUpperCase() + t.slice(1), hint: 'switch theme', run: function () { document.documentElement.dataset.theme = t; try { localStorage.setItem('zyvor.theme', t); } catch (e) {} } }); });
    // Ask Zeus + views
    cmds.push({ id: 'zeus', cat: 'Ask Zeus', label: '⚡ Ask Zeus', hint: 'open assistant', run: function () { var b = document.getElementById('brainDrawerToggle'); b && b.click(); } });
    cmds.push({ id: 'view:vault', cat: 'View', label: 'Browse Server Vault', hint: '', run: function () { var b = document.getElementById('nebulaBrowseVault'); b && b.click(); } });
    cmds.push({ id: 'view:refresh', cat: 'View', label: 'Refresh fleet', hint: '', run: function () { window.loadFleet && window.loadFleet(); } });
    cmds.push({ id: 'share', cat: 'Export', label: '📸 Share verdict card', hint: st.selectedVm ? 'download PNG' : 'analyze a disk first', run: shareVerdict });
    cmds.push({ id: 'sound', cat: 'System', label: (soundOn() ? '🔊 Sound: on' : '🔇 Sound: off') + ' — toggle', hint: 'audio cues', run: toggleSound });
    cmds.push({ id: 'tour', cat: 'System', label: '🧭 Take the tour', hint: 'guided walkthrough', run: function () { runTour(true); } });
    cmds.push({ id: 'compare', cat: 'View', label: '⚖ Compare disks', hint: 'side-by-side diff', run: function () { window.gkCompare && window.gkCompare(); } });
    cmds.push({ id: 'trend', cat: 'View', label: '📈 Boot-score trend', hint: st.selectedVm ? 'history sparkline' : 'analyze a disk first', run: function () { window.gkScoreTrend && window.gkScoreTrend(); } });
    return cmds;
  }

  var pal, palInput, palList, palCmds = [], palSel = 0, palFiltered = [];
  function mountPalette() {
    pal = el('div', 'gk-pal', '');
    pal.setAttribute('role', 'dialog'); pal.setAttribute('aria-modal', 'true'); pal.hidden = true;
    pal.innerHTML =
      '<div class="gk-pal__box">' +
      '<input class="gk-pal__in" id="gkPalInput" placeholder="Search disks, actions, themes — type &gt; to Ask Zeus…" autocomplete="off" spellcheck="false" />' +
      '<div class="gk-pal__list" id="gkPalList" role="listbox"></div>' +
      '<footer class="gk-pal__foot"><kbd>↑</kbd><kbd>↓</kbd> navigate <kbd>↵</kbd> run <kbd>esc</kbd> close</footer>' +
      '</div>';
    document.body.appendChild(pal);
    palInput = pal.querySelector('#gkPalInput'); palList = pal.querySelector('#gkPalList');
    pal.addEventListener('mousedown', function (e) { if (e.target === pal) closePal(); });
    palInput.addEventListener('input', renderPal);
    palInput.addEventListener('keydown', function (e) {
      if (e.key === 'ArrowDown') { e.preventDefault(); palSel = Math.min(palSel + 1, palFiltered.length - 1); renderSel(); }
      else if (e.key === 'ArrowUp') { e.preventDefault(); palSel = Math.max(palSel - 1, 0); renderSel(); }
      else if (e.key === 'Enter') { e.preventDefault(); activate(palFiltered[palSel]); }
      else if (e.key === 'Escape') { closePal(); }
    });
  }
  function openPal() {
    if (!pal) mountPalette();
    palCmds = buildCommands(); palSel = 0; palInput.value = '';
    pal.hidden = false; renderPal(); palInput.focus(); cue('tick');
  }
  function closePal() { if (pal) pal.hidden = true; }
  function renderPal() {
    var q = palInput.value.trim();
    if (q.charAt(0) === '>') {
      // Ask-Zeus mode
      var question = q.slice(1).trim();
      palFiltered = [{ id: 'ask', cat: 'Ask Zeus', label: question ? '⚡ Ask: ' + question : '⚡ Type a question for Zeus…', hint: question ? 'send to assistant' : '', run: function () { question && askZeus(question); } }];
      palSel = 0; paintRows(); return;
    }
    if (!q) {
      var rec = recents(); var byId = {}; palCmds.forEach(function (c) { byId[c.id] = c; });
      palFiltered = rec.map(function (id) { return byId[id]; }).filter(Boolean).slice(0, 5);
      if (!palFiltered.length) palFiltered = palCmds.filter(function (c) { return c.cat === 'Action'; });
    } else {
      palFiltered = palCmds.map(function (c) { return { c: c, s: fuzzy(q, c.label + ' ' + c.cat) }; })
        .filter(function (x) { return x.s > 0; }).sort(function (a, b) { return b.s - a.s; }).slice(0, 12).map(function (x) { return x.c; });
    }
    palSel = 0;
    paintRows();
  }
  function paintRows() {
    palList.innerHTML = palFiltered.length ? palFiltered.map(function (c, i) {
      return '<div class="gk-pal__row' + (i === palSel ? ' sel' : '') + '" data-i="' + i + '" role="option"><span class="gk-pal__cat">' + esc(c.cat) + '</span><span class="gk-pal__label">' + esc(c.label) + '</span><span class="gk-pal__hint">' + esc(c.hint || '') + '</span></div>';
    }).join('') : '<div class="gk-pal__empty">No matches</div>';
    palList.querySelectorAll('.gk-pal__row').forEach(function (r) {
      r.addEventListener('mouseenter', function () { palSel = +r.dataset.i; renderSel(); });
      r.addEventListener('click', function () { activate(palFiltered[+r.dataset.i]); });
    });
  }
  function renderSel() {
    palList.querySelectorAll('.gk-pal__row').forEach(function (r, i) { r.classList.toggle('sel', i === palSel); if (i === palSel) r.scrollIntoView({ block: 'nearest' }); });
  }
  function activate(c) { if (!c) return; closePal(); pushRecent(c.id); try { c.run(); } catch (e) { window.gkToast('Action failed: ' + (e.message || e), 'err'); } }

  /* ── keys ── */
  function isTyping(e) { var t = e.target; return t && (/^(INPUT|TEXTAREA|SELECT)$/.test(t.tagName) || t.isContentEditable); }
  document.addEventListener('keydown', function (e) {
    if ((e.metaKey || e.ctrlKey) && (e.key === 'k' || e.key === 'K')) { e.preventDefault(); pal && !pal.hidden ? closePal() : openPal(); }
  });

  /* ── Ambient aurora — drifting accent glow behind the shell ── */
  function mountAurora() {
    if (reduce || document.getElementById('gkAurora')) return;
    var a = el('div', 'gk-aurora'); a.id = 'gkAurora';
    a.innerHTML = '<span class="gk-aurora__b b1"></span><span class="gk-aurora__b b2"></span><span class="gk-aurora__b b3"></span>';
    document.body.insertBefore(a, document.body.firstChild);
  }

  /* ── Theme-switch radial wipe ── */
  function mountThemeWipe() {
    if (reduce) return;
    var last = document.documentElement.dataset.theme;
    new MutationObserver(function () {
      var t = document.documentElement.dataset.theme;
      if (t === last) return; last = t;
      var w = el('div', 'gk-wipe'); document.body.appendChild(w);
      // paint with the NEW accent
      requestAnimationFrame(function () {
        w.style.background = 'radial-gradient(circle at 50% 42%, ' + accent() + ' 0%, transparent 60%)';
        w.classList.add('go');
      });
      setTimeout(function () { w.remove(); }, 720);
      window.gkToast('Theme · ' + (t ? t[0].toUpperCase() + t.slice(1) : ''), 'info');
    }).observe(document.documentElement, { attributes: true, attributeFilter: ['data-theme'] });
  }
  function accent() { try { return (getComputedStyle(document.documentElement).getPropertyValue('--accent') || '#7cf').trim(); } catch (e) { return '#7cf'; } }

  /* ── Cinematic Zeus scan overlay (gk:analyze driven) ── */
  function mountScan() {
    var ov = null, phaseEls = [], dismissTimer = null;
    function build(steps) {
      teardown();
      ov = el('div', 'gk-scan');
      var phaseHtml = (steps || []).map(function (s, i) {
        return '<li class="gk-scan__ph" data-i="' + i + '"><span class="gk-scan__dot"></span><span>' + esc(s) + '</span></li>';
      }).join('');
      ov.innerHTML =
        '<div class="gk-scan__core">' +
          '<div class="gk-scan__ring"><svg viewBox="0 0 120 120"><circle class="gk-scan__trk" cx="60" cy="60" r="52"/><circle class="gk-scan__arc" cx="60" cy="60" r="52"/></svg>' +
            '<div class="gk-scan__bolt">⚡</div><div class="gk-scan__num" hidden><b>0</b><small>boot</small></div></div>' +
          '<div class="gk-scan__title">Zeus is scanning<span class="gk-scan__dots">…</span></div>' +
          '<ul class="gk-scan__phases">' + phaseHtml + '</ul>' +
          '<div class="gk-scan__verdict" hidden></div>' +
        '</div><div class="gk-scan__grid"></div>';
      document.body.appendChild(ov);
      phaseEls = [].slice.call(ov.querySelectorAll('.gk-scan__ph'));
      requestAnimationFrame(function () { ov.classList.add('go'); });
      ov.addEventListener('click', function (e) { if (e.target === ov) dismiss(); });
    }
    function markStep(i) {
      phaseEls.forEach(function (p, j) {
        p.classList.toggle('done', j < i - 1);
        p.classList.toggle('active', j === i - 1);
      });
      var arc = ov && ov.querySelector('.gk-scan__arc');
      if (arc) { var frac = Math.max(0.06, (i - 0.4) / Math.max(1, phaseEls.length)); arc.style.strokeDashoffset = String(327 * (1 - frac)); }
    }
    function finish(ok, score) {
      if (!ov) return;
      phaseEls.forEach(function (p) { p.classList.remove('active'); p.classList.add('done'); });
      var arc = ov.querySelector('.gk-scan__arc'); if (arc) arc.style.strokeDashoffset = '0';
      var s = score == null ? null : Math.round(score);
      var tone = !ok ? 'err' : (s != null && s >= 90 ? 'ok' : (s != null && s >= 60 ? 'warn' : (s == null ? 'ok' : 'err')));
      ov.classList.add('done', tone);
      ov.querySelector('.gk-scan__bolt').hidden = true;
      var numWrap = ov.querySelector('.gk-scan__num');
      var vEl = ov.querySelector('.gk-scan__verdict');
      var titleEl = ov.querySelector('.gk-scan__title');
      if (s != null) {
        numWrap.hidden = false;
        countUp(numWrap.querySelector('b'), s, 900);
        var line = s >= 90 ? 'Boot-ready — ship it' : s >= 60 ? 'Bootable with fixes' : 'Blocked — needs repair';
        titleEl.innerHTML = 'Verdict';
        vEl.hidden = false; vEl.textContent = line;
      } else {
        titleEl.textContent = ok ? 'Analysis complete' : 'Analysis stopped';
      }
      if (ok && (s == null || s >= 60)) burst(tone === 'ok');
      dismissTimer = setTimeout(dismiss, s != null && s >= 90 ? 2600 : 3200);
    }
    function dismiss() { if (!ov) return; ov.classList.add('out'); var o = ov; ov = null; setTimeout(function () { o.remove(); }, 380); }
    function teardown() { if (dismissTimer) { clearTimeout(dismissTimer); dismissTimer = null; } if (ov) { ov.remove(); ov = null; } phaseEls = []; }
    window.addEventListener('gk:analyze', function (ev) {
      var d = ev.detail || {};
      if (reduce) return; // respect reduced motion — banner + toasts still cover it
      if (d.phase === 'start') build(d.steps);
      else if (d.phase === 'step') { if (!ov) build(null); markStep(d.i); }
      else if (d.phase === 'done') finish(d.ok !== false, d.score);
      else if (d.phase === 'error') finish(false, null);
    });
  }
  function countUp(node, to, ms) {
    if (!node) return; var start = null, from = 0;
    function tick(ts) { if (start == null) start = ts; var p = Math.min(1, (ts - start) / ms); node.textContent = Math.round(from + (to - from) * (1 - Math.pow(1 - p, 3))); if (p < 1) requestAnimationFrame(tick); }
    requestAnimationFrame(tick);
  }
  /* lightning + spark burst */
  function burst(good) {
    if (reduce) return;
    var flash = el('div', 'gk-flash' + (good ? ' ok' : ' warn')); document.body.appendChild(flash);
    setTimeout(function () { flash.remove(); }, 620);
    var wrap = el('div', 'gk-sparks'); document.body.appendChild(wrap);
    var glyphs = good ? ['⚡', '✦', '✧', '⚡'] : ['⚠', '✦', '⚡'];
    for (var i = 0; i < 18; i++) {
      var s = el('span', 'gk-spark'); s.textContent = glyphs[i % glyphs.length];
      var ang = (i / 18) * Math.PI * 2, dist = 120 + (i % 5) * 46;
      s.style.setProperty('--dx', Math.cos(ang) * dist + 'px');
      s.style.setProperty('--dy', Math.sin(ang) * dist + 'px');
      s.style.animationDelay = (i % 6) * 18 + 'ms';
      wrap.appendChild(s);
    }
    setTimeout(function () { wrap.remove(); }, 1300);
  }

  /* ── Konami → Storm mode easter egg ── */
  function mountStorm() {
    var seq = ['ArrowUp', 'ArrowUp', 'ArrowDown', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'ArrowLeft', 'ArrowRight', 'b', 'a'];
    var pos = 0, on = false, timer = null;
    document.addEventListener('keydown', function (e) {
      var k = e.key.length === 1 ? e.key.toLowerCase() : e.key;
      pos = (k === seq[pos]) ? pos + 1 : (k === seq[0] ? 1 : 0);
      if (pos === seq.length) { pos = 0; toggle(); }
    });
    function toggle() {
      on = !on;
      if (on) {
        document.body.classList.add('gk-storm-on');
        window.gkToast('⚡ Storm mode engaged — Zeus is pleased', 'ok');
        strike();
        timer = setInterval(function () { if (!reduce) strike(); }, 3400);
      } else {
        clearInterval(timer); document.body.classList.remove('gk-storm-on');
        window.gkToast('Storm mode off', 'info');
      }
    }
    function strike() {
      var f = el('div', 'gk-storm-flash'); document.body.appendChild(f);
      setTimeout(function () { f.remove(); }, 500);
      cue('rumble');
    }
    window.gkStorm = toggle;
  }

  /* ── Shortcut cheat sheet (declarative registry → accurate, styled) ── */
  var THEMES = ['carbon', 'phosphor', 'solaris', 'abyss'];
  function setTheme(t) {
    try { document.documentElement.dataset.theme = t; localStorage.setItem('zyvor.theme', t); } catch (e) { document.documentElement.dataset.theme = t; }
  }
  function cycleTheme() {
    var cur = document.documentElement.dataset.theme || 'phosphor';
    setTheme(THEMES[(THEMES.indexOf(cur) + 1) % THEMES.length]);
  }
  var SHORTCUTS = [
    { sec: 'Analysis', rows: [
      ['I', 'Fingerprint (inspect)'], ['D', 'Boot Doctor'], ['M', 'Migration plan'],
      ['R', 'Repair plan'], ['Q', 'Quick scan · inspect + doctor'], ['L', 'Launch preview'],
    ] },
    { sec: 'Navigation', rows: [
      ['← →', 'Move between disks'], ['Enter', 'Analyze focused disk'], ['/', 'Search disks'],
      ['U', 'Upload a disk'], ['E', 'Evidence console'], ['C', 'Compare mode'],
    ] },
    { sec: 'Interface', rows: [
      ['⌘K', 'Command palette'], ['B', 'Toggle Ask Zeus'], ['T', 'Cycle theme'],
      ['?', 'This cheat sheet'], ['Esc', 'Close · clear selection'],
    ] },
  ];
  function kbdify(keys) {
    return keys.split(' ').map(function (k) { return '<kbd class="gk-kbd">' + esc(k) + '</kbd>'; }).join(' ');
  }
  function mountShortcuts() {
    var modal = document.getElementById('shortcutsModal');
    if (!modal) return;
    var list = modal.querySelector('.shortcuts-list');
    if (!list) return;
    // Replace stale hardcoded rows with the accurate declarative sheet.
    list.classList.add('gk-sheet');
    list.innerHTML = SHORTCUTS.map(function (g) {
      return '<li class="gk-sheet__sec">' + esc(g.sec) + '</li>' + g.rows.map(function (r) {
        return '<li class="gk-sheet__row"><span class="gk-sheet__lbl">' + esc(r[1]) + '</span><span class="gk-sheet__k">' + kbdify(r[0]) + '</span></li>';
      }).join('');
    }).join('') + '<li class="gk-sheet__foot">Tip: try the Konami code ↑↑↓↓←→←→ B A</li>';
    // Theme-cycle keybind (not handled elsewhere).
    document.addEventListener('keydown', function (e) {
      if (isTyping(e) || e.metaKey || e.ctrlKey || e.altKey) return;
      if (e.key === 't' || e.key === 'T') { e.preventDefault(); cycleTheme(); }
    });
  }

  /* ── Skeleton loaders for the fleet grid ── */
  function mountSkeletons() {
    var grid = document.getElementById('fleetGrid');
    if (!grid) return;
    function hasReal() { return !!grid.querySelector('.vm-card'); }
    function emptyShown() { var e = document.getElementById('fleetEmpty'); return e && !e.classList.contains('hidden'); }
    function clearSkel() { grid.querySelectorAll('.gk-skel-card').forEach(function (n) { n.remove(); }); }
    function showSkel() {
      if (hasReal() || emptyShown() || grid.querySelector('.gk-skel-card')) return;
      var frag = document.createDocumentFragment();
      for (var i = 0; i < 3; i++) {
        var c = el('div', 'gk-skel-card');
        c.innerHTML = '<div class="gk-skel" style="height:14px;width:62%"></div>' +
          '<div class="gk-skel" style="height:10px;width:40%;margin-top:9px"></div>' +
          '<div class="gk-skel-row"><span class="gk-skel" style="height:22px;width:46px"></span><span class="gk-skel" style="height:22px;width:46px"></span><span class="gk-skel" style="height:22px;width:46px"></span></div>';
        frag.appendChild(c);
      }
      grid.appendChild(frag);
    }
    showSkel();
    new MutationObserver(function () { if (hasReal() || emptyShown()) clearSkel(); }).observe(grid, { childList: true });
    // Safety: clear after 12s even if nothing rendered.
    setTimeout(clearSkel, 12000);
  }

  /* ── Arrow-key fleet navigation ── */
  function mountFleetNav() {
    var focusI = -1;
    function cards() { return [].slice.call(document.querySelectorAll('#fleetGrid .vm-card')); }
    function paint(list) { list.forEach(function (c, i) { c.classList.toggle('gk-kbd-focus', i === focusI); }); if (list[focusI]) list[focusI].scrollIntoView({ block: 'nearest' }); }
    document.addEventListener('keydown', function (e) {
      if (isTyping(e) || e.metaKey || e.ctrlKey) return;
      // Don't hijack keys while a modal surface (tour / palette / compare) is open.
      if (document.querySelector('.gk-tour') || (pal && !pal.hidden) || document.querySelector('.gk-cmp:not([hidden])')) return;
      var list = cards(); if (!list.length) return;
      if (e.key === 'ArrowRight' || e.key === 'ArrowDown') { e.preventDefault(); focusI = (focusI + 1) % list.length; paint(list); }
      else if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') { e.preventDefault(); focusI = (focusI - 1 + list.length) % list.length; paint(list); }
      else if (e.key === 'Enter' && focusI >= 0 && list[focusI]) { e.preventDefault(); list[focusI].click(); setTimeout(function () { window.runFullAnalysis && window.runFullAnalysis(); }, 120); }
    });
  }

  /* ── Click-to-copy on report mono/UUID cells ── */
  function mountCopy() {
    document.addEventListener('click', function (e) {
      var t = e.target.closest ? e.target.closest('.ir-mono') : null;
      if (!t) return;
      var txt = (t.textContent || '').trim();
      if (!txt || txt === '—') return;
      var done = function () { window.gkToast('Copied · ' + (txt.length > 28 ? txt.slice(0, 28) + '…' : txt), 'ok'); t.classList.add('gk-copied'); setTimeout(function () { t.classList.remove('gk-copied'); }, 700); };
      try { navigator.clipboard.writeText(txt).then(done, function () {}); } catch (err) {}
    });
    // Hint affordance
    var style = el('style'); style.textContent = '.ir-mono{cursor:copy}'; document.head.appendChild(style);
  }

  /* ── Ask Zeus starter chips on empty chat ── */
  var STARTERS = ['Is this disk boot-ready?', 'What blocks migration to KubeVirt?', 'Explain the boot score', 'What should I fix first?'];
  function mountStarterChips() {
    var chat = document.getElementById('brainAskChat');
    var form = document.getElementById('brainAskForm');
    var input = document.getElementById('brainAskInput');
    if (!chat || !form || !input) return;
    function sync() {
      var empty = !chat.querySelector('.brain-chat-msg');
      var existing = chat.parentNode.querySelector('.gk-starters');
      if (empty && !existing) {
        var bar = el('div', 'gk-starters');
        bar.innerHTML = STARTERS.map(function (p) { return '<button type="button" class="gk-starter">' + esc(p) + '</button>'; }).join('');
        bar.querySelectorAll('.gk-starter').forEach(function (b) {
          b.addEventListener('click', function () {
            input.value = b.textContent;
            if (form.requestSubmit) form.requestSubmit(); else form.dispatchEvent(new Event('submit', { cancelable: true, bubbles: true }));
          });
        });
        chat.parentNode.insertBefore(bar, chat.nextSibling);
      } else if (!empty && existing) { existing.remove(); }
    }
    sync();
    new MutationObserver(sync).observe(chat, { childList: true });
  }

  /* ── Ask Zeus from anywhere (fills + submits the chat) ── */
  function askZeus(text) {
    var toggle = document.getElementById('brainDrawerToggle');
    var drawer = document.getElementById('brainDrawer');
    if (drawer && !drawer.classList.contains('open') && toggle) toggle.click();
    setTimeout(function () {
      var input = document.getElementById('brainAskInput');
      var form = document.getElementById('brainAskForm');
      if (!input || !form) { window.gkToast('Ask Zeus panel unavailable', 'warn'); return; }
      input.value = text;
      if (form.requestSubmit) form.requestSubmit(); else form.dispatchEvent(new Event('submit', { cancelable: true, bubbles: true }));
      input.focus();
    }, 160);
  }

  /* ── Synthesized Web Audio cues (no assets, CSP-safe) ── */
  var actx = null;
  function soundOn() { try { var v = localStorage.getItem('gk.sound'); return v == null ? !reduce : v === '1'; } catch (e) { return false; } }
  function toggleSound() {
    var next = !soundOn();
    try { localStorage.setItem('gk.sound', next ? '1' : '0'); } catch (e) {}
    if (next) { cue('chord'); window.gkToast('🔊 Sound cues on', 'ok'); } else { window.gkToast('🔇 Sound cues muted', 'info'); }
  }
  function ac() { if (!actx) { try { actx = new (window.AudioContext || window.webkitAudioContext)(); } catch (e) { actx = false; } } if (actx && actx.state === 'suspended') { try { actx.resume(); } catch (e) {} } return actx || null; }
  function note(freq, start, dur, type, peak) {
    var a = ac(); if (!a) return;
    var t0 = a.currentTime + start;
    var o = a.createOscillator(), g = a.createGain();
    o.type = type || 'sine'; o.frequency.value = freq;
    g.gain.setValueAtTime(0.0001, t0);
    g.gain.exponentialRampToValueAtTime(peak || 0.12, t0 + 0.012);
    g.gain.exponentialRampToValueAtTime(0.0001, t0 + dur);
    o.connect(g); g.connect(a.destination);
    o.start(t0); o.stop(t0 + dur + 0.02);
  }
  function cue(kind) {
    if (!soundOn() || !ac()) return;
    if (kind === 'tick') { note(1180, 0, 0.05, 'square', 0.05); }
    else if (kind === 'chord') { [523.25, 659.25, 783.99, 1046.5].forEach(function (f, i) { note(f, i * 0.07, 0.5, 'triangle', 0.09); }); }
    else if (kind === 'warn') { note(392, 0, 0.28, 'triangle', 0.1); note(311, 0.1, 0.32, 'triangle', 0.09); }
    else if (kind === 'bad') { note(160, 0, 0.42, 'sawtooth', 0.11); }
    else if (kind === 'rumble') { note(58, 0, 0.5, 'sine', 0.16); note(41, 0.04, 0.55, 'sine', 0.12); }
  }
  function mountAudioCues() {
    window.addEventListener('gk:analyze', function (ev) {
      var d = ev.detail || {};
      if (d.phase === 'done') { var s = d.score; cue(d.ok === false ? 'bad' : (s != null && s >= 90 ? 'chord' : (s != null && s < 60 ? 'bad' : 'warn'))); }
      else if (d.phase === 'error') { cue('bad'); }
    });
    window.gkCue = cue;
  }

  /* ── Zeus verdict share-card (canvas → PNG) ── */
  function tok(name, fb) { try { var v = getComputedStyle(document.documentElement).getPropertyValue(name).trim(); return v || fb; } catch (e) { return fb; } }
  function shareVerdict() {
    var st = window.state || {};
    var vm = st.selectedVm;
    var cache = vm && window.getVmCache ? window.getVmCache(vm.id) : null;
    if (!vm || !cache || cache.bootScore == null) { window.gkToast('Run ⚡ Analyze on a disk first', 'warn'); return; }
    var ins = (cache.inspect && cache.inspect.operating_system) || {};
    var mp = cache.migrationPlan || {};
    var score = Math.round(cache.bootScore);
    var blockers = (cache.blockers || []).length;
    var checks = cache.checks || [];
    var passed = checks.filter(function (c) { return c.passed; }).length;
    var mig = mp.migration_score && mp.migration_score.score != null ? Math.round(mp.migration_score.score) : null;
    var accent = tok('--accent', '#7cc7ff');
    var good = tok('--success', '#39d98a'), warn = tok('--warn', '#f5a623'), bad = tok('--danger', '#ff5c5c');
    var ink = tok('--text-main', '#eaf2ff'), soft = tok('--text-soft', '#8aa0c0');
    var scoreCol = score >= 90 ? good : score >= 60 ? warn : bad;
    var W = 1200, H = 630, s = 2;
    var cv = document.createElement('canvas'); cv.width = W * s; cv.height = H * s;
    var x = cv.getContext('2d'); x.scale(s, s);
    // background
    var bg = x.createLinearGradient(0, 0, W, H);
    bg.addColorStop(0, '#0a0f1a'); bg.addColorStop(1, '#111a2b');
    x.fillStyle = bg; x.fillRect(0, 0, W, H);
    // accent glow
    var gl = x.createRadialGradient(950, 150, 40, 950, 150, 420);
    gl.addColorStop(0, accent + '33'); gl.addColorStop(1, 'transparent');
    x.fillStyle = gl; x.fillRect(0, 0, W, H);
    // grid
    x.strokeStyle = accent + '14'; x.lineWidth = 1;
    for (var gx = 0; gx <= W; gx += 40) { x.beginPath(); x.moveTo(gx, 0); x.lineTo(gx, H); x.stroke(); }
    for (var gy = 0; gy <= H; gy += 40) { x.beginPath(); x.moveTo(0, gy); x.lineTo(W, gy); x.stroke(); }
    // brand
    x.fillStyle = accent; x.font = '700 34px system-ui,sans-serif'; x.fillText('⚡ Zeus AI', 64, 92);
    x.fillStyle = soft; x.font = '400 18px ui-monospace,monospace'; x.fillText('OFFLINE VM INTELLIGENCE · GUESTKIT', 64, 122);
    // ring
    var cx = 960, cy = 315, r = 150;
    x.lineWidth = 26; x.lineCap = 'round';
    x.strokeStyle = accent + '22'; x.beginPath(); x.arc(cx, cy, r, 0, Math.PI * 2); x.stroke();
    x.strokeStyle = scoreCol; x.beginPath(); x.arc(cx, cy, r, -Math.PI / 2, -Math.PI / 2 + Math.PI * 2 * (score / 100)); x.stroke();
    x.fillStyle = ink; x.textAlign = 'center'; x.font = '800 96px system-ui,sans-serif'; x.fillText(String(score), cx, cy + 20);
    x.fillStyle = soft; x.font = '600 22px ui-monospace,monospace'; x.fillText('BOOT SCORE', cx, cy + 66);
    x.textAlign = 'left';
    // verdict
    var verdict = blockers ? (blockers + ' blocker' + (blockers > 1 ? 's' : '') + ' to clear') : (mp.target ? 'Ready to migrate → ' + mp.target : 'Boot-ready');
    x.fillStyle = ink; x.font = '700 52px system-ui,sans-serif'; x.fillText(verdict, 64, 250);
    x.fillStyle = soft; x.font = '400 30px system-ui,sans-serif';
    x.fillText((ins.product_name || ins.distribution || vm.name || 'Disk') + (ins.arch ? '  ·  ' + ins.arch : ''), 64, 300);
    // stat chips
    var stats = [[String(passed) + '/' + checks.length, 'CHECKS'], [String(blockers), 'BLOCKERS'], [mig != null ? String(mig) : '—', 'MIGRATE']];
    var sx = 64;
    stats.forEach(function (p) {
      x.fillStyle = '#ffffff0d'; roundRect(x, sx, 360, 180, 96, 16); x.fill();
      x.fillStyle = ink; x.font = '800 46px system-ui,sans-serif'; x.fillText(p[0], sx + 22, 418);
      x.fillStyle = soft; x.font = '600 16px ui-monospace,monospace'; x.fillText(p[1], sx + 22, 444);
      sx += 200;
    });
    // footer
    x.fillStyle = soft; x.font = '400 18px ui-monospace,monospace';
    x.fillText('zyvor.dev/guestkit', 64, 566);
    // download
    cv.toBlob(function (blob) {
      if (!blob) { window.gkToast('Could not render card', 'err'); return; }
      var url = URL.createObjectURL(blob);
      var a = el('a'); a.href = url; a.download = 'zeus-verdict-' + (vm.name || vm.id).replace(/[^\w.-]+/g, '_') + '.png';
      document.body.appendChild(a); a.click(); a.remove();
      setTimeout(function () { URL.revokeObjectURL(url); }, 4000);
      window.gkToast('Verdict card downloaded', 'ok'); cue('chord');
    }, 'image/png');
  }
  function roundRect(x, px, py, w, h, r) { x.beginPath(); x.moveTo(px + r, py); x.arcTo(px + w, py, px + w, py + h, r); x.arcTo(px + w, py + h, px, py + h, r); x.arcTo(px, py + h, px, py, r); x.arcTo(px, py, px + w, py, r); x.closePath(); }
  window.gkShareVerdict = shareVerdict;

  /* ── Global drag-to-analyze drop overlay ── */
  function mountDropCatcher() {
    var ov = el('div', 'gk-drop');
    ov.innerHTML = '<div class="gk-drop__card"><div class="gk-drop__ic">💿</div><div class="gk-drop__t">Drop to fingerprint</div>' +
      '<div class="gk-drop__s">GuestKit analyzes the disk offline before you boot it</div>' +
      '<div class="gk-drop__chips">QCOW2 · VMDK · VHDX · RAW · OVA · ISO</div></div>';
    document.body.appendChild(ov);
    var depth = 0;
    function hasFiles(e) { var t = e.dataTransfer && e.dataTransfer.types; return t && (t.indexOf ? t.indexOf('Files') > -1 : t.contains && t.contains('Files')); }
    function show() { ov.classList.add('on'); } function hide() { depth = 0; ov.classList.remove('on'); }
    window.addEventListener('dragenter', function (e) { if (!hasFiles(e)) return; e.preventDefault(); depth++; show(); });
    window.addEventListener('dragover', function (e) { if (ov.classList.contains('on')) e.preventDefault(); });
    window.addEventListener('dragleave', function (e) { if (!ov.classList.contains('on')) return; depth--; if (depth <= 0) hide(); });
    window.addEventListener('drop', function (e) {
      if (!ov.classList.contains('on')) return;
      e.preventDefault(); hide();
      var f = e.dataTransfer && e.dataTransfer.files && e.dataTransfer.files[0];
      if (!f) return;
      if (window.uploadFile) { window.uploadFile(f); window.gkToast('Uploading ' + f.name + '…', 'info'); cue('tick'); }
      else { var inp = document.getElementById('fileInput'); if (inp) { try { var dt = new DataTransfer(); dt.items.add(f); inp.files = dt.files; inp.dispatchEvent(new Event('change', { bubbles: true })); } catch (err) {} } }
    });
    window.addEventListener('dragend', hide);
  }

  /* ── First-run coach-mark tour (spotlight) ── */
  var TOUR = [
    { sel: '#dropzone', title: 'Start here', body: 'Drop a VM disk — or drag one anywhere on the page. GuestKit fingerprints the OS, drivers, and boot config offline, before you ever launch it.' },
    { sel: '#brainDrawerToggle', title: 'Ask Zeus', body: 'Your offline copilot. Ask about boot risks, migration blockers, or launch readiness. Tip: press ⌘K and type “>” to ask from anywhere.' },
    { sel: '#themeToggle', title: 'Make it yours', body: 'Four themes — Carbon, Phosphor, Solaris, Abyss. Click here or press T to cycle. Everything restyles live.' },
    { sel: null, title: 'You’re set ⚡', body: 'Press ⌘K for the command palette, ? for all shortcuts, and ⚡ Analyze on any disk for the full report. Happy migrating.' },
  ];
  function runTour(force) {
    try { if (!force && localStorage.getItem('gk.onboarded') === '1') return; } catch (e) {}
    var i = 0;
    var back = el('div', 'gk-tour');
    var hole = el('div', 'gk-tour__hole');
    var card = el('div', 'gk-tour__card');
    back.appendChild(hole); back.appendChild(card); document.body.appendChild(back);
    function finish() { try { localStorage.setItem('gk.onboarded', '1'); } catch (e) {} back.remove(); window.removeEventListener('resize', place); window.removeEventListener('keydown', onKey); }
    function place() {
      var step = TOUR[i]; var t = step.sel ? document.querySelector(step.sel) : null;
      if (t && t.getBoundingClientRect) {
        var r = t.getBoundingClientRect(); var pad = 8;
        hole.style.display = 'block';
        hole.style.left = (r.left - pad) + 'px'; hole.style.top = (r.top - pad) + 'px';
        hole.style.width = (r.width + pad * 2) + 'px'; hole.style.height = (r.height + pad * 2) + 'px';
        // place card below or above depending on space
        var below = r.bottom + 180 < window.innerHeight;
        card.style.left = Math.max(16, Math.min(r.left, window.innerWidth - 360)) + 'px';
        card.style.top = (below ? r.bottom + pad + 12 : r.top - pad - 12) + 'px';
        card.style.transform = below ? 'none' : 'translateY(-100%)';
      } else {
        hole.style.display = 'none';
        card.style.left = '50%'; card.style.top = '50%'; card.style.transform = 'translate(-50%,-50%)';
      }
    }
    function render() {
      var step = TOUR[i]; var last = i === TOUR.length - 1;
      card.innerHTML = '<div class="gk-tour__step">' + (i + 1) + ' / ' + TOUR.length + '</div>' +
        '<h4>' + esc(step.title) + '</h4><p>' + esc(step.body) + '</p>' +
        '<div class="gk-tour__nav">' + (last ? '' : '<button type="button" class="gk-tour__skip">Skip</button>') +
        '<button type="button" class="gk-tour__next">' + (last ? 'Done' : 'Next →') + '</button></div>';
      card.querySelector('.gk-tour__next').addEventListener('click', function () { if (last) finish(); else { i++; step0(); } });
      var sk = card.querySelector('.gk-tour__skip'); if (sk) sk.addEventListener('click', finish);
      place();
    }
    function step0() { render(); }
    function onKey(e) { if (e.key === 'Escape') finish(); else if (e.key === 'Enter') { if (i === TOUR.length - 1) finish(); else { i++; render(); } } }
    window.addEventListener('resize', place); window.addEventListener('keydown', onKey);
    render(); cue('tick');
  }
  window.gkTour = function () { runTour(true); };

  /* ── Fleet compare (client-side, uses cached analysis) ── */
  function compareMetrics(vm) {
    var c = (vm && window.getVmCache) ? window.getVmCache(vm.id) : {};
    var ins = c.inspect || {}, os = ins.operating_system || {}, mp = c.migrationPlan || {};
    var checks = c.checks || [];
    return {
      'OS': os.product_name || os.distribution || '—',
      'Arch': os.arch || '—',
      'Boot score': c.bootScore != null ? Math.round(c.bootScore) : null,
      'Blockers': (c.blockers || []).length,
      'Checks passed': checks.length ? checks.filter(function (x) { return x.passed; }).length + '/' + checks.length : '—',
      'Migration score': mp.migration_score && mp.migration_score.score != null ? Math.round(mp.migration_score.score) : null,
      'Packages': ins.packages && ins.packages.count != null ? ins.packages.count : '—',
      'Kernels': ins.kernels && ins.kernels.count != null ? ins.kernels.count : '—',
      'Users': ins.users && ins.users.count != null ? ins.users.count : '—',
      'Guest tools': (ins.vm_tools && ins.vm_tools.detected || []).join(', ') || '—',
      'cloud-init': ins.cloud_init ? (ins.cloud_init.enabled ? 'enabled' : 'present') : '—',
    };
  }
  function mountCompare() {
    var ov = el('div', 'gk-cmp'); ov.hidden = true;
    ov.innerHTML = '<div class="gk-cmp__box"><header><h3>Compare disks</h3><button type="button" class="gk-cmp__x" aria-label="Close">✕</button></header>' +
      '<div class="gk-cmp__sel"><select class="gk-cmp__a"></select><span class="gk-cmp__vs">vs</span><select class="gk-cmp__b"></select></div>' +
      '<div class="gk-cmp__body"></div></div>';
    document.body.appendChild(ov);
    var selA = ov.querySelector('.gk-cmp__a'), selB = ov.querySelector('.gk-cmp__b'), body = ov.querySelector('.gk-cmp__body');
    ov.querySelector('.gk-cmp__x').addEventListener('click', function () { ov.hidden = true; });
    ov.addEventListener('mousedown', function (e) { if (e.target === ov) ov.hidden = true; });
    function render() {
      var vms = (window.state && window.state.vms) || [];
      var a = vms.find(function (v) { return v.id === selA.value; });
      var b = vms.find(function (v) { return v.id === selB.value; });
      if (!a || !b) { body.innerHTML = '<p class="gk-cmp__empty">Pick two disks.</p>'; return; }
      var ma = compareMetrics(a), mb = compareMetrics(b);
      var keys = Object.keys(ma);
      body.innerHTML = '<table class="gk-cmp__tbl"><thead><tr><th></th><th>' + esc(a.name || a.id) + '</th><th>' + esc(b.name || b.id) + '</th></tr></thead><tbody>' +
        keys.map(function (k) {
          var va = ma[k], vb = mb[k];
          var da = va == null ? '—' : va, db = vb == null ? '—' : vb;
          var diff = String(da) !== String(db);
          return '<tr class="' + (diff ? 'gk-cmp__diff' : '') + '"><td class="gk-cmp__k">' + esc(k) + '</td><td>' + esc(da) + '</td><td>' + esc(db) + '</td></tr>';
        }).join('') + '</tbody></table>';
    }
    function open() {
      var vms = (window.state && window.state.vms) || [];
      if (vms.length < 2) { window.gkToast('Need at least two disks to compare', 'warn'); return; }
      var opts = vms.map(function (v) { return '<option value="' + esc(v.id) + '">' + esc(v.name || v.id) + '</option>'; }).join('');
      selA.innerHTML = opts; selB.innerHTML = opts;
      selA.selectedIndex = 0; selB.selectedIndex = Math.min(1, vms.length - 1);
      ov.hidden = false; render(); cue('tick');
    }
    selA.addEventListener('change', render); selB.addEventListener('change', render);
    window.gkCompare = open;
  }

  /* ── Boot-score momentum — remember every score per disk, so a re-scan
        after a repair can answer "did it actually help?". Records history in
        localStorage keyed by the analyzed disk, celebrates the delta with a
        toast, and backs the 📈 trend sparkline. Token-driven, CSP-safe. ── */
  var SCORE_HKEY = 'gk.scoreHist';
  function readScoreHist() { try { var o = JSON.parse(localStorage.getItem(SCORE_HKEY) || '{}'); return (o && typeof o === 'object') ? o : {}; } catch (e) { return {}; } }
  function writeScoreHist(o) { try { localStorage.setItem(SCORE_HKEY, JSON.stringify(o)); } catch (e) {} }
  function histFor(key) { var a = readScoreHist()[key]; return Array.isArray(a) ? a : []; }
  function nowTs() { try { return Date.now(); } catch (e) { return 0; } }
  function relTime(ts) {
    if (!ts) return '';
    var d = Math.max(0, (nowTs() - ts) / 1000);
    if (d < 45) return 'just now';
    if (d < 3600) return Math.round(d / 60) + 'm ago';
    if (d < 86400) return Math.round(d / 3600) + 'h ago';
    return Math.round(d / 86400) + 'd ago';
  }
  function toneCol(s) { return s >= 90 ? tok('--success', '#39d98a') : s >= 60 ? tok('--warn', '#f5a623') : tok('--danger', '#ff5c5c'); }

  function mountScoreMomentum() {
    window.addEventListener('gk:analyze', function (ev) {
      var d = ev.detail || {};
      if (d.phase !== 'done') return;
      var s = d.score;
      if (typeof s !== 'number' || isNaN(s)) return; // only real boot scores
      s = Math.round(s);
      var key = d.vm || '(disk)';
      var all = readScoreHist();
      var arr = Array.isArray(all[key]) ? all[key] : [];
      var prev = arr.length ? arr[arr.length - 1].s : null;
      arr.push({ t: nowTs(), s: s });
      if (arr.length > 24) arr = arr.slice(arr.length - 24);
      all[key] = arr; writeScoreHist(all);
      if (prev != null && s !== prev) {
        var delta = s - prev, up = delta > 0;
        window.gkToast((up ? '▲ +' : '▼ −') + Math.abs(delta) + ' boot score since last scan', up ? 'ok' : 'warn',
          { label: '📈 Trend', run: function () { window.gkScoreTrend && window.gkScoreTrend(); } });
      }
    });
  }

  /* ── 📈 Boot-score trend sparkline (inline SVG, no assets) ── */
  function sparkSvg(hist) {
    var W = 520, H = 120, pad = 14, n = hist.length;
    var innerW = W - 2 * pad, innerH = H - 2 * pad;
    var xs = function (i) { return n <= 1 ? W / 2 : pad + i * innerW / (n - 1); };
    var ys = function (v) { return pad + (1 - Math.max(0, Math.min(100, v)) / 100) * innerH; };
    var latest = hist[n - 1].s, col = toneCol(latest);
    var acc = tok('--accent', '#7cc7ff'), soft = tok('--text-soft', '#8aa0c0');
    var pts = hist.map(function (h, i) { return xs(i).toFixed(1) + ',' + ys(h.s).toFixed(1); });
    var area = 'M ' + xs(0).toFixed(1) + ',' + (H - pad) + ' L ' + pts.join(' L ') + ' L ' + xs(n - 1).toFixed(1) + ',' + (H - pad) + ' Z';
    var g60 = ys(60), g90 = ys(90);
    var dots = hist.map(function (h, i) {
      var last = i === n - 1;
      return '<circle cx="' + xs(i).toFixed(1) + '" cy="' + ys(h.s).toFixed(1) + '" r="' + (last ? 4.5 : 2.6) + '" fill="' + (last ? col : acc) + '"' + (last ? ' class="gk-trend__last"' : '') + '><title>' + esc(h.s + ' · ' + relTime(h.t)) + '</title></circle>';
    }).join('');
    return '<svg class="gk-trend__svg" viewBox="0 0 ' + W + ' ' + H + '" role="img" aria-label="Boot score history">' +
      '<line x1="' + pad + '" y1="' + g90.toFixed(1) + '" x2="' + (W - pad) + '" y2="' + g90.toFixed(1) + '" stroke="' + tok('--success', '#39d98a') + '" stroke-dasharray="3 5" stroke-opacity="0.45"/>' +
      '<line x1="' + pad + '" y1="' + g60.toFixed(1) + '" x2="' + (W - pad) + '" y2="' + g60.toFixed(1) + '" stroke="' + tok('--warn', '#f5a623') + '" stroke-dasharray="3 5" stroke-opacity="0.45"/>' +
      '<text x="' + (W - pad) + '" y="' + (g90 - 4).toFixed(1) + '" text-anchor="end" font-size="9" fill="' + soft + '">90 · ship</text>' +
      '<text x="' + (W - pad) + '" y="' + (g60 - 4).toFixed(1) + '" text-anchor="end" font-size="9" fill="' + soft + '">60 · boot</text>' +
      '<path d="' + area + '" fill="' + acc + '" fill-opacity="0.12"/>' +
      (n > 1 ? '<polyline points="' + pts.join(' ') + '" fill="none" stroke="' + col + '" stroke-width="2.5" stroke-linejoin="round" stroke-linecap="round"/>' : '') +
      dots + '</svg>';
  }
  function trendTile(val, label) { return '<div class="gk-trend__tile"><b>' + esc(String(val)) + '</b><span>' + esc(label) + '</span></div>'; }
  function renderTrend(vm, hist) {
    var n = hist.length, latest = hist[n - 1].s, first = hist[0].s;
    var best = hist.reduce(function (m, h) { return Math.max(m, h.s); }, 0);
    var delta = latest - first, col = toneCol(latest);
    var name = (vm && (vm.name || vm.id)) || 'Disk';
    var verdict = latest >= 90 ? 'Boot-ready — ship it' : latest >= 60 ? 'Bootable with fixes' : 'Blocked — needs repair';
    return '<div class="gk-trend__head"><div><div class="gk-trend__name">' + esc(name) + '</div>' +
      '<div class="gk-trend__verdict" style="color:' + col + '">' + esc(verdict) + '</div></div>' +
      '<div class="gk-trend__big" style="color:' + col + '">' + latest + '<small>/100</small></div></div>' +
      sparkSvg(hist) +
      '<div class="gk-trend__stats">' +
        trendTile(best, 'Best') + trendTile(first, 'First') +
        trendTile((delta >= 0 ? '+' : '−') + Math.abs(delta), 'Δ since first') + trendTile(n, 'Scans') +
      '</div>' +
      '<div class="gk-trend__list">' + hist.slice().reverse().slice(0, 6).map(function (h) {
        return '<span class="gk-trend__pill"><b style="color:' + toneCol(h.s) + '">' + h.s + '</b> ' + esc(relTime(h.t)) + '</span>';
      }).join('') + '</div>';
  }
  function mountScoreTrend() {
    var css =
      '.gk-trend{position:fixed;inset:0;z-index:120;display:flex;align-items:center;justify-content:center;padding:24px;background:color-mix(in srgb,#05070c 72%,transparent);backdrop-filter:blur(4px)}' +
      '.gk-trend[hidden]{display:none}' +
      '.gk-trend__box{width:min(560px,94vw);background:var(--bg-card,#111a2b);border:1px solid var(--border-soft,#26324a);border-radius:var(--radius-lg,16px);box-shadow:0 24px 70px rgba(0,0,0,.5);overflow:hidden;animation:gkTrendIn .22s ease}' +
      '.gk-trend__box>header{display:flex;align-items:center;justify-content:space-between;padding:14px 18px;border-bottom:1px solid var(--border-soft,#26324a)}' +
      '.gk-trend__box h3{margin:0;font-size:15px;color:var(--text-main,#eaf2ff)}' +
      '.gk-trend__x{background:none;border:0;color:var(--text-soft,#8aa0c0);font-size:16px;line-height:1;cursor:pointer}.gk-trend__x:hover{color:var(--text-main,#eaf2ff)}' +
      '.gk-trend__body{padding:18px}' +
      '.gk-trend__head{display:flex;align-items:flex-start;justify-content:space-between;gap:12px}' +
      '.gk-trend__name{font-size:14px;font-weight:600;color:var(--text-main,#eaf2ff);word-break:break-all}' +
      '.gk-trend__verdict{font-size:11px;text-transform:uppercase;letter-spacing:.08em;margin-top:2px}' +
      '.gk-trend__big{font-size:42px;font-weight:800;line-height:1;font-variant-numeric:tabular-nums;flex:none}' +
      '.gk-trend__big small{font-size:14px;color:var(--text-soft,#8aa0c0);font-weight:600}' +
      '.gk-trend__svg{display:block;width:100%;height:auto;margin:10px 0 14px}' +
      '.gk-trend__stats{display:grid;grid-template-columns:repeat(4,1fr);gap:8px;margin-bottom:14px}' +
      '.gk-trend__tile{background:color-mix(in srgb,var(--text-main,#eaf2ff) 4%,transparent);border:1px solid var(--border-soft,#26324a);border-radius:12px;padding:9px 6px;text-align:center}' +
      '.gk-trend__tile b{display:block;font-size:19px;color:var(--text-main,#eaf2ff);font-variant-numeric:tabular-nums}' +
      '.gk-trend__tile span{font-size:9px;text-transform:uppercase;letter-spacing:.09em;color:var(--text-soft,#8aa0c0)}' +
      '.gk-trend__list{display:flex;flex-wrap:wrap;gap:6px}' +
      '.gk-trend__pill{font-size:11px;color:var(--text-soft,#8aa0c0);background:color-mix(in srgb,var(--text-main,#eaf2ff) 4%,transparent);border-radius:999px;padding:3px 9px}' +
      '.gk-trend__pill b{font-variant-numeric:tabular-nums}' +
      '@keyframes gkTrendIn{from{opacity:0;transform:translateY(10px) scale(.98)}to{opacity:1;transform:none}}' +
      '@keyframes gkTrendPulse{0%,100%{opacity:1}50%{opacity:.35}}' +
      '.gk-trend__last{animation:gkTrendPulse 1.6s ease-in-out infinite}' +
      '@media (prefers-reduced-motion:reduce){.gk-trend__box{animation:none}.gk-trend__last{animation:none}}';
    var style = el('style'); style.textContent = css; document.head.appendChild(style);

    var ov = el('div', 'gk-trend'); ov.hidden = true;
    ov.setAttribute('role', 'dialog'); ov.setAttribute('aria-modal', 'true');
    ov.innerHTML = '<div class="gk-trend__box"><header><h3>📈 Boot-score trend</h3>' +
      '<button type="button" class="gk-trend__x" aria-label="Close">✕</button></header>' +
      '<div class="gk-trend__body"></div></div>';
    document.body.appendChild(ov);
    var body = ov.querySelector('.gk-trend__body');
    function close() { ov.hidden = true; }
    ov.querySelector('.gk-trend__x').addEventListener('click', close);
    ov.addEventListener('mousedown', function (e) { if (e.target === ov) close(); });
    document.addEventListener('keydown', function (e) { if (e.key === 'Escape' && !ov.hidden) close(); });
    window.gkScoreTrend = function () {
      var st = window.state || {}, vm = st.selectedVm;
      var key = vm ? (vm.name || vm.id) : null;
      var hist = key ? histFor(key) : [];
      if (hist.length < 1) { window.gkToast('Run ⚡ Analyze on a disk to start its trend', 'warn'); return; }
      body.innerHTML = renderTrend(vm, hist);
      ov.hidden = false; cue('tick');
    };
  }

  ready(function () {
    mountAurora(); mountThemeWipe(); mountActivityLog(); mountToasts(); mountDock(); mountPalette(); mountScan(); mountStorm();
    mountShortcuts(); mountSkeletons(); mountFleetNav(); mountCopy(); mountStarterChips(); mountAudioCues(); mountDropCatcher(); mountCompare();
    mountScoreTrend(); mountScoreMomentum();
    window.gkToast('Press ⌘K for the command palette', 'info');
    setTimeout(function () { runTour(false); }, 900);
  });
})();
