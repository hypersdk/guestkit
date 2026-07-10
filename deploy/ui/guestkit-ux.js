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
    return cmds;
  }

  var pal, palInput, palList, palCmds = [], palSel = 0, palFiltered = [];
  function mountPalette() {
    pal = el('div', 'gk-pal', '');
    pal.setAttribute('role', 'dialog'); pal.setAttribute('aria-modal', 'true'); pal.hidden = true;
    pal.innerHTML =
      '<div class="gk-pal__box">' +
      '<input class="gk-pal__in" id="gkPalInput" placeholder="Search disks, actions, themes — or ⚡ Ask Zeus…" autocomplete="off" spellcheck="false" />' +
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
    pal.hidden = false; renderPal(); palInput.focus();
  }
  function closePal() { if (pal) pal.hidden = true; }
  function renderPal() {
    var q = palInput.value.trim();
    if (!q) {
      var rec = recents(); var byId = {}; palCmds.forEach(function (c) { byId[c.id] = c; });
      palFiltered = rec.map(function (id) { return byId[id]; }).filter(Boolean).slice(0, 5);
      if (!palFiltered.length) palFiltered = palCmds.filter(function (c) { return c.cat === 'Action'; });
    } else {
      palFiltered = palCmds.map(function (c) { return { c: c, s: fuzzy(q, c.label + ' ' + c.cat) }; })
        .filter(function (x) { return x.s > 0; }).sort(function (a, b) { return b.s - a.s; }).slice(0, 12).map(function (x) { return x.c; });
    }
    palSel = 0;
    palList.innerHTML = palFiltered.length ? palFiltered.map(function (c, i) {
      return '<div class="gk-pal__row' + (i === 0 ? ' sel' : '') + '" data-i="' + i + '" role="option"><span class="gk-pal__cat">' + esc(c.cat) + '</span><span class="gk-pal__label">' + esc(c.label) + '</span><span class="gk-pal__hint">' + esc(c.hint || '') + '</span></div>';
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
    var ov = null, phaseEls = [], stepTimer = null;
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
      setTimeout(dismiss, s != null && s >= 90 ? 2600 : 3200);
    }
    function dismiss() { if (!ov) return; ov.classList.add('out'); var o = ov; ov = null; setTimeout(function () { o.remove(); }, 380); }
    function teardown() { if (stepTimer) clearTimeout(stepTimer); if (ov) { ov.remove(); ov = null; } phaseEls = []; }
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

  ready(function () {
    mountAurora(); mountThemeWipe(); mountActivityLog(); mountToasts(); mountDock(); mountPalette(); mountScan(); mountStorm();
    mountShortcuts(); mountSkeletons(); mountFleetNav(); mountCopy(); mountStarterChips();
    window.gkToast('Press ⌘K for the command palette', 'info');
  });
})();
