/* GuestKit Console — UI layer (mission rail, brain, HUD, dock, timeline) */

const MISSION_STEPS = [
  { id: 'capture', wizard: 'ingest', num: '01', label: 'Capture', hint: 'Upload, server image, URL', action: null },
  { id: 'fingerprint', wizard: 'assure', num: '02', label: 'Fingerprint', hint: 'OS, kernel, bootloader', action: 'inspect' },
  { id: 'diagnose', wizard: 'assure', num: '03', label: 'Diagnose', hint: 'Boot risks, driver gaps', action: 'doctor' },
  { id: 'repair', wizard: 'plan', num: '04', label: 'Repair', hint: 'VirtIO, cloud-init, fstab', action: 'repair-plan' },
  { id: 'convert', wizard: 'plan', num: '05', label: 'Convert', hint: 'qcow2, ova, resize', action: 'convert' },
  { id: 'launch', wizard: 'launch', num: '06', label: 'Launch', hint: 'VM manifest, DataVolume', action: 'provision' },
];

const TIMELINE_LABELS = {
  uploaded: 'Uploaded',
  scanned: 'Scanned',
  risks: 'Risks detected',
  repair: 'Repair applied',
  yaml: 'YAML generated',
  launched: 'Launched',
  converted: 'Converted',
};

const BRAIN_PROMPTS = [
  'Why will this VM fail to boot?',
  'Safest migration path?',
  'Generate VM YAML for 4 CPU / 8 GB',
];

const DOCK_PREFS_KEY = 'zyvor.dockPrefs';
const RAIL_COMPACT_KEY = 'zyvor.railCompact';
const CINEMA_KEY = 'zyvor.cinemaMode';

function gk$(sel) { return document.querySelector(sel); }

function osIcon(vm, cache) {
  const name = (vm?.name || '').toLowerCase();
  const os = cache?.inspect?.os?.distribution || cache?.inspect?.os?.os_type || '';
  const osL = os.toLowerCase();
  if (/windows|win/.test(name) || /windows/.test(osL)) return '🪟';
  if (/ubuntu|debian|fedora|rhel|centos|linux|cirros|buntu/.test(name + osL)) return '🐧';
  if ((vm?.format || '').toLowerCase() === 'ova') return '📦';
  if (vm?.kind === 'folder') return '📁';
  return '💾';
}

function riskLevel(cache) {
  const blockers = cache?.blockers?.length || 0;
  const warnings = cache?.bootScore != null && cache.bootScore < 70 ? 1 : 0;
  if (blockers) return { label: 'high risk', cls: 'risk' };
  if (warnings || (cache?.bootScore != null && cache.bootScore < 85)) return { label: 'caution', cls: 'warn' };
  if (cache?.bootScore != null) return { label: 'ready', cls: 'ready' };
  return { label: 'unknown', cls: '' };
}

function missionBadgeForStep(stepId, vm, cache) {
  if (!vm && stepId !== 'capture') return { text: 'blocked', cls: 'blocked' };
  switch (stepId) {
    case 'capture':
      return vm ? { text: 'done', cls: 'done' } : { text: 'ready', cls: 'ready' };
    case 'fingerprint':
      if (cache?.inspect) return { text: 'done', cls: 'done' };
      return vm ? { text: 'ready', cls: 'ready' } : { text: 'blocked', cls: 'blocked' };
    case 'diagnose':
      if (cache?.bootScore != null) {
        if (cache.blockers?.length) return { text: 'warning', cls: 'warning' };
        return { text: 'done', cls: 'done' };
      }
      return vm ? { text: 'ready', cls: 'ready' } : { text: 'blocked', cls: 'blocked' };
    case 'repair':
      if (cache?.lastOp === 'repair-plan') return { text: 'done', cls: 'done' };
      if (cache?.blockers?.length) return { text: 'ai-suggested', cls: 'ai-suggested' };
      return cache?.bootScore != null || vm ? { text: 'ready', cls: 'ready' } : { text: 'blocked', cls: 'blocked' };
    case 'convert':
      if (cache?.lastOp === 'convert') return { text: 'done', cls: 'done' };
      return vm ? { text: 'ready', cls: 'ready' } : { text: 'blocked', cls: 'blocked' };
    case 'launch':
      if (cache?.lastOp === 'provision') return { text: 'done', cls: 'done' };
      return vm ? { text: 'ready', cls: 'ready' } : { text: 'blocked', cls: 'blocked' };
    default:
      return { text: '—', cls: '' };
  }
}

function setReadinessRing(el, score) {
  if (!el) return;
  const pct = score != null ? Math.max(0, Math.min(100, Math.round(score))) : null;
  el.style.setProperty('--ring-pct', pct != null ? String(pct) : '0');
  const valueEl = el.querySelector('.readiness-ring__value');
  if (valueEl) valueEl.textContent = pct != null ? String(pct) : '—';
  if (pct == null) el.dataset.score = '0';
  else if (pct >= 85) el.dataset.score = 'high';
  else if (pct >= 60) el.dataset.score = 'mid';
  else el.dataset.score = 'low';
}

function missionWarningCount(stepId, vm, cache) {
  if (!vm) return 0;
  if (stepId === 'diagnose') {
    const fails = (cache?.checks || []).filter((c) => !c.passed).length;
    const blockers = cache?.blockers?.length || 0;
    return fails + blockers;
  }
  if (stepId === 'repair' && cache?.blockers?.length) return cache.blockers.length;
  return 0;
}

function renderMissionRail() {
  /* Mission rail removed — pipeline rendered by GuestKitNebula */
}

function renderBrainPanel() {
  const vm = window.state?.selectedVm;
  const cache = vm ? window.getVmCache?.(vm.id) || {} : {};
  window.GuestKitNebula?.renderBrainPanel?.(vm, cache);
  renderEvidenceConsole(vm, cache);
  window.GuestKitAi?.renderAiDeck?.();
  window.GuestKitAi?.renderAiNarrative?.();
  window.GuestKitFeatures?.loadJobHistory?.(vm?.id);
  renderBrainTimeline(vm?.id || (window.state?.selectedClusterVm ? `cluster:${window.state.selectedClusterVm.namespace}/${window.state.selectedClusterVm.name}` : null));
  syncBrainJobTracker();
}

