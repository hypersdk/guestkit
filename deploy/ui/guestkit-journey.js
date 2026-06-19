/* GuestKit Journey — pipeline state, next action, confidence (Nebula uses state helpers only) */

const JOURNEY_STEPS = [
  { id: 'capture', label: 'Imported', key: 'imported' },
  { id: 'fingerprint', label: 'Fingerprinted', key: 'inspect' },
  { id: 'diagnose', label: 'Diagnosed', key: 'doctor' },
  { id: 'repair', label: 'Repaired', key: 'repair' },
  { id: 'yaml', label: 'YAML Ready', key: 'yaml' },
  { id: 'launch', label: 'Launched', key: 'launch' },
];

const MISSION_STATUS = {
  DONE: 'done', READY: 'ready', RUNNING: 'running', WARNING: 'warning',
  BLOCKED: 'blocked', LOCKED: 'locked', FAILED: 'failed',
};

function j$(sel) { return document.querySelector(sel); }

function deriveFleetStats(vms) {
  const disks = (vms || []).filter((v) => window.isFleetDisk?.(v) ?? true);
  let unscanned = 0; let bootable = 0; let risky = 0; let failed = 0; let ready = 0;
  disks.forEach((d) => {
    const c = window.getVmCache?.(d.id) || {};
    if (!c.inspect) unscanned++;
    if (c.status === 'failed') failed++;
    if (c.bootScore != null && c.bootScore >= 70 && !(c.blockers?.length)) bootable++;
    if (c.bootScore != null && c.bootScore >= 85 && !(c.blockers?.length)) ready++;
    if (c.blockers?.length || (c.bootScore != null && c.bootScore < 60)) risky++;
  });
  return { total: disks.length, unscanned, bootable, risky, failed, ready };
}

function deriveConfidenceBreakdown(cache) {
  const score = cache?.bootScore;
  const checks = cache?.checks || [];
  const inspect = cache?.inspect || {};
  const failIds = checks.filter((c) => !c.passed).map((c) => (c.id || '').toLowerCase());
  const hasInspect = !!inspect.os || !!inspect.boot;
  const boot = score != null ? Math.round(score) : null;
  const fs = hasInspect ? (failIds.some((id) => /fstab|filesystem|fs/.test(id)) ? 60 : 95) : null;
  const drv = hasInspect ? (failIds.some((id) => /virtio|driver|module/.test(id)) ? 55 : 85) : null;
  const kv = cache?.migrateScore != null ? Math.round(cache.migrateScore)
    : (boot != null ? Math.max(40, boot - 10) : null);
  const ci = inspect.cloud_init?.present ? 90 : (hasInspect ? 40 : null);
  return { boot, fs, drv, kv, ci };
}

function deriveNextAction(vm, cache, systemStatus) {
  if (!vm) return { action: 'capture', label: 'Upload or select a disk', workflow: null, primary: 'ingest' };
  cache = cache || {};
  if (!cache.inspect) return { action: 'inspect', label: 'Run Fingerprint', workflow: 'inspect', primary: 'inspect' };
  if (cache.bootScore == null) return { action: 'doctor', label: 'Run Boot Doctor', workflow: 'doctor', primary: 'doctor' };
  if (cache.blockers?.length) return { action: 'repair-plan', label: 'Generate Repair Plan', workflow: 'repair-plan', primary: 'repair-plan' };
  if (!window.state?.lastYaml && cache.lastOp !== 'provision') {
    return { action: 'provision', label: 'Generate KubeVirt YAML', workflow: 'provision', primary: 'provision' };
  }
  const clusterOk = systemStatus?.kubevirt === 'healthy' || systemStatus?.kubevirt === 'ready';
  const cdiOk = systemStatus?.cdi === 'ready';
  if (!clusterOk || !cdiOk) {
    return { action: 'provision', label: 'Review Launch Readiness', workflow: 'provision', primary: 'provision', locked: 'Cluster not fully ready' };
  }
  return { action: 'provision', label: 'Launch VM', workflow: 'provision', primary: 'provision' };
}

