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

function renderMissionRail() {
  const nav = gk$('#missionRailNav');
  if (!nav) return;
  const vm = window.state?.selectedVm;
  const cache = vm ? window.getVmCache?.(vm.id) || {} : {};
  const activeWizard = window.state?.wizard?.step || 'ingest';
  nav.innerHTML = MISSION_STEPS.map((s) => {
    const badge = missionBadgeForStep(s.id, vm, cache);
    const active = s.wizard === activeWizard || (activeWizard === 'assure' && ['fingerprint', 'diagnose'].includes(s.id) && s.wizard === 'assure');
    return `
      <button type="button" class="mission-step${active ? ' active' : ''}" data-mission="${s.id}" data-wizard="${s.wizard}" data-action="${s.action || ''}">
        <span class="mission-step__num">${s.num}</span>
        <span class="mission-step__body">
          <span class="mission-step__label">${s.label}</span>
          <span class="mission-step__hint">${s.hint}</span>
        </span>
        <span class="mission-badge ${badge.cls}">${badge.text}</span>
      </button>`;
  }).join('');

  nav.querySelectorAll('.mission-step').forEach((btn) => {
    btn.addEventListener('click', () => {
      const wizard = btn.dataset.wizard;
      const action = btn.dataset.action;
      const mission = btn.dataset.mission;
      if (mission === 'convert') {
        showConvertStudio();
        return;
      }
      if (wizard) window.scrollToPanel?.(wizard);
      if (action && vm && !btn.querySelector('.mission-badge.blocked')) {
        if (action === 'provision') showLaunchPreview(() => window.runAction?.('provision'));
        else window.runAction?.(action);
      }
    });
  });
}