function renderDiskInspector(vm, cache) {
  const el = gk$('#diskInspector');
  if (!el) return;
  if (!vm) {
    el.classList.add('hidden');
    return;
  }
  el.classList.remove('hidden');
  cache = cache || window.getVmCache?.(vm.id) || {};

  gk$('#diskInspectorTitle').textContent = vm.name || 'Unnamed disk';
  gk$('#diskInspectorMeta').textContent = `${vm.format || 'disk'} · ${window.fmtBytes?.(vm.size_bytes)} · ${vm.id.slice(0, 12)}…`;

  setReadinessRing(gk$('#diskInspectorRing'), cache.bootScore);
  const scoreEl = gk$('#diskInspectorScore');
  if (scoreEl && cache.bootScore == null) scoreEl.textContent = '—';

  const flight = gk$('#diskInspectorFlight');
  if (flight) {
    const steps = [
      { id: 'capture', label: 'Capture', done: true },
      { id: 'fingerprint', label: 'Fingerprint', done: !!cache.inspect },
      { id: 'diagnose', label: 'Diagnose', done: cache.bootScore != null, warn: (cache.blockers?.length || 0) > 0 },
      { id: 'repair', label: 'Repair', done: cache.lastOp === 'repair-plan' },
      { id: 'launch', label: 'Launch', done: cache.lastOp === 'provision' },
    ];
    flight.innerHTML = steps.map((s, i) => {
      const cls = s.done ? 'done' : (i === steps.findIndex((x) => !x.done) ? 'active' : '');
      const warn = s.warn ? ' warn' : '';
      const conn = i < steps.length - 1
        ? `<span class="flight-connector${s.done ? ' done' : ''}"></span>`
        : '';
      return `${i ? '' : ''}<div class="flight-node ${cls}${warn}"><span class="flight-node__dot"></span>${s.label}</div>${conn}`;
    }).join('');
  }

  const actions = gk$('#diskInspectorActions');
  if (actions) {
    actions.innerHTML = `
      <button type="button" class="disk-inspector__action-card" data-inspector-action="inspect">
        <strong>Inspect</strong><span>OS fingerprint</span>
      </button>
      <button type="button" class="disk-inspector__action-card" data-inspector-action="doctor">
        <strong>Doctor</strong><span>Boot score</span>
      </button>
      <button type="button" class="disk-inspector__action-card" data-inspector-action="provision">
        <strong>Launch</strong><span>KubeVirt VM</span>
      </button>`;
    const secondary = document.createElement('div');
    secondary.className = 'disk-inspector__secondary';
    secondary.innerHTML = `
      <button type="button" class="btn sm ghost" data-inspector-action="migration-plan">Migration plan</button>
      <button type="button" class="btn sm ghost" data-inspector-action="repair-plan">Repair dry-run</button>`;
    actions.appendChild(secondary);
    actions.querySelectorAll('[data-inspector-action]').forEach((btn) => {
      btn.addEventListener('click', (e) => {
        e.stopPropagation();
        const action = btn.dataset.inspectorAction;
        if (action === 'provision') showLaunchPreview(() => window.runAction?.('provision'));
        else window.runAction?.(action);
      });
    });
  }

  const dna = gk$('#diskInspectorDna');
  if (dna) {
    const inspect = cache.inspect;
    if (inspect?.os) {
      dna.textContent = `${inspect.os.distribution || inspect.os.os_type || 'unknown'} · ${inspect.boot?.mode || inspect.boot?.boot_mode || 'boot ?'} · last: ${cache.lastOp || 'never'}`;
    } else {
      dna.textContent = 'Run Inspect to populate disk DNA.';
    }
  }
}

function updateSelectedCommandBar(vm, cache) {
  const bar = gk$('#selectedCommandBar');
  if (!bar) return;
  if (!vm) {
    bar.classList.add('hidden');
    return;
  }
  bar.classList.remove('hidden');
  cache = cache || window.getVmCache?.(vm.id) || {};
  const sys = window.state?.systemStatus || {};
  gk$('#commandBarName').textContent = vm.name || 'Unnamed';
  const scoreEl = gk$('#commandBarScore');
  if (scoreEl) {
    scoreEl.textContent = cache.bootScore != null ? `boot ${Math.round(cache.bootScore)}` : 'not scanned';
  }
  const riskEl = gk$('#commandBarRisk');
  if (riskEl) {
    const n = (cache?.blockers?.length || 0) + (cache?.checks || []).filter((c) => !c.passed).length;
    if (n > 0) {
      riskEl.textContent = `${n} risk${n === 1 ? '' : 's'}`;
      riskEl.classList.remove('hidden');
    } else {
      riskEl.textContent = '';
      riskEl.classList.add('hidden');
    }
  }
  const cmds = [
    ['cmdBarInspect', 'inspect'],
    ['cmdBarDoctor', 'doctor'],
    ['cmdBarLaunch', 'launch'],
    ['cmdBarYaml', 'provision'],
    ['cmdBarRepair', 'repair-plan'],
    ['cmdBarMigrate', 'migration-plan'],
  ];
  cmds.forEach(([id, action]) => {
    const btn = gk$(`#${id}`);
    if (!btn) return;
    const reason = window.GuestKitJourney?.explainDisabledAction?.(action, vm, cache, sys) || '';
    btn.disabled = !!reason && action !== 'inspect';
    btn.title = reason || btn.getAttribute('data-original-title') || '';
    btn.classList.toggle('primary', action === (window.GuestKitJourney?.deriveNextAction?.(vm, cache, sys)?.primary));
    btn.classList.toggle('ghost', !!reason);
  });
}

function renderEvidenceConsole(vm, cache) {
  if (!vm) {
    gk$('#timelineEmpty')?.classList.remove('hidden');
    gk$('#evidenceTimeline')?.replaceChildren();
    gk$('#riskEmpty')?.classList.remove('hidden');
    gk$('#riskContent')?.classList.add('hidden');
    gk$('#logsEmpty')?.classList.remove('hidden');
    gk$('#evidenceLogs')?.replaceChildren();
    window.GuestKitJourney?.renderFindingsTab?.({});
    window.GuestKitJourney?.renderDiffTab?.({});
    return;
  }
  cache = cache || window.getVmCache?.(vm.id) || {};
  renderEvidenceTimeline(vm.id);
  renderRiskTab(cache);
  renderEvidenceLogs(vm, cache);
  window.GuestKitJourney?.renderFindingsTab?.(cache);
  window.GuestKitJourney?.renderDiffTab?.(cache);
}