function humanSummary(vm, cache) {
  if (!vm) return 'Select a disk from Image Vault to begin the recovery pipeline.';
  cache = cache || {};
  const inspect = cache.inspect;
  if (!inspect) {
    return `${vm.name || 'This disk'} is imported but not fingerprinted. Run Fingerprint to detect OS, bootloader, and KubeVirt readiness.`;
  }
  const os = inspect.os?.distribution || inspect.os?.os_type || 'unknown OS';
  const score = cache.bootScore;
  if (score == null) {
    return `GuestKit detected ${os}. Run Boot Doctor to assess boot confidence and migration risks.`;
  }
  const blockers = cache.blockers?.length || 0;
  if (blockers) {
    return `${os} disk scored ${Math.round(score)} with ${blockers} blocker(s). Repair is recommended before launch.`;
  }
  if (score >= 85) {
    return `${os} appears launch-ready (boot ${Math.round(score)}%). Generate KubeVirt YAML or launch when cluster checks pass.`;
  }
  return `${os} scored ${Math.round(score)} — launch possible with caution. Review risks or run repair first.`;
}

function explainDisabledAction(action, vm, cache, systemStatus) {
  if (!vm) return 'Select a disk first.';
  cache = cache || {};
  switch (action) {
    case 'inspect': return '';
    case 'doctor': return cache.inspect ? '' : 'Run Fingerprint first.';
    case 'repair-plan': return cache.bootScore != null ? '' : 'Run Boot Doctor first.';
    case 'migration-plan': return cache.bootScore != null ? '' : 'Run Boot Doctor first.';
    case 'provision':
    case 'launch':
      if (!cache.inspect) return 'Disk has not been fingerprinted.';
      if (cache.bootScore == null) return 'Run Boot Doctor before launch.';
      if (systemStatus?.kubevirt !== 'healthy' && systemStatus?.kubevirt !== 'ready') return 'KubeVirt cluster is not ready.';
      if (systemStatus?.cdi !== 'ready') return 'CDI is not ready for disk import.';
      return '';
    default: return '';
  }
}

function mapJobToStep(job) {
  const m = { inspect: 'fingerprint', doctor: 'diagnose', 'repair-plan': 'repair', 'migration-plan': 'yaml', provision: 'launch' };
  return m[job] || null;
}

function deriveJourneyState(vm, cache, fleet, systemStatus) {
  const fleetStats = deriveFleetStats(fleet || window.state?.vms);
  const confidence = deriveConfidenceBreakdown(cache || {});
  const next = deriveNextAction(vm, cache, systemStatus);
  const summary = humanSummary(vm, cache);
  const steps = JOURNEY_STEPS.map((s) => {
    let status = MISSION_STATUS.LOCKED;
    if (!vm && s.id !== 'capture') status = MISSION_STATUS.LOCKED;
    else if (s.id === 'capture') status = vm ? MISSION_STATUS.DONE : MISSION_STATUS.READY;
    else if (s.id === 'fingerprint') status = cache?.inspect ? MISSION_STATUS.DONE : (vm ? MISSION_STATUS.READY : MISSION_STATUS.LOCKED);
    else if (s.id === 'diagnose') {
      if (cache?.bootScore != null) status = cache.blockers?.length ? MISSION_STATUS.WARNING : MISSION_STATUS.DONE;
      else status = cache?.inspect ? MISSION_STATUS.READY : MISSION_STATUS.LOCKED;
    } else if (s.id === 'repair') {
      if (cache?.lastOp === 'repair-plan') status = MISSION_STATUS.DONE;
      else if (cache?.blockers?.length) status = MISSION_STATUS.WARNING;
      else status = cache?.bootScore != null ? MISSION_STATUS.READY : MISSION_STATUS.LOCKED;
    } else if (s.id === 'yaml') {
      status = window.state?.lastYaml ? MISSION_STATUS.DONE : (cache?.bootScore != null ? MISSION_STATUS.READY : MISSION_STATUS.LOCKED);
    } else if (s.id === 'launch') {
      status = cache?.lastOp === 'provision' ? MISSION_STATUS.DONE : (window.state?.lastYaml ? MISSION_STATUS.READY : MISSION_STATUS.LOCKED);
    }
    if (window.state?.activeJob && s.id === mapJobToStep(window.state.activeJob)) status = MISSION_STATUS.RUNNING;
    return { ...s, status };
  });
  const launchLock = explainDisabledAction('launch', vm, cache, systemStatus);
  const brainConfidence = !vm ? 'low' : (!cache?.inspect ? 'low' : (cache.bootScore == null ? 'medium' : (cache.blockers?.length ? 'medium' : 'high')));
  const brainReason = !vm ? 'No disk selected' : (!cache?.inspect ? 'Disk not fingerprinted' : (cache.bootScore == null ? 'Boot score pending' : 'Based on scan + doctor checks'));
  return { fleetStats, confidence, next, summary, steps, launchLock, brainConfidence, brainReason };
}