function renderBrainPanel() {
  const vm = window.state?.selectedVm;
  const clusterVm = window.state?.selectedClusterVm;
  const cache = vm ? window.getVmCache?.(vm.id) || {} : {};
  const inspect = cache.inspect || window.state?.lastClusterInspect?.inspect;
  const briefing = window.state?.lastBriefing || window.state?.lastClusterBriefing;

  const dnaEl = gk$('#brainDna');
  if (dnaEl) {
    if (!vm && !clusterVm) {
      dnaEl.innerHTML = '<p class="brain-rec">Select a disk or cluster VM to view Disk DNA.</p>';
    } else if (inspect) {
      const os = inspect.os || {};
      const boot = inspect.boot || {};
      dnaEl.innerHTML = `
        <dl>
          <div class="brain-dna-row"><dt>OS</dt><dd>${window.escapeHtml?.(os.distribution || os.os_type || 'unknown') || 'unknown'}</dd></div>
          <div class="brain-dna-row"><dt>Boot mode</dt><dd>${window.escapeHtml?.(boot.mode || boot.boot_mode || '—') || '—'}</dd></div>
          <div class="brain-dna-row"><dt>Kernel</dt><dd>${window.escapeHtml?.(boot.kernel || inspect.kernel?.version || '—') || '—'}</dd></div>
          <div class="brain-dna-row"><dt>Filesystem</dt><dd>${window.escapeHtml?.(inspect.filesystem?.root_fs || inspect.storage?.root || '—') || '—'}</dd></div>
          <div class="brain-dna-row"><dt>Cloud-init</dt><dd>${inspect.cloud_init?.present ? 'present' : 'none'}</dd></div>
          <div class="brain-dna-row"><dt>Agent</dt><dd>${clusterVm ? (window.state?.lastClusterGuestInfo?.agent_connected ? 'connected' : 'missing') : 'offline disk'}</dd></div>
        </dl>`;
    } else {
      dnaEl.innerHTML = '<p class="brain-rec">Run <strong>Fingerprint</strong> to populate Disk DNA.</p>';
    }
  }

  const confEl = gk$('#brainConfidence');
  if (confEl) {
    const score = cache.bootScore;
    const checks = cache.checks || [];
    const topWarns = checks.filter((c) => !c.passed).slice(0, 4);
    if (score != null) {
      confEl.innerHTML = `
        <div class="brain-confidence">
          <div class="brain-confidence__ring">${Math.round(score)}</div>
          <div>
            ${topWarns.length ? topWarns.map((w) => `<p class="brain-rec">${window.escapeHtml?.(w.id || w.message || 'check failed')}</p>`).join('') : '<p class="brain-rec">No critical boot warnings.</p>'}
          </div>
        </div>`;
    } else {
      confEl.innerHTML = '<p class="brain-rec">Run <strong>Diagnose</strong> for boot confidence score.</p>';
    }
  }

  const recEl = gk$('#brainRecs');
  if (recEl) {
    const actions = briefing?.recommended_actions || cache?.briefing?.recommended_actions || [];
    if (actions.length) {
      recEl.innerHTML = actions.slice(0, 5).map((a) =>
        `<button type="button" class="brain-rec-action" data-workflow="${window.escapeHtml?.(a.workflow || '')}">
          <strong>#${a.priority}</strong> ${window.escapeHtml?.(a.title) || ''} — ${window.escapeHtml?.(a.detail) || ''}
        </button>`
      ).join('');
      recEl.querySelectorAll('.brain-rec-action').forEach((btn) => {
        btn.addEventListener('click', () => {
          const wf = btn.dataset.workflow;
          if (wf === 'provision') showLaunchPreview(() => window.runAction?.('provision'));
          else if (wf) window.runAction?.(wf.replace('guestkit.', '').replace('migrate-plan', 'migration-plan'));
        });
      });
    } else if (briefing?.headline || cache?.briefing?.headline) {
      const b = briefing || cache.briefing;
      recEl.innerHTML = `<p class="brain-rec"><strong>${window.escapeHtml?.(b.headline)}</strong><br>${window.escapeHtml?.(b.summary || '')}</p>`;
    } else {
      recEl.innerHTML = '<p class="brain-rec">Run Doctor with explain for AI recommendations.</p>';
    }
  }

  renderBrainChecksHeatmap(cache?.checks);
  window.GuestKitAi?.renderAiDeck?.();
  window.GuestKitAi?.renderAiNarrative?.();
  window.GuestKitFeatures?.loadJobHistory?.(vm?.id);

  renderBrainTimeline(vm?.id || (clusterVm ? `cluster:${clusterVm.namespace}/${clusterVm.name}` : null));
  syncBrainJobTracker();
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
  if (active) {
    tracker.className = 'brain-section';
    tracker.innerHTML = `<h3 class="brain-section__title cyan">Live Job</h3>${legacy.innerHTML}`;
  }
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

function renderFleetDiskCard(vm, selected, cache) {
  const smoke = window.isSmokeDisk?.(vm);
  const risk = riskLevel(cache);
  const status = window.vmStatusLabel?.(cache, vm) || 'unknown';
  const icon = osIcon(vm, cache);
  const scanned = cache?.lastOp ? cache.lastOp : 'never';
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
      ${cache?.bootScore != null ? `<span class="disk-tag ready">boot ${Math.round(cache.bootScore)}</span>` : ''}
      ${smoke ? '<span class="disk-tag warn">smoke</span>' : ''}
    </div>
    <div class="disk-card__actions">
      <button type="button" class="btn sm glass" data-disk-action="inspect">Inspect</button>
      <button type="button" class="btn sm glass" data-disk-action="doctor">Doctor</button>
      <button type="button" class="btn sm glass" data-disk-action="migration-plan">Plan</button>
      <button type="button" class="btn sm primary" data-disk-action="provision">Launch</button>
    </div>
    <p class="disk-card__meta">Last scan: ${scanned}</p>`;
}

function bindDiskCardActions(card, vm) {
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
  if (!dock) return;
  dock.classList.remove('pos-bottom', 'pos-left', 'pos-right', 'compact', 'auto-hide', 'pinned');
  dock.classList.add(`pos-${prefs.position || 'bottom'}`);
  if (prefs.compact) dock.classList.add('compact');
  if (prefs.autoHide) dock.classList.add('auto-hide');
  if (prefs.pinned) dock.classList.add('pinned');
  const offset = prefs.position === 'left' || prefs.position === 'right' ? '24px' : `${(prefs.compact ? 56 : 72) + 16}px`;
  document.documentElement.style.setProperty('--iw-dock-offset', offset);
}

function setupCommandDock() {
  const dock = gk$('#commandDock');
  if (!dock) return;
  applyDockPrefs();

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
    window.scrollToPanel?.('launch');
    window.setActiveTab?.('raw');
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

  const set = (id, val, ok) => {
    const el = gk$(`#${id}`);
    if (!el) return;
    el.textContent = val || '—';
    const dot = el.parentElement?.querySelector('.status-strip__dot');
    if (dot) {
      dot.className = 'status-strip__dot ' + (ok === true ? 'ok' : ok === false ? 'err' : ok === 'ai' ? 'ai' : '');
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
  const tbody = gk$('#launchPreviewBody');
  if (tbody && vm) {
    tbody.innerHTML = `
      <tr><td>VM name</td><td>${window.escapeHtml?.(spec.vmName)}</td></tr>
      <tr><td>Namespace</td><td>${window.escapeHtml?.(window.state?.uiConfig?.default_namespace || 'default')}</td></tr>
      <tr><td>CPU / Memory</td><td>${spec.cpu} vCPU · ${window.escapeHtml?.(spec.mem)}</td></tr>
      <tr><td>Disk</td><td>${window.fmtBytes?.(vm.size_bytes)} ${vm.format}</td></tr>
      <tr><td>Boot score</td><td>${cache?.bootScore != null ? Math.round(cache.bootScore) : '—'}</td></tr>
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
  renderBrainPanel();
  setupCommandDock();
  setupImportPortal();
  setupBrainAsk();
  window.GuestKitAi?.initGuestKitAi?.();
  refreshHudStatus();
  setInterval(refreshHudStatus, 30000);
  gk$('#compareRunBtn')?.addEventListener('click', runCompare);
  gk$('#compareCancelBtn')?.addEventListener('click', () => hideModal('compareModal'));
}

function onSelectVmConsole() {
  renderMissionRail();
  renderBrainPanel();
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
  showConvertStudio,
  showCompareMode,
  injectLaunchAdvice,
  appendTimelineEvent,
};