function renderEvidenceTimeline(vmId) {
  const list = gk$('#evidenceTimeline');
  const empty = gk$('#timelineEmpty');
  if (!list) return;
  const events = loadTimeline(vmId);
  if (!events.length) {
    empty?.classList.remove('hidden');
    list.innerHTML = '';
    return;
  }
  empty?.classList.add('hidden');
  list.innerHTML = events.map((e) => {
    const label = TIMELINE_LABELS[e.event] || e.event;
    const time = new Date(e.at).toLocaleString([], { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
    return `<li class="done"><strong>${window.escapeHtml?.(label) || label}</strong>${e.detail ? ` — ${window.escapeHtml?.(e.detail) || e.detail}` : ''} <span class="mono" style="color:var(--text-muted)">${time}</span></li>`;
  }).join('');
}

function renderRiskTab(cache) {
  const empty = gk$('#riskEmpty');
  const content = gk$('#riskContent');
  if (!content) return;
  const checks = (cache?.checks || []).filter((c) => !c.passed);
  const blockers = cache?.blockers || [];
  const items = [
    ...blockers.map((b) => ({ msg: typeof b === 'string' ? b : (b.message || b.id || 'blocker'), fail: true })),
    ...checks.map((c) => ({ msg: c.message || c.id, fail: !c.passed })),
  ];
  if (!items.length) {
    empty?.classList.remove('hidden');
    content.classList.add('hidden');
    content.innerHTML = '';
    return;
  }
  empty?.classList.add('hidden');
  content.classList.remove('hidden');
  content.innerHTML = items.map((it) =>
    `<div class="risk-item${it.fail ? ' fail' : ''}">${window.escapeHtml?.(it.msg) || it.msg}</div>`
  ).join('');
}

function appendEvidenceLog(line) {
  const logs = gk$('#evidenceLogs');
  const empty = gk$('#logsEmpty');
  if (!logs || !line) return;
  empty?.classList.add('hidden');
  const prev = logs.textContent || '';
  logs.textContent = prev ? `${prev}\n${line}` : line;
  logs.scrollTop = logs.scrollHeight;
}

function clearEvidenceLogs() {
  const logs = gk$('#evidenceLogs');
  if (logs) logs.textContent = '';
}

function renderEvidenceLogs(vm, cache) {
  const empty = gk$('#logsEmpty');
  const logs = gk$('#evidenceLogs');
  if (!logs) return;
  const lines = [];
  if (cache.lastOp) lines.push(`[${cache.lastOp}] status=${cache.status || 'unknown'}`);
  if (cache.lastError) lines.push(`ERROR: ${cache.lastError}`);
  if (cache.bootScore != null) lines.push(`boot_score=${Math.round(cache.bootScore)}`);
  if (window.state?.lastJobResult) {
    try {
      lines.push(JSON.stringify(window.state.lastJobResult, null, 2).slice(0, 4000));
    } catch { /* ignore */ }
  }
  if (!lines.length) {
    empty?.classList.remove('hidden');
    logs.textContent = '';
    return;
  }
  empty?.classList.add('hidden');
  logs.textContent = lines.join('\n');
}

function clearDiskSelectionUi() {
  gk$('#diskInspector')?.classList.add('hidden');
  gk$('#selectedCommandBar')?.classList.add('hidden');
  document.querySelector('.guestkit-shell')?.classList.remove('command-bay-active');
  renderEvidenceConsole(null);
}

function scrollToDiskContext() {
  const inspector = gk$('#diskInspector');
  const bar = gk$('#selectedCommandBar');
  const el = inspector && !inspector.classList.contains('hidden') ? inspector : bar;
  if (!el || el.classList.contains('hidden')) return;
  requestAnimationFrame(() => {
    el.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
    el.classList.remove('disk-inspector--reveal');
    void el.offsetWidth;
    el.classList.add('disk-inspector--reveal');
  });
}

function scrollToEvidenceConsole() {
  const el = gk$('#panel-results');
  if (!el) return;
  requestAnimationFrame(() => el.scrollIntoView({ block: 'nearest', behavior: 'smooth' }));
}

function clearDiskSelection() {
  if (!window.state?.selectedVm) return false;
  window.state.selectedVm = null;
  window.renderFleet?.();
  clearDiskSelectionUi();
  window.updateSelectionPanels?.();
  renderMissionRail();
  window.GuestKitNebula?.renderAllNebula?.(null, {});
  return true;
}

function timelineKey(vmId) {
  return `zyvor.timeline.${vmId}`;
}

function loadTimeline(vmId) {
  if (!vmId) return [];
  try {
    return JSON.parse(localStorage.getItem(timelineKey(vmId)) || '[]');
  } catch {
    return [];
  }
}

function appendTimelineEvent(vmId, event, detail) {
  if (!vmId) return;
  const events = loadTimeline(vmId);
  events.push({ event, detail: detail || '', at: new Date().toISOString() });
  localStorage.setItem(timelineKey(vmId), JSON.stringify(events.slice(-20)));
  renderBrainTimeline(vmId);
}

function renderBrainTimeline(vmId) {
  const el = gk$('#brainTimeline');
  if (!el) return;
  const events = loadTimeline(vmId);
  if (!events.length) {
    el.innerHTML = '<li>Mission timeline appears after first action.</li>';
    return;
  }
  el.innerHTML = events.map((e) => {
    const label = TIMELINE_LABELS[e.event] || e.event;
    const time = new Date(e.at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    return `<li class="done">${label}${e.detail ? ` — ${window.escapeHtml?.(e.detail) || e.detail}` : ''} <span class="mono">${time}</span></li>`;
  }).join('');
}

function syncBrainJobTracker() {
  const tracker = gk$('#brainJobTracker');
  const legacy = gk$('#jobTracker');
  if (!tracker || !legacy) return;
  const active = !legacy.classList.contains('hidden');
  tracker.classList.toggle('hidden', !active);
  legacy.classList.toggle('job-tracker--mirrored', active);
  if (active) {
    tracker.className = 'brain-section';
    tracker.innerHTML = `<h3 class="brain-section__title cyan">Live Job</h3>${legacy.innerHTML}`;
  } else {
    tracker.innerHTML = '';
  }
}

function resetJobBadge() {
  const badge = gk$('#jobBadge');
  if (!badge || window.state?.activeJob) return;
  badge.textContent = 'idle';
  badge.className = 'badge live';
}

function mapTimelineEvent(action) {
  const map = {
    import: 'uploaded',
    'repair-plan': 'repair',
    provision: 'launched',
    convert: 'converted',
    inspect: 'scanned',
    doctor: 'risks',
    'migration-plan': 'yaml',
  };
  return map[action] || null;
}

function fleetCardVariant(cache) {
  if (!cache?.inspect) return 'unscanned';
  if (cache.blockers?.length || (cache.bootScore != null && cache.bootScore < 60)) return 'risky';
  if (cache.bootScore != null && cache.bootScore >= 85) return 'ready';
  return '';
}

function renderFleetDiskCard(vm, selected, cache) {
  const smoke = window.isSmokeDisk?.(vm);
  const risk = riskLevel(cache);
  const status = window.vmStatusLabel?.(cache, vm) || 'unknown';
  const icon = osIcon(vm, cache);
  const scanned = cache?.lastOp ? cache.lastOp : 'never';
  const score = cache?.bootScore;
  const scorePct = score != null ? Math.round(score) : 0;
  const variant = fleetCardVariant(cache);
  const conf = window.GuestKitJourney?.deriveConfidenceBreakdown?.(cache) || {};
  const confRings = ['boot', 'fs', 'drv', 'kv', 'ci'].map((k) => {
    const v = conf[k];
    return `<span class="conf-ring" title="${k.toUpperCase()}"><span class="conf-ring__label">${k.toUpperCase()}</span>
      <span class="conf-ring__bar"><span class="conf-ring__fill" style="width:${v != null ? v : 0}%"></span></span></span>`;
  }).join('');
  const primaryAction = !cache?.inspect ? 'inspect' : (cache.bootScore == null ? 'doctor' : 'provision');
  const primaryLabel = primaryAction === 'inspect' ? 'Fingerprint' : (primaryAction === 'doctor' ? 'Doctor' : 'Launch');
  return `
    <span class="disk-card__head">
      <span class="disk-card__icon">${icon}</span>
      <span>
        <p class="disk-card__name">${window.escapeHtml?.(vm.name || 'unnamed') || vm.name}</p>
        <p class="disk-card__meta">${vm.format || 'disk'} · ${window.fmtBytes?.(vm.size_bytes)} · ${vm.id.slice(0, 8)}…</p>
      </span>
    </span>
    <div class="disk-card__tags">
      <span class="disk-tag ${status === 'ready' ? 'ready' : status === 'failed' ? 'risk' : ''}">${status}</span>
      ${risk.cls ? `<span class="disk-tag ${risk.cls}">${risk.label}</span>` : ''}
      ${score != null ? `<span class="disk-tag ready">boot ${scorePct}</span>` : ''}
      ${smoke ? '<span class="disk-tag warn">smoke</span>' : ''}
    </div>
    <div class="disk-card__confidence">${confRings}</div>
    <div class="disk-card__readiness">
      <span>readiness</span>
      <div class="disk-card__readiness-bar"><div class="disk-card__readiness-fill" style="width:${scorePct}%"></div></div>
      <span class="mono">${score != null ? scorePct : '—'}</span>
    </div>
    <div class="disk-card__actions disk-card__actions--compact">
      <button type="button" class="btn sm glass" data-disk-action="inspect">Inspect</button>
      <button type="button" class="btn sm accent" data-disk-action="doctor">Doctor</button>
      <button type="button" class="btn sm primary" data-disk-action="${primaryAction}">${primaryLabel}</button>
    </div>
    <p class="disk-card__meta">Last scan: ${scanned}</p>`;
}

function bindDiskCardActions(card, vm) {
  const variant = fleetCardVariant(window.getVmCache?.(vm.id));
  if (variant) card.dataset.variant = variant;
  card.querySelectorAll('[data-disk-action]').forEach((btn) => {
    btn.addEventListener('click', (e) => {
      e.stopPropagation();
      window.selectVm?.(vm);
      const action = btn.dataset.diskAction;
      if (action === 'provision') showLaunchPreview(() => window.runAction?.('provision'));
      else window.runAction?.(action);
    });
  });
}

function getDockPrefs() {
  try {
    return JSON.parse(localStorage.getItem(DOCK_PREFS_KEY) || '{}');
  } catch {
    return {};
  }
}

function saveDockPrefs(prefs) {
  localStorage.setItem(DOCK_PREFS_KEY, JSON.stringify({ ...getDockPrefs(), ...prefs }));
}

function applyDockPrefs() {
  const prefs = getDockPrefs();
  const dock = gk$('#commandDock');
  const root = document.getElementById('nebulaRoot') || document.querySelector('.guestkit-shell');
  const cinema = localStorage.getItem(CINEMA_KEY) === '1' || prefs.cinema;
  if (root) {
    root.dataset.cinema = cinema ? 'true' : 'false';
    root.classList.toggle('cinema-mode', cinema);
  }
  gk$('#cinemaModeBtn')?.classList.toggle('active', cinema);
  gk$('#dockControls')?.classList.toggle('hidden', !cinema);
  if (!dock) return;
  dock.classList.remove('pos-bottom', 'pos-left', 'pos-right', 'compact', 'auto-hide', 'pinned', 'dock-hidden');
  if (!cinema) {
    dock.classList.add('dock-hidden');
    document.documentElement.style.setProperty('--iw-dock-offset', '0px');
    return;
  }
  dock.classList.remove('dock-hidden');
  dock.classList.add(`pos-${prefs.position || 'bottom'}`);
  if (prefs.compact) dock.classList.add('compact');
  if (prefs.autoHide) dock.classList.add('auto-hide');
  if (prefs.pinned !== false) dock.classList.add('pinned');
  const offset = prefs.position === 'left' || prefs.position === 'right' ? '24px' : '92px';
  document.documentElement.style.setProperty('--iw-dock-offset', offset);
}

function setupCinemaMode() {
  gk$('#cinemaModeBtn')?.addEventListener('click', () => {
    const on = localStorage.getItem(CINEMA_KEY) !== '1';
    localStorage.setItem(CINEMA_KEY, on ? '1' : '0');
    applyDockPrefs();
    window.toast?.(on ? 'Cinema mode — focused workspace' : 'Normal mode', 'ok');
  });
  gk$('#cinemaExitBtn')?.addEventListener('click', () => {
    localStorage.setItem(CINEMA_KEY, '0');
    applyDockPrefs();
  });
}

function setupBrainDrawer() {
  /* Nebula brain drawer wired in initGuestKitNebula */
}

function setupNebulaExtras() {
  gk$('#statusStrip')?.addEventListener('click', () => {
    const drawer = gk$('#clusterReadinessDrawer');
    drawer?.classList.add('open');
    drawer?.classList.remove('hidden');
    window.GuestKitJourney?.renderClusterReadinessDrawer?.(window.state?.systemStatus);
  });
  gk$('#clusterReadinessClose')?.addEventListener('click', () => {
    gk$('#clusterReadinessDrawer')?.classList.remove('open');
  });
  gk$('#copyDebugBundleBtn')?.addEventListener('click', () => window.GuestKitJourney?.copyDebugBundle?.());
  document.addEventListener('click', (e) => {
    const btn = e.target.closest('[data-hero-action]');
    if (btn) {
      e.preventDefault();
      window.runAction?.(btn.dataset.heroAction);
    }
  });
  window.GuestKitJourney?.checkSessionRestore?.();
}

function setupRailCompact() {
  gk$('#railCompactBtn')?.addEventListener('click', () => {
    const rail = gk$('#missionRail');
    const compact = rail?.dataset.compact !== 'true';
    localStorage.setItem(RAIL_COMPACT_KEY, compact ? '1' : '0');
    if (rail) rail.dataset.compact = compact ? 'true' : 'false';
  });
}

function showLaunchMonitor() {
  const stages = [
    { id: 'create', label: 'Create VM', status: 'done' },
    { id: 'import', label: 'Import disk', status: 'running' },
    { id: 'schedule', label: 'Schedule pod', status: 'waiting' },
    { id: 'boot', label: 'Boot guest', status: 'waiting' },
    { id: 'agent', label: 'Guest agent', status: 'waiting' },
    { id: 'console', label: 'Console ready', status: 'waiting' },
  ];
  window.GuestKitJourney?.renderLaunchMonitor?.(stages);
  showModal('launchMonitorModal');
  gk$('#launchMonitorClose')?.addEventListener('click', () => hideModal('launchMonitorModal'), { once: true });
  pollLaunchMonitor();
}

async function pollLaunchMonitor() {
  const vm = window.state?.selectedVm;
  if (!vm) return;
  try {
    const data = await window.api?.('/kubevirt/vms');
    const vms = data?.data || [];
    const name = (vm.name || '').replace(/\.[^.]+$/, '');
    const match = vms.find((v) => v.name === name || v.name?.includes(name.slice(0, 12)));
    const phase = (match?.phase || match?.status || '').toLowerCase();
    const stages = [
      { id: 'create', label: 'Create VM', status: 'done' },
      { id: 'import', label: 'Import disk', status: 'done' },
      { id: 'schedule', label: 'Schedule pod', status: phase.includes('sched') || phase.includes('pend') ? 'running' : 'done' },
      { id: 'boot', label: 'Boot guest', status: phase.includes('run') ? 'done' : (phase ? 'running' : 'waiting') },
      { id: 'agent', label: 'Guest agent', status: match?.guest_agent_connected ? 'done' : 'waiting' },
      { id: 'console', label: 'Console ready', status: phase.includes('run') ? 'running' : 'waiting' },
    ];
    window.GuestKitJourney?.renderLaunchMonitor?.(stages);
  } catch { /* */ }
}

function setupCommandBar() {
  gk$('#cmdBarInspect')?.addEventListener('click', () => window.runAction?.('inspect'));
  gk$('#cmdBarDoctor')?.addEventListener('click', () => window.runAction?.('doctor'));
  gk$('#cmdBarLaunch')?.addEventListener('click', () => showLaunchPreview(() => window.runAction?.('provision')));
  gk$('#cmdBarYaml')?.addEventListener('click', () => window.runAction?.('provision'));
  gk$('#cmdBarRepair')?.addEventListener('click', () => window.runAction?.('repair-plan'));
  gk$('#cmdBarMigrate')?.addEventListener('click', () => window.runAction?.('migration-plan'));
  gk$('#cmdBarClear')?.addEventListener('click', () => clearDiskSelection());
  gk$('#diskInspectorClose')?.addEventListener('click', () => clearDiskSelection());
  document.querySelectorAll('.evidence-cta').forEach((btn) => {
    btn.addEventListener('click', () => {
      const action = btn.dataset.action;
      if (action) window.runAction?.(action);
    });
  });
}

function setupDockMagnification() {
  const dock = gk$('#commandDock');
  const inner = dock?.querySelector('.mac-dock-inner');
  if (!inner) return;

  const items = () => [...inner.querySelectorAll('.mac-dock-item, .dock-item')];
  const RANGE = 110;
  const MAX_BOOST = 0.55;

  function resetScales() {
    items().forEach((item) => item.style.setProperty('--dock-scale', '1'));
  }

  function onPointerMove(e) {
    const pointer = dock?.classList.contains('pos-left') || dock?.classList.contains('pos-right')
      ? e.clientY
      : e.clientX;

    items().forEach((item) => {
      const rect = item.getBoundingClientRect();
      const center = dock?.classList.contains('pos-left') || dock?.classList.contains('pos-right')
        ? rect.top + rect.height / 2
        : rect.left + rect.width / 2;
      const dist = Math.abs(pointer - center);
      const t = Math.max(0, 1 - dist / RANGE);
      const eased = t * t;
      const scale = 1 + eased * MAX_BOOST;
      item.style.setProperty('--dock-scale', scale.toFixed(3));
    });
  }

  inner.addEventListener('mousemove', onPointerMove);
  inner.addEventListener('mouseleave', resetScales);
}

function setupCommandDock() {
  const dock = gk$('#commandDock');
  if (!dock) return;
  applyDockPrefs();
  setupDockMagnification();

  gk$('#dockPinBtn')?.addEventListener('click', () => {
    const prefs = getDockPrefs();
    saveDockPrefs({ pinned: !prefs.pinned });
    applyDockPrefs();
  });
  gk$('#dockCompactBtn')?.addEventListener('click', () => {
    const prefs = getDockPrefs();
    saveDockPrefs({ compact: !prefs.compact });
    applyDockPrefs();
  });
  gk$('#dockAutoHideBtn')?.addEventListener('click', () => {
    const prefs = getDockPrefs();
    saveDockPrefs({ autoHide: !prefs.autoHide });
    applyDockPrefs();
  });
  gk$('#dockPosBtn')?.addEventListener('click', () => {
    const order = ['bottom', 'left', 'right'];
    const prefs = getDockPrefs();
    const idx = order.indexOf(prefs.position || 'bottom');
    saveDockPrefs({ position: order[(idx + 1) % order.length] });
    applyDockPrefs();
  });

  gk$('#dockLogsBtn')?.addEventListener('click', () => {
    window.scrollToPanel?.('assure');
    window.setActiveTab?.('logs');
    scrollToEvidenceConsole();
  });
  gk$('#dockYamlBtn')?.addEventListener('click', () => {
    window.setActiveTab?.('yaml');
  });
  gk$('#dockAiBtn')?.addEventListener('click', () => {
    gk$('#brainAskInput')?.focus();
  });
}

async function refreshHudStatus() {
  const cfg = window.state?.uiConfig;
  let status = {};
  try {
    status = (await window.api?.('/system/status'))?.data || {};
  } catch {
    status = {};
  }
  if (window.state) window.state.systemStatus = status;
  window.GuestKitJourney?.renderClusterReadinessDrawer?.(status);
  const healthPill = gk$('#healthPill');
  const healthDot = gk$('#healthDot');
  const healthLabel = gk$('#healthLabel');
  const stripLabel = gk$('#healthLabelStrip');
  const stripDot = gk$('#statusDotApi');
  const agentOk = status.agent === 'online';
  if (healthDot) healthDot.className = 'pulse-dot ' + (agentOk ? 'ok' : '');
  if (healthPill) healthPill.classList.toggle('ok', agentOk);
  if (healthLabel) healthLabel.textContent = agentOk ? 'Agent online' : (status.agent || 'Connecting…');
  if (stripLabel) stripLabel.textContent = status.api === 'offline' ? 'offline' : 'online';
  if (stripDot) stripDot.className = 'status-dot ' + (status.api === 'offline' ? 'err' : 'ok');

  const set = (id, val, ok) => {
    const el = gk$(`#${id}`);
    if (!el) return;
    el.textContent = val || '—';
    const dot = el.parentElement?.querySelector('.status-dot, .status-strip__dot');
    if (dot) {
      dot.className = (dot.classList.contains('status-dot') ? 'status-dot ' : 'status-strip__dot ')
        + (ok === true ? 'ok' : ok === false ? 'err' : ok === 'ai' ? 'ai' : '');
    }
  };

  set('hudAgent', status.agent || (window.state?.agentReachable ? 'online' : 'idle'), status.agent === 'online');
  set('hudCluster', status.cluster || cfg?.cluster_name || 'local', status.cluster === 'ready');
  set('hudStorage', status.storage || cfg?.storage_path || '—', true);
  set('hudKubevirt', status.kubevirt || 'unknown', status.kubevirt === 'healthy');
  set('hudCdi', status.cdi || 'unknown', status.cdi === 'ready');
  set('hudLastScan', status.last_scan ? new Date(status.last_scan).toLocaleString() : '—', status.last_scan ? 'ai' : null);

  const workerEl = gk$('#hudWorker');
  if (workerEl) {
    workerEl.textContent = status.worker || '—';
    const dot = workerEl.parentElement?.querySelector('.status-strip__dot');
    if (dot) dot.className = 'status-strip__dot ' + (status.worker === 'online' ? 'ok' : '');
  }

  const disksEl = gk$('#hudDiskCount');
  if (disksEl && status.disk_count != null) disksEl.textContent = `${status.disk_count} disks on host`;

  const countEl = gk$('#hudVmCount');
  if (countEl) {
    const disks = (window.state?.vms || []).filter((v) => window.isFleetDisk?.(v) ?? true);
    const clusterN = status.cluster_vm_count;
    const n = window.state?.fleetMode === 'cluster'
      ? (window.state?.clusterVms?.length || clusterN || 0)
      : (disks.length || status.disk_count || 0);
    const suffix = clusterN != null && window.state?.fleetMode !== 'cluster'
      ? ` · ${clusterN} cluster VM${clusterN === 1 ? '' : 's'}`
      : '';
    countEl.textContent = `${n} asset${n === 1 ? '' : 's'}${suffix}`;
  }
}

function showModal(id) {
  gk$(`#${id}`)?.classList.remove('hidden');
}

function hideModal(id) {
  gk$(`#${id}`)?.classList.add('hidden');
}

function renderBrainChecksHeatmap(checks) {
  const el = gk$('#brainChecks');
  if (!el) return;
  if (!checks?.length) {
    el.innerHTML = '';
    el.classList.add('hidden');
    return;
  }
  el.classList.remove('hidden');
  el.innerHTML = `<p class="brain-section__title">Boot checks <span class="brain-section__hint">click for AI explain</span></p><div class="brain-checks-grid">${
    checks.slice(0, 12).map((c) => {
      const cls = c.passed ? 'pass' : 'fail';
      return `<button type="button" class="brain-check ${cls}" data-check-id="${window.escapeHtml?.(c.id)}" data-check-msg="${window.escapeHtml?.(c.message || c.id)}" title="${window.escapeHtml?.(c.message || c.id)}">${window.escapeHtml?.(c.id)}</button>`;
    }).join('')
  }</div>`;
  el.querySelectorAll('.brain-check').forEach((btn) => {
    btn.addEventListener('click', () => {
      window.GuestKitAi?.explainBootCheck?.(btn.dataset.checkId, btn.dataset.checkMsg);
    });
  });
}

function parseLaunchSpec(vm, cache) {
  let cpu = '4';
  let mem = '8 GiB';
  const yaml = window.state?.lastYaml || '';
  const cores = yaml.match(/cores:\s*(\d+)/);
  const memMatch = yaml.match(/memory:\s*(\S+)/);
  if (cores) cpu = cores[1];
  if (memMatch) mem = memMatch[1];
  const vmName = (vm?.name || 'guest').replace(/\.[^.]+$/, '');
  return { cpu, mem, vmName };
}

function showLaunchPreview(onConfirm) {
  const vm = window.state?.selectedVm;
  const cache = vm ? window.getVmCache?.(vm.id) : {};
  const spec = parseLaunchSpec(vm, cache);
  const score = cache?.bootScore;
  const blockers = cache?.blockers?.length || 0;
  const hasInspect = !!cache?.inspect;
  const hasDoctor = score != null;

  setReadinessRing(gk$('#launchReadinessRing'), score);
  const label = gk$('#launchReadinessLabel');
  if (label) {
    if (score == null) label.textContent = 'Run Doctor before launch';
    else if (blockers) label.textContent = `${blockers} blocker(s) — review risks`;
    else if (score >= 85) label.textContent = 'Ready to launch';
    else label.textContent = 'Launch with caution';
  }

  const checklist = gk$('#launchChecklist');
  if (checklist) {
    const sys = window.state?.systemStatus || {};
    const kvOk = sys.kubevirt === 'healthy' || sys.kubevirt === 'ready';
    const cdiOk = sys.cdi === 'ready';
    const items = [
      { text: 'Disk captured & fingerprinted', state: hasInspect ? 'pass' : 'pending' },
      { text: 'Boot score assessed (Doctor)', state: hasDoctor ? (score >= 70 ? 'pass' : 'warn') : 'pending' },
      { text: 'No critical blockers', state: blockers ? 'fail' : (hasDoctor ? 'pass' : 'pending') },
      { text: 'KubeVirt cluster ready', state: kvOk ? 'pass' : 'fail' },
      { text: 'CDI import ready', state: cdiOk ? 'pass' : 'fail' },
      { text: 'Migration target selected', state: document.getElementById('targetSelect')?.value ? 'pass' : 'pending' },
      { text: 'VM manifest spec ready', state: window.state?.lastYaml ? 'pass' : 'warn' },
    ];
    checklist.innerHTML = items.map((it) => `<li class="${it.state}">${window.escapeHtml?.(it.text) || it.text}</li>`).join('');
  }

  const tbody = gk$('#launchPreviewBody');
  if (tbody && vm) {
    tbody.innerHTML = `
      <tr><td>VM name</td><td>${window.escapeHtml?.(spec.vmName)}</td></tr>
      <tr><td>Namespace</td><td>${window.escapeHtml?.(window.state?.uiConfig?.default_namespace || 'default')}</td></tr>
      <tr><td>CPU / Memory</td><td>${spec.cpu} vCPU · ${window.escapeHtml?.(spec.mem)}</td></tr>
      <tr><td>Disk</td><td>${window.fmtBytes?.(vm.size_bytes)} ${vm.format}</td></tr>
      <tr><td>Boot score</td><td>${score != null ? Math.round(score) : '—'}</td></tr>
      <tr><td>Migrate score</td><td>${cache?.migrateScore != null ? Math.round(cache.migrateScore) : '—'}</td></tr>
      <tr><td>Target</td><td>${window.escapeHtml?.(document.getElementById('targetSelect')?.value || 'kubevirt')}</td></tr>
      <tr id="launchAiAdviceRow" class="hidden"><td>AI advice</td><td id="launchAiAdvice">—</td></tr>`;
  window.GuestKitAi?.aiLaunchAdvice?.();
  }
  showModal('launchPreviewModal');
  const confirmBtn = gk$('#launchPreviewConfirm');
  const dryBtn = gk$('#launchPreviewDryRun');
  const replace = (el, fn) => {
    if (!el) return;
    const clone = el.cloneNode(true);
    el.parentNode.replaceChild(clone, el);
    clone.addEventListener('click', fn);
  };
  replace(gk$('#launchPreviewCancel'), () => hideModal('launchPreviewModal'));
  replace(confirmBtn, () => {
    hideModal('launchPreviewModal');
    onConfirm?.();
  });
  replace(dryBtn, () => {
    hideModal('launchPreviewModal');
    window.runAction?.('repair-plan');
  });
}

function showConvertStudio() {
  showModal('convertStudioModal');
  const vm = window.state?.selectedVm;
  const srcFmt = gk$('#convertSourceFmt');
  const tgtFmt = gk$('#convertTargetFmt');
  if (srcFmt && vm) srcFmt.textContent = vm.format || 'unknown';
  if (tgtFmt) tgtFmt.value = 'qcow2';
}

async function runConvertJob() {
  const vm = window.state?.selectedVm;
  if (!vm) {
    window.toast?.('Select a disk first', 'err');
    return;
  }
  const target = gk$('#convertTargetFmt')?.value || 'qcow2';
  const compression = gk$('#convertCompression')?.checked ?? false;
  try {
    const data = await window.api?.(`/vms/${vm.id}/convert`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ target_format: target, compression }),
    });
    hideModal('convertStudioModal');
    window.toast?.('Conversion job queued', 'ok');
    if (data?.data?.job_id) {
      window.showJobTracker?.('convert', data.data.job_id);
      await window.pollJob?.(data.data.job_id, 'convert');
    }
    await window.loadFleet?.();
  } catch (e) {
    window.toast?.(e.message, 'err');
  }
}