function fleetMissionHint(stepId, fleetStats) {
  switch (stepId) {
    case 'capture': return fleetStats.total ? `${fleetStats.total} imported` : 'No disks';
    case 'fingerprint': return fleetStats.unscanned ? `${fleetStats.unscanned} unscanned` : 'All scanned';
    case 'diagnose': return fleetStats.unscanned ? 'Boot risk unknown' : `${fleetStats.bootable} bootable`;
    case 'repair': return fleetStats.risky ? `${fleetStats.risky} need repair` : 'No repairs queued';
    case 'convert': return 'Waiting for selection';
    case 'launch': return fleetStats.ready ? `${fleetStats.ready} launch-ready` : 'Readiness required';
    default: return '';
  }
}

function renderFindingsTab(cache) {
  const el = j$('#findingsContent');
  const empty = j$('#findingsEmpty');
  if (!el) return;
  const checks = cache?.checks || [];
  const blockers = cache?.blockers || [];
  if (!checks.length && !blockers.length) {
    empty?.classList.remove('hidden');
    el.classList.add('hidden');
    return;
  }
  empty?.classList.add('hidden');
  el.classList.remove('hidden');
  const critical = blockers.map((b) => typeof b === 'string' ? b : (b.message || b.id));
  const warnings = checks.filter((c) => !c.passed).map((c) => c.message || c.id);
  const info = checks.filter((c) => c.passed).slice(0, 8).map((c) => c.message || c.id);
  el.innerHTML = [
    critical.length ? `<h4>Critical</h4>${critical.map((m) => `<p class="risk-item fail">${window.escapeHtml?.(m)}</p>`).join('')}` : '',
    warnings.length ? `<h4>Warnings</h4>${warnings.map((m) => `<p class="risk-item">${window.escapeHtml?.(m)}</p>`).join('')}` : '',
    info.length ? `<h4>Info</h4>${info.map((m) => `<p class="risk-item ok">✓ ${window.escapeHtml?.(m)}</p>`).join('')}` : '',
  ].join('');
}

function renderDiffTab(cache) {
  const el = j$('#diffContent');
  const empty = j$('#diffEmpty');
  if (!el) return;
  const plan = cache?.repairPlan;
  const before = plan?.before_score ?? cache?.bootScore;
  const after = plan?.after_score;
  if (!plan && after == null) {
    empty?.classList.remove('hidden');
    el.classList.add('hidden');
    return;
  }
  empty?.classList.add('hidden');
  el.classList.remove('hidden');
  el.innerHTML = `<div class="diff-grid">
    <div><h4>Before</h4><p>Boot score: ${before ?? '—'}</p><p>Blockers: ${cache?.blockers?.length || 0}</p></div>
    <div><h4>After</h4><p>Boot score: ${after ?? '—'}</p><p>Repair plan applied</p></div>
  </div><pre class="mono">${window.escapeHtml?.(JSON.stringify(plan || {}, null, 2).slice(0, 3000))}</pre>`;
}

