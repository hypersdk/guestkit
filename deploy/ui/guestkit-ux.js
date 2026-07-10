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

  ready(function () { mountActivityLog(); mountToasts(); mountDock(); mountPalette();
    window.gkToast('Press ⌘K for the command palette', 'info');
  });
})();