function setupImportPortal() {
  gk$('#importUrlBtn')?.addEventListener('click', () => showModal('importUrlModal'));
  gk$('#importS3Btn')?.addEventListener('click', () => showModal('importS3Modal'));
  gk$('#importNfsBtn')?.addEventListener('click', () => showModal('importNfsModal'));
  gk$('#importUrlSubmit')?.addEventListener('click', submitUrlImport);
  gk$('#importS3Submit')?.addEventListener('click', submitS3Import);
  gk$('#importUrlCancel')?.addEventListener('click', () => hideModal('importUrlModal'));
  gk$('#importS3Cancel')?.addEventListener('click', () => hideModal('importS3Modal'));
  gk$('#convertSubmit')?.addEventListener('click', runConvertJob);
  gk$('#convertCancel')?.addEventListener('click', () => hideModal('convertStudioModal'));
}

async function submitUrlImport() {
  const url = gk$('#importUrlInput')?.value?.trim();
  if (!url) return;
  try {
    const data = await window.api?.('/vms/import-from-url', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ url }),
    });
    hideModal('importUrlModal');
    window.toast?.('Import started', 'ok');
    await window.loadFleet?.();
    if (data?.data?.id) window.selectVm?.(data.data);
  } catch (e) {
    window.toast?.(e.message, 'err');
  }
}