function renderClusterReadinessDrawer(status) {
  const el = j$('#clusterReadinessBody');
  if (!el) return;
  status = status || {};
  const rows = [
    ['Agent', status.agent || '—', status.agent === 'online'],
    ['Cluster', status.cluster || '—', status.cluster === 'ready'],
    ['KubeVirt', status.kubevirt || '—', status.kubevirt === 'healthy' || status.kubevirt === 'ready'],
    ['CDI', status.cdi || '—', status.cdi === 'ready'],
    ['Storage', status.storage || '—', true],
    ['Worker', status.worker || '—', status.worker === 'online'],
    ['Disks', status.disk_count != null ? String(status.disk_count) : '—', true],
  ];
  el.innerHTML = rows.map(([k, v, ok]) =>
    `<div class="readiness-row ${ok ? 'ok' : 'warn'}"><span>${k}</span><strong>${window.escapeHtml?.(v)}</strong></div>`
  ).join('');
}

function renderLaunchMonitor(stages) {
  const el = j$('#launchMonitorBody');
  if (!el) return;
  const defaults = [
    { id: 'create', label: 'Create VM', status: 'waiting' },
    { id: 'import', label: 'Import disk', status: 'waiting' },
    { id: 'schedule', label: 'Schedule pod', status: 'waiting' },
    { id: 'boot', label: 'Boot guest', status: 'waiting' },
    { id: 'agent', label: 'Guest agent', status: 'waiting' },
    { id: 'console', label: 'Console ready', status: 'waiting' },
  ];
  const list = stages || defaults;
  el.innerHTML = `<ul class="launch-monitor">${list.map((s) =>
    `<li class="launch-stage ${s.status}"><span class="launch-stage__dot"></span>${s.label}</li>`
  ).join('')}</ul>`;
}

function renderFleetSummaryBar(stats) {
  const el = j$('#fleetVaultSummary');
  if (!el) return;
  el.innerHTML = `<span><strong>${stats.total}</strong> imported</span>
    <span>${stats.unscanned} unscanned</span>
    <span>${stats.bootable} bootable</span>
    <span>${stats.risky} risky</span>
    <span>${stats.failed} failed</span>`;
}

function copyDebugBundle() {
  const vm = window.state?.selectedVm;
  const cache = vm ? window.getVmCache?.(vm.id) : {};
  const bundle = {
    at: new Date().toISOString(),
    vm, cache, systemStatus: window.state?.systemStatus,
    lastJob: window.state?.lastJobResult, journey: window.state?.journeyState,
  };
  navigator.clipboard.writeText(JSON.stringify(bundle, null, 2));
  window.toast?.('Debug bundle copied', 'ok');
}

function checkSessionRestore() {
  try {
    const raw = localStorage.getItem('zyvor.sessionRestore');
    if (!raw) return;
    const s = JSON.parse(raw);
    if (!s?.vmId || Date.now() - s.at > 86400000) return;
    const modal = j$('#sessionRestoreModal');
    if (!modal) return;
    j$('#sessionRestoreText').textContent = `Continue with ${s.vmName || s.vmId}? Last: ${s.lastAction || '—'}`;
    modal.classList.remove('hidden');
    j$('#sessionRestoreResume')?.addEventListener('click', () => {
      const vm = window.state?.vms?.find((v) => v.id === s.vmId);
      if (vm) window.selectVm?.(vm);
      modal.classList.add('hidden');
    }, { once: true });
    j$('#sessionRestoreFresh')?.addEventListener('click', () => {
      localStorage.removeItem('zyvor.sessionRestore');
      modal.classList.add('hidden');
    }, { once: true });
  } catch { /* */ }
}

function saveSessionRestore(vm, action) {
  if (!vm) return;
  localStorage.setItem('zyvor.sessionRestore', JSON.stringify({
    vmId: vm.id, vmName: vm.name, lastAction: action, at: Date.now(),
  }));
}

window.GuestKitJourney = {
  deriveJourneyState,
  deriveConfidenceBreakdown,
  deriveFleetStats,
  deriveNextAction,
  humanSummary,
  explainDisabledAction,
  fleetMissionHint,
  renderFindingsTab,
  renderDiffTab,
  renderClusterReadinessDrawer,
  renderLaunchMonitor,
  renderFleetSummaryBar,
  copyDebugBundle,
  saveSessionRestore,
  checkSessionRestore,
  MISSION_STATUS,
};