async function submitS3Import() {
  const bucket = gk$('#importS3Bucket')?.value?.trim();
  const key = gk$('#importS3Key')?.value?.trim();
  const endpoint = gk$('#importS3Endpoint')?.value?.trim();
  if (!bucket || !key) return;
  try {
    const data = await window.api?.('/vms/import-from-s3', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ bucket, key, endpoint: endpoint || undefined }),
    });
    hideModal('importS3Modal');
    window.toast?.('S3 import started', 'ok');
    await window.loadFleet?.();
    if (data?.data?.id) window.selectVm?.(data.data);
  } catch (e) {
    window.toast?.(e.message, 'err');
  }
}

function injectLaunchAdvice(text) {
  const row = gk$('#launchAiAdviceRow');
  const cell = gk$('#launchAiAdvice');
  if (row && cell && text) {
    row.classList.remove('hidden');
    cell.textContent = text;
  }
}

function setupBrainAsk() {
  gk$('#brainAskForm')?.addEventListener('submit', (e) => {
    e.preventDefault();
    const q = gk$('#brainAskInput')?.value?.trim();
    if (q) window.GuestKitAi?.askBrain?.(q);
    gk$('#brainAskInput').value = '';
  });
  gk$('#brainExportReport')?.addEventListener('click', exportReadinessReport);
  gk$('#brainCompareBtn')?.addEventListener('click', showCompareMode);
  gk$('#brainQuickRepair')?.addEventListener('click', () => window.runAction?.('repair-plan'));
  gk$('#brainQuickYaml')?.addEventListener('click', () => showLaunchPreview(() => window.runAction?.('provision')));
  gk$('#brainQuickExplain')?.addEventListener('click', () => window.GuestKitAi?.askBrain?.('Explain the top boot risks for this VM'));
  gk$('#brainQuickPlan')?.addEventListener('click', () => window.runAction?.('migration-plan'));
}

async function askBrain(question) {
  return window.GuestKitAi?.askBrain?.(question);
}

async function exportReadinessReport() {
  const vm = window.state?.selectedVm;
  if (!vm) {
    window.toast?.('Select a disk first', 'err');
    return;
  }
  try {
    const res = await fetch(`${window.API_BASE || '/api/v1'}/vms/${vm.id}/readiness-report`, { method: 'POST' });
    const blob = await res.blob();
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${vm.name || vm.id}-readiness.pdf`;
    a.click();
    URL.revokeObjectURL(url);
    window.toast?.('Readiness report downloaded', 'ok');
  } catch (e) {
    window.toast?.(e.message, 'err');
  }
}

function showCompareMode() {
  window.GuestKitFeatures?.populateCompareSelects?.();
  showModal('compareModal');
}

function renderComparePane(vmId, label, diff, side) {
  const vm = window.state?.vms?.find((v) => v.id === vmId);
  const cache = window.getVmCache?.(vmId) || {};
  const inspect = cache.inspect || {};
  const os = inspect.os || {};
  const score = side === 'before'
    ? diff.before_boot_score
    : diff.after_boot_score;
  const blockers = side === 'before' ? diff.before_blockers : diff.after_blockers;
  const osLabel = side === 'before'
    ? (diff.before_os?.distribution || diff.before_os?.os_type || os.distribution || os.os_type || '—')
    : (diff.after_os?.distribution || diff.after_os?.os_type || os.distribution || os.os_type || '—');
  return `
    <div class="compare-pane">
      <h3>${label}: ${window.escapeHtml?.(vm?.name || vmId.slice(0, 8))}</h3>
      <dl>
        <dt>Boot score</dt><dd>${score != null ? Math.round(score) : (cache.bootScore != null ? Math.round(cache.bootScore) : '—')}</dd>
        <dt>Blockers</dt><dd>${blockers ?? '—'}</dd>
        <dt>OS</dt><dd>${window.escapeHtml?.(String(osLabel))}</dd>
        <dt>Format</dt><dd>${window.escapeHtml?.(vm?.format || '—')} · ${window.fmtBytes?.(vm?.size_bytes)}</dd>
        <dt>Last op</dt><dd>${window.escapeHtml?.(cache.lastOp || '—')}</dd>
      </dl>
    </div>`;
}

function showCompareWorkbench(beforeId, afterId, diff) {
  const panel = gk$('#panel-compare');
  const split = gk$('#compareSplitView');
  const bench = gk$('#compareWorkbenchResults');
  if (!panel || !split) return;
  panel.classList.remove('hidden');
  split.innerHTML = renderComparePane(beforeId, 'Before', diff, 'before')
    + renderComparePane(afterId, 'After', diff, 'after');
  if (bench) {
    const delta = diff.boot_score_delta ?? 0;
    const deltaCls = delta > 0 ? 'ready' : delta < 0 ? 'risk' : '';
    bench.innerHTML = `
      <div class="compare-summary">
        <div class="compare-card ${deltaCls}"><span>Score delta</span><strong>${delta > 0 ? '+' : ''}${Math.round(delta)}</strong></div>
        <div class="compare-card"><span>New warnings</span><strong>${diff.new_warnings ?? 0}</strong></div>
      </div>`;
  }
  window.scrollToPanel?.('assure');
  gk$('#compareSplitClose')?.addEventListener('click', () => panel.classList.add('hidden'), { once: true });
}

async function runCompare() {
  const before = gk$('#compareBeforeSelect')?.value;
  const after = gk$('#compareAfterSelect')?.value;
  if (!before || !after) return;
  try {
    const data = await window.api?.(`/vms/compare?before=${before}&after=${after}`, { method: 'POST' });
    const out = gk$('#compareResults');
    const diff = data?.data?.diff || data?.data || {};
    if (out) {
      const delta = diff.boot_score_delta ?? 0;
      const deltaCls = delta > 0 ? 'ready' : delta < 0 ? 'risk' : '';
      out.innerHTML = `
        <div class="compare-summary">
          <div class="compare-card"><span>Before score</span><strong>${Math.round(diff.before_boot_score ?? 0)}</strong></div>
          <div class="compare-card"><span>After score</span><strong>${Math.round(diff.after_boot_score ?? 0)}</strong></div>
          <div class="compare-card ${deltaCls}"><span>Delta</span><strong>${delta > 0 ? '+' : ''}${Math.round(delta)}</strong></div>
          <div class="compare-card"><span>Blockers</span><strong>${diff.before_blockers ?? 0} → ${diff.after_blockers ?? 0}</strong></div>
        </div>
        <pre class="mono">${window.escapeHtml?.(JSON.stringify(diff, null, 2))}</pre>`;
    }
    showCompareWorkbench(before, after, diff);
    const beforeVm = window.state?.vms?.find((v) => v.id === before);
    const afterVm = window.state?.vms?.find((v) => v.id === after);
    window.GuestKitAi?.aiCompareNarrative?.(
      beforeVm?.name || before.slice(0, 8),
      afterVm?.name || after.slice(0, 8),
      diff,
    );
    hideModal('compareModal');
  } catch (e) {
    window.toast?.(e.message, 'err');
  }
}

function setupIronwolfTheme() {
  document.documentElement.dataset.theme = 'ironwolf';
  localStorage.setItem('zyvor.theme', 'ironwolf');
}

function initGuestKitConsole() {
  setupIronwolfTheme();
  renderMissionRail();
  setupCommandDock();
  setupCinemaMode();
  setupBrainDrawer();
  setupNebulaExtras();
  setupRailCompact();
  setupCommandBar();
  setupImportPortal();
  setupBrainAsk();
  window.GuestKitAi?.initGuestKitAi?.();
  refreshHudStatus();
  setInterval(refreshHudStatus, 30000);
  gk$('#compareRunBtn')?.addEventListener('click', runCompare);
  gk$('#compareCancelBtn')?.addEventListener('click', () => hideModal('compareModal'));
}

function onSelectVmConsole() {
  renderBrainPanel();
  const vm = window.state?.selectedVm;
  const cache = vm ? window.getVmCache?.(vm.id) : {};
  window.GuestKitNebula?.renderAllNebula?.(vm, cache);
  if (window.innerWidth < 1200) gk$('#nebulaBrainDrawer')?.classList.add('open');
  window.GuestKitAi?.onSelectVmAi?.();
}

function onJobCompleteConsole(action, vmId) {
  const ev = mapTimelineEvent(action);
  if (ev && vmId) appendTimelineEvent(vmId, ev);
  if (action === 'doctor') {
    const payload = window.extractPayload?.({ data: window.state?.lastJobResult });
    const checks = payload?.bootability?.checks;
    if (checks && vmId) {
      const cache = window.getVmCache?.(vmId) || {};
      window.updateVmCache?.(vmId, { ...cache, checks });
    }
    window.GuestKitAi?.onDoctorComplete?.(vmId);
  }
  if (action === 'inspect' && vmId) {
    const cache = window.getVmCache?.(vmId) || {};
    /* inspect stored via onJobComplete patch in app.js */
  }
  renderMissionRail();
  renderBrainPanel();
  const vm = window.state?.selectedVm;
  const cache = vmId ? window.getVmCache?.(vmId) || {} : {};
  if (vm) {
    renderEvidenceConsole(vm, cache);
    window.GuestKitNebula?.renderAllNebula?.(vm, cache);
    if (['inspect', 'doctor', 'repair-plan', 'migration-plan', 'provision'].includes(action)) {
      setTimeout(() => scrollToEvidenceConsole(), 120);
    }
  }
  window.GuestKitFeatures?.loadJobHistory?.(vmId);
}

window.GuestKitConsole = {
  initGuestKitConsole,
  renderMissionRail,
  renderBrainPanel,
  refreshHudStatus,
  renderFleetDiskCard,
  bindDiskCardActions,
  onSelectVmConsole,
  onJobCompleteConsole,
  showLaunchPreview,
  showLaunchMonitor,
  showConvertStudio,
  showCompareMode,
  injectLaunchAdvice,
  appendTimelineEvent,
  resetJobBadge,
  appendEvidenceLog,
  clearEvidenceLogs,
  renderDiskInspector,
  updateSelectedCommandBar,
  renderEvidenceConsole,
  clearDiskSelectionUi,
  clearDiskSelection,
  scrollToDiskContext,
  scrollToEvidenceConsole,
  setReadinessRing,
  parseLaunchSpec,
};
