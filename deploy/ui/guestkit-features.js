/* GuestKit — extended features pack */

const CACHE_KEY = 'zyvor.vmCache.v1';
const RECENT_KEY = 'zyvor.recentDisks';
const PREFS_KEY = 'zyvor.guestkitPrefs';

function loadVmCachePersist() {
  try {
    const raw = localStorage.getItem(CACHE_KEY);
    if (raw) Object.assign(window.state.vmCache, JSON.parse(raw));
  } catch { /* */ }
}

function saveVmCachePersist() {
  try {
    localStorage.setItem(CACHE_KEY, JSON.stringify(window.state.vmCache));
  } catch { /* */ }
}

function getPrefs() {
  try {
    const raw = localStorage.getItem(PREFS_KEY) || localStorage.getItem('zyvor.nebulaPrefs');
    return JSON.parse(raw || '{}');
  } catch { return {}; }
}

function savePrefs(p) {
  localStorage.setItem(PREFS_KEY, JSON.stringify({ ...getPrefs(), ...p }));
}

function pushRecent(vm) {
  if (!vm?.id) return;
  let recent = [];
  try { recent = JSON.parse(localStorage.getItem(RECENT_KEY) || '[]'); } catch { /* */ }
  recent = [{ id: vm.id, name: vm.name, at: Date.now() }, ...recent.filter((r) => r.id !== vm.id)].slice(0, 8);
  localStorage.setItem(RECENT_KEY, JSON.stringify(recent));
  renderRecentDisks();
}

function renderRecentDisks() {
  const el = document.getElementById('recentDisksMenu');
  if (!el) return;
  let recent = [];
  try { recent = JSON.parse(localStorage.getItem(RECENT_KEY) || '[]'); } catch { /* */ }
  if (!recent.length) {
    el.innerHTML = '<span class="recent-empty">No recent disks</span>';
    return;
  }
  el.innerHTML = recent.map((r) => {
    const vm = window.state?.vms?.find((v) => v.id === r.id);
    if (!vm) return '';
    return `<button type="button" class="recent-disk-btn" data-id="${r.id}">${window.escapeHtml?.(r.name) || r.id.slice(0, 8)}</button>`;
  }).join('');
  el.querySelectorAll('.recent-disk-btn').forEach((btn) => {
    btn.addEventListener('click', () => {
      const vm = window.state.vms.find((v) => v.id === btn.dataset.id);
      if (vm) window.selectVm?.(vm);
    });
  });
}

function missionProgress(vm, cache) {
  if (!vm) return 0;
  let n = 1;
  if (cache?.inspect) n++;
  if (cache?.bootScore != null) n++;
  if (cache?.lastOp === 'repair-plan') n++;
  if (cache?.lastOp === 'convert' || cache?.migrateScore != null) n++;
  if (cache?.lastOp === 'provision') n++;
  return Math.min(n, 6);
}

function renderMissionProgress() {
  const el = document.getElementById('missionProgress');
  if (!el) return;
  const vm = window.state?.selectedVm;
  const cache = vm ? window.getVmCache?.(vm.id) : {};
  const n = missionProgress(vm, cache);
  el.innerHTML = `<div class="mission-progress"><div class="mission-progress__fill" style="width:${(n / 6) * 100}%"></div></div><span class="mission-progress__label">${n}/6 mission steps</span>`;
}

function filterFleetVms(vms) {
  const q = (document.getElementById('fleetDiskSearch')?.value || '').trim().toLowerCase();
  const sort = document.getElementById('fleetDiskSort')?.value || 'rank';
  const chip = document.querySelector('.filter-chip.active')?.dataset.filter || '';
  const saved = document.getElementById('fleetSavedView')?.value || '';
  let list = (vms || []).filter((v) => window.isFleetDisk?.(v) ?? true);
  if (q) {
    list = list.filter((v) =>
      (v.name || '').toLowerCase().includes(q)
      || (v.format || '').toLowerCase().includes(q)
      || v.id.toLowerCase().includes(q)
      || (q.startsWith('os:') && (window.getVmCache?.(v.id)?.inspect?.os?.distribution || '').toLowerCase().includes(q.slice(3)))
      || (q.startsWith('risk:high') && ((window.getVmCache?.(v.id)?.blockers?.length) || (window.getVmCache?.(v.id)?.bootScore != null && window.getVmCache?.(v.id).bootScore < 60))));
  }
  const filterKey = chip || saved;
  if (filterKey) {
    list = list.filter((v) => {
      const c = window.getVmCache?.(v.id) || {};
      const os = (c.inspect?.os?.distribution || c.inspect?.os?.os_type || '').toLowerCase();
      switch (filterKey) {
        case 'unscanned': return !c.inspect;
        case 'bootable': return c.bootScore != null && c.bootScore >= 70 && !(c.blockers?.length);
        case 'repair': return (c.blockers?.length) || (c.bootScore != null && c.bootScore < 60);
        case 'ready': return c.bootScore != null && c.bootScore >= 85 && !(c.blockers?.length);
        case 'linux': return !!c.inspect && !/windows|win/.test(os);
        case 'windows': return /windows|win/.test(os);
        case 'failed': return c.status === 'failed';
        default: return true;
      }
    });
  }
  if (sort === 'name') list.sort((a, b) => (a.name || '').localeCompare(b.name || ''));
  else if (sort === 'size') list.sort((a, b) => (b.size_bytes || 0) - (a.size_bytes || 0));
  else if (sort === 'score') {
    list.sort((a, b) => {
      const sa = window.getVmCache?.(a.id)?.bootScore ?? -1;
      const sb = window.getVmCache?.(b.id)?.bootScore ?? -1;
      return sb - sa;
    });
  }
  return list;
}

const fleetBatch = new Set();
let fleetViewMode = localStorage.getItem('zyvor.fleetView') || 'card';

function setFleetViewMode(mode) {
  fleetViewMode = mode;
  localStorage.setItem('zyvor.fleetView', mode);
  document.getElementById('fleetGrid')?.classList.toggle('hidden', mode === 'table');
  document.getElementById('fleetTable')?.classList.toggle('hidden', mode !== 'table');
  document.getElementById('fleetViewCard')?.classList.toggle('active', mode === 'card');
  document.getElementById('fleetViewTable')?.classList.toggle('active', mode === 'table');
}

function updateFleetBatchBar() {
  const bar = document.getElementById('fleetBatchBar');
  const count = document.getElementById('fleetBatchCount');
  if (!bar) return;
  if (fleetBatch.size) {
    bar.classList.remove('hidden');
    if (count) count.textContent = `${fleetBatch.size} selected`;
  } else {
    bar.classList.add('hidden');
  }
}

function renderFleetTable() {
  const tbody = document.getElementById('fleetTableBody');
  if (!tbody || fleetViewMode !== 'table') return;
  const vms = filterFleetVms(window.state?.vms || []);
  tbody.innerHTML = vms.map((vm) => {
    const c = window.getVmCache?.(vm.id) || {};
    const os = c.inspect?.os?.distribution || c.inspect?.os?.os_type || '—';
    const risk = c.blockers?.length ? 'high' : (c.bootScore != null && c.bootScore < 70 ? 'caution' : 'low');
    const checked = fleetBatch.has(vm.id) ? 'checked' : '';
    return `<tr data-vm-id="${vm.id}">
      <td><input type="checkbox" class="fleet-row-check" data-id="${vm.id}" ${checked} /></td>
      <td>${window.escapeHtml?.(vm.name) || vm.name}</td>
      <td>${window.escapeHtml?.(os)}</td>
      <td>${vm.format || '—'}</td>
      <td>${window.fmtBytes?.(vm.size_bytes)}</td>
      <td>${c.bootScore != null ? Math.round(c.bootScore) : '—'}</td>
      <td>${c.migrateScore != null ? Math.round(c.migrateScore) : '—'}</td>
      <td>${risk}</td>
      <td>${c.lastOp || 'never'}</td>
      <td><button type="button" class="btn sm ghost fleet-row-open" data-id="${vm.id}">Open</button></td>
    </tr>`;
  }).join('');
  tbody.querySelectorAll('.fleet-row-check').forEach((cb) => {
    cb.addEventListener('change', () => {
      if (cb.checked) fleetBatch.add(cb.dataset.id);
      else fleetBatch.delete(cb.dataset.id);
      updateFleetBatchBar();
    });
  });
  tbody.querySelectorAll('.fleet-row-open').forEach((btn) => {
    btn.addEventListener('click', () => {
      const vm = window.state?.vms?.find((v) => v.id === btn.dataset.id);
      if (vm) window.selectVm?.(vm);
    });
  });
}

async function batchRunWorkflow(action) {
  const ids = [...fleetBatch];
  if (!ids.length) { window.toast?.('Select disks first', 'err'); return; }
  for (const id of ids) {
    const vm = window.state?.vms?.find((v) => v.id === id);
    if (!vm) continue;
    window.selectVm?.(vm);
    const r = await window.runAction?.(action);
    if (!r?.ok) break;
  }
  fleetBatch.clear();
  updateFleetBatchBar();
  window.renderFleet?.();
}

function exportFleetReport(fmt) {
  const vms = filterFleetVms(window.state?.vms || []);
  const rows = vms.map((vm) => {
    const c = window.getVmCache?.(vm.id) || {};
    return {
      id: vm.id, name: vm.name, format: vm.format, size: vm.size_bytes,
      boot: c.bootScore, kv: c.migrateScore, os: c.inspect?.os?.distribution,
      status: c.status, lastOp: c.lastOp, blockers: c.blockers?.length || 0,
    };
  });
  let blob; let name;
  if (fmt === 'csv') {
    const hdr = Object.keys(rows[0] || { id: '', name: '' }).join(',');
    const body = rows.map((r) => Object.values(r).join(',')).join('\n');
    blob = new Blob([`${hdr}\n${body}`], { type: 'text/csv' });
    name = 'guestkit-fleet.csv';
  } else if (fmt === 'md') {
    const md = ['# GuestKit Fleet Report', '', ...rows.map((r) => `- **${r.name}** boot=${r.boot ?? '—'} blockers=${r.blockers}`)].join('\n');
    blob = new Blob([md], { type: 'text/markdown' });
    name = 'guestkit-fleet.md';
  } else {
    blob = new Blob([JSON.stringify(rows, null, 2)], { type: 'application/json' });
    name = 'guestkit-fleet.json';
  }
  const a = document.createElement('a');
  a.href = URL.createObjectURL(blob);
  a.download = name;
  a.click();
}

async function runFleetScanModal() {
  const modal = document.getElementById('fleetScanModal');
  const status = document.getElementById('fleetScanStatus');
  const stages = document.getElementById('fleetScanStages');
  if (!modal) return;
  modal.classList.remove('hidden');
  const steps = ['Import check', 'Partition scan', 'OS detection', 'Bootloader scan', 'Driver scan', 'KubeVirt readiness'];
  let i = 0;
  const tick = () => {
    if (i >= steps.length) {
      if (status) status.textContent = 'Fleet scan complete';
      return;
    }
    if (status) status.textContent = steps[i];
    if (stages) {
      stages.innerHTML = steps.map((s, idx) =>
        `<li class="${idx < i ? 'done' : idx === i ? 'running' : ''}">${s}</li>`
      ).join('');
    }
    i += 1;
    setTimeout(tick, 800);
  };
  tick();
  document.getElementById('fleetScanClose')?.addEventListener('click', () => modal.classList.add('hidden'), { once: true });
}

function setupFleetAdvanced() {
  document.querySelectorAll('.filter-chip').forEach((chip) => {
    chip.addEventListener('click', () => {
      document.querySelectorAll('.filter-chip').forEach((c) => c.classList.remove('active'));
      chip.classList.add('active');
      window.renderFleet?.();
    });
  });
  document.getElementById('fleetSavedView')?.addEventListener('change', (e) => {
    const v = e.target.value;
    const map = { all: '', repair: 'repair', ready: 'ready' };
    if (map[v] != null) {
      document.querySelectorAll('.filter-chip').forEach((c) => {
        c.classList.toggle('active', c.dataset.filter === map[v]);
      });
    }
    window.renderFleet?.();
  });
  document.getElementById('fleetViewCard')?.addEventListener('click', () => { setFleetViewMode('card'); window.renderFleet?.(); });
  document.getElementById('fleetViewTable')?.addEventListener('click', () => { setFleetViewMode('table'); window.renderFleet?.(); });
  document.getElementById('fleetSelectAll')?.addEventListener('change', (e) => {
    const vms = filterFleetVms(window.state?.vms || []);
    fleetBatch.clear();
    if (e.target.checked) vms.forEach((v) => fleetBatch.add(v.id));
    updateFleetBatchBar();
    renderFleetTable();
  });
  document.getElementById('batchScanBtn')?.addEventListener('click', () => batchRunWorkflow('inspect'));
  document.getElementById('batchDoctorBtn')?.addEventListener('click', () => batchRunWorkflow('doctor'));
  document.getElementById('batchExportBtn')?.addEventListener('click', () => exportFleetReport('json'));
  document.getElementById('batchDeleteBtn')?.addEventListener('click', async () => {
    if (!fleetBatch.size || !confirm(`Delete ${fleetBatch.size} disk(s)?`)) return;
    for (const id of [...fleetBatch]) {
      try { await window.api?.(`/vms/${id}`, { method: 'DELETE' }); delete window.state.vmCache[id]; } catch { /* */ }
    }
    fleetBatch.clear();
    updateFleetBatchBar();
    await window.loadFleet?.();
  });
  setFleetViewMode(fleetViewMode);
}

async function runFullMigrationChain() {
  const vm = window.state?.selectedVm;
  if (!vm) {
    window.toast?.('Select a disk first', 'err');
    return;
  }
  window.toast?.('Starting full migration chain…', 'ok');
  for (const action of ['inspect', 'doctor', 'migration-plan']) {
    const r = await window.runAction?.(action);
    if (!r?.ok) {
      window.toast?.(`Chain stopped at ${action}`, 'err');
      return;
    }
  }
  window.GuestKitConsole?.showLaunchPreview?.(() => window.runAction?.('provision'));
}

async function deleteSelectedDisk() {
  const vm = window.state?.selectedVm;
  if (!vm) return;
  if (!confirm(`Remove "${vm.name}" from the vault?`)) return;
  try {
    await window.api?.(`/vms/${vm.id}`, { method: 'DELETE' });
    delete window.state.vmCache[vm.id];
    saveVmCachePersist();
    window.state.selectedVm = null;
    await window.loadFleet?.();
    window.toast?.('Disk removed', 'ok');
    window.GuestKitConsole?.renderMissionRail?.();
    window.GuestKitConsole?.renderBrainPanel?.();
  } catch (e) {
    window.toast?.(e.message, 'err');
  }
}

function notifyJobDone(action) {
  if (!getPrefs().notifications) return;
  if (typeof Notification !== 'undefined' && Notification.permission === 'granted') {
    new Notification('GuestKit', { body: `${action} completed` });
  }
}

function requestNotificationPermission() {
  if (typeof Notification !== 'undefined' && Notification.permission === 'default') {
    Notification.requestPermission();
  }
}

function showShortcutsHelp() {
  document.getElementById('shortcutsModal')?.classList.remove('hidden');
}

function setupKeyboardShortcuts() {
  document.addEventListener('keydown', (e) => {
    if (e.target.matches('input, textarea, select') && e.key !== 'Escape') return;
    if (e.key === '?' && !e.metaKey && !e.ctrlKey) {
      e.preventDefault();
      showShortcutsHelp();
      return;
    }
    if (e.key === 'Escape') {
      const modals = document.querySelectorAll('.guestkit-modal-backdrop:not(.hidden)');
      if (modals.length) {
        modals.forEach((m) => m.classList.add('hidden'));
        return;
      }
      document.getElementById('brainDrawer')?.classList.remove('open');
      if (window.GuestKitConsole?.clearDiskSelection?.()) return;
      return;
    }
    if (e.metaKey || e.ctrlKey) return;
    const vm = window.state?.selectedVm;
    const map = {
      i: () => vm && window.runAction?.('inspect'),
      d: () => vm && window.runAction?.('doctor'),
      r: () => vm && window.runAction?.('repair-plan'),
      m: () => vm && window.runAction?.('migration-plan'),
      l: () => vm && window.GuestKitConsole?.showLaunchPreview?.(() => window.runAction?.('provision')),
      u: () => document.getElementById('fileInput')?.click(),
      q: () => quickScan(),
      c: () => window.GuestKitConsole?.showCompareMode?.(),
      b: () => document.getElementById('brainDrawer')?.classList.toggle('open'),
      e: () => {
        window.scrollToPanel?.('assure');
        window.setActiveTab?.('timeline');
        window.GuestKitConsole?.scrollToEvidenceConsole?.();
      },
      '/': () => { e.preventDefault(); document.getElementById('fleetDiskSearch')?.focus(); },
    };
    if (map[e.key]) { e.preventDefault(); map[e.key](); return; }
    const step = ['1', '2', '3', '4', '5', '6'].indexOf(e.key);
    if (step >= 0) {
      document.querySelectorAll('.mission-step')[step]?.click();
    }
  });
}

function setupFleetToolbar() {
  document.getElementById('fleetDiskSearch')?.addEventListener('input', () => window.renderFleet?.());
  document.getElementById('fleetDiskSort')?.addEventListener('change', () => window.renderFleet?.());
  document.getElementById('fullMigrationBtn')?.addEventListener('click', runFullMigrationChain);
  document.getElementById('fleetAiOverviewBtn')?.addEventListener('click', () => {
    runFleetScanModal();
    window.GuestKitAi?.aiFleetOverview?.();
  });
  document.getElementById('deleteDiskBtn')?.addEventListener('click', deleteSelectedDisk);
  document.getElementById('copyDiskIdBtn')?.addEventListener('click', async () => {
    const id = window.state?.selectedVm?.id;
    if (!id) return;
    await navigator.clipboard.writeText(id);
    window.toast?.('Disk ID copied', 'ok');
  });
  document.getElementById('copyJsonBtn')?.addEventListener('click', async () => {
    const raw = document.getElementById('rawJson')?.textContent;
    if (raw) { await navigator.clipboard.writeText(raw); window.toast?.('JSON copied', 'ok'); }
  });
  document.getElementById('exportTimelineBtn')?.addEventListener('click', () => {
    const vm = window.state?.selectedVm;
    if (!vm) return;
    const key = `zyvor.timeline.${vm.id}`;
    const data = localStorage.getItem(key) || '[]';
    const blob = new Blob([data], { type: 'application/json' });
    const a = document.createElement('a');
    a.href = URL.createObjectURL(blob);
    a.download = `${vm.name || vm.id}-timeline.json`;
    a.click();
  });
  document.getElementById('openZeusBtn')?.addEventListener('click', () => {
    const url = window.state?.uiConfig?.zeus_url;
    if (url) window.open(url, '_blank', 'noopener');
    else window.toast?.('Zeus URL not configured', 'err');
  });
  document.getElementById('shortcutsCloseBtn')?.addEventListener('click', () => {
    document.getElementById('shortcutsModal')?.classList.add('hidden');
  });
  document.getElementById('prefsNotifications')?.addEventListener('change', (e) => {
    savePrefs({ notifications: e.target.checked });
    if (e.target.checked) requestNotificationPermission();
  });
  const notifEl = document.getElementById('prefsNotifications');
  const refreshEl = document.getElementById('prefsAutoRefresh');
  if (notifEl) notifEl.checked = Boolean(getPrefs().notifications);
  if (refreshEl) refreshEl.checked = getPrefs().autoRefresh !== false;
  refreshEl?.addEventListener('change', (e) => {
    savePrefs({ autoRefresh: e.target.checked });
    setupAutoRefresh();
  });
  document.getElementById('recentDisksToggle')?.addEventListener('click', () => {
    document.querySelector('.recent-disks-menu')?.classList.toggle('open');
  });
}

let autoRefreshTimer;
function setupAutoRefresh() {
  clearInterval(autoRefreshTimer);
  if (getPrefs().autoRefresh === false) return;
  autoRefreshTimer = setInterval(() => {
    if (window.state?.fleetMode === 'cluster') return;
    window.loadFleet?.();
  }, 60000);
}

async function quickScan() {
  const vm = window.state?.selectedVm;
  if (!vm) { window.toast?.('Select a disk first', 'err'); return; }
  window.toast?.('Quick scan: inspect + doctor…', 'ok');
  const ins = await window.runAction?.('inspect');
  if (!ins?.ok) return;
  await window.runAction?.('doctor');
}

async function applyYamlFromBrain() {
  if (!window.state?.lastYaml) {
    window.toast?.('Generate YAML first (Launch or Plan)', 'err');
    return;
  }
  try {
    await window.api?.('/kubevirt/apply', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ yaml: window.state.lastYaml }),
    });
    window.toast?.('YAML applied to cluster', 'ok');
  } catch (e) {
    window.toast?.(e.message, 'err');
  }
}

async function loadJobHistory(vmId) {
  const el = document.getElementById('brainJobHistory');
  if (!el || !vmId) return;
  try {
    const data = await window.api?.(`/vms/${vmId}/jobs`);
    const jobs = data?.data || [];
    if (!jobs.length) {
      el.innerHTML = '<p class="brain-rec">No jobs yet for this disk.</p>';
      return;
    }
    el.innerHTML = jobs.slice(0, 8).map((j) => {
      const t = j.completed_at || j.submitted_at;
      const time = t ? new Date(t).toLocaleString() : '—';
      const status = (j.status || 'pending').toLowerCase();
      return `<div class="brain-job"><span><span class="mono">${j.operation.replace('guestkit.', '')}</span> · ${time}</span><span class="job-status-pill ${status}">${status}</span></div>`;
    }).join('');
  } catch {
    el.innerHTML = '<p class="brain-rec">Job history unavailable.</p>';
  }
}

function populateCompareSelects() {
  const vms = (window.state?.vms || []).filter((v) => window.isFleetDisk?.(v));
  const opts = vms.map((v) => `<option value="${v.id}">${window.escapeHtml?.(v.name)}</option>`).join('');
  const before = document.getElementById('compareBeforeSelect');
  const after = document.getElementById('compareAfterSelect');
  if (before) before.innerHTML = opts;
  if (after) {
    after.innerHTML = opts;
    if (window.state?.selectedVm) after.value = window.state.selectedVm.id;
  }
}

async function cleanupShadowRows() {
  if (!confirm('Remove zero-byte and cluster-shadow disk rows from the vault?')) return;
  try {
    const data = await window.api?.('/vms/cleanup-shadows', { method: 'POST' });
    const n = data?.data?.deleted ?? 0;
    window.toast?.(`Removed ${n} shadow row${n === 1 ? '' : 's'}`, 'ok');
    await window.loadFleet?.();
    window.GuestKitConsole?.refreshHudStatus?.();
  } catch (e) {
    window.toast?.(e.message, 'err');
  }
}

async function submitNfsImport() {
  const path = document.getElementById('importNfsPath')?.value?.trim();
  const host = document.getElementById('importNfsHost')?.value?.trim();
  if (!path) return;
  try {
    const data = await window.api?.('/vms/import-from-nfs', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ path, host: host || undefined }),
    });
    document.getElementById('importNfsModal')?.classList.add('hidden');
    window.toast?.('NFS import complete', 'ok');
    await window.loadFleet?.();
    if (data?.data?.id) {
      const vm = data.data;
      window.selectVm?.(vm);
    }
  } catch (e) {
    window.toast?.(e.message, 'err');
  }
}

function initGuestKitFeatures() {
  loadVmCachePersist();
  setupFleetToolbar();
  setupFleetAdvanced();
  setupKeyboardShortcuts();
  setupAutoRefresh();
  renderRecentDisks();
  renderMissionProgress();

  document.getElementById('importNfsBtn')?.addEventListener('click', () => {
    document.getElementById('importNfsModal')?.classList.remove('hidden');
  });
  document.getElementById('importNfsCancel')?.addEventListener('click', () => {
    document.getElementById('importNfsModal')?.classList.add('hidden');
  });
  document.getElementById('importNfsSubmit')?.addEventListener('click', submitNfsImport);
  document.getElementById('quickScanBtn')?.addEventListener('click', quickScan);
  document.getElementById('applyYamlBrainBtn')?.addEventListener('click', applyYamlFromBrain);
  document.getElementById('cleanupShadowsBtn')?.addEventListener('click', cleanupShadowRows);
  document.getElementById('helpShortcutsBtn')?.addEventListener('click', showShortcutsHelp);

  const origUpdate = window.updateVmCache;
  if (origUpdate) {
    window.updateVmCache = (id, patch) => {
      origUpdate(id, patch);
      saveVmCachePersist();
      renderMissionProgress();
    };
  }

  const origSelect = window.selectVm;
  if (origSelect) {
    window.selectVm = (vm) => {
      origSelect(vm);
      pushRecent(vm);
      renderMissionProgress();
    };
  }

  const origComplete = window.GuestKitConsole?.onJobCompleteConsole;
  window.GuestKitConsole.onJobCompleteConsole = (action, vmId) => {
    origComplete?.(action, vmId);
    saveVmCachePersist();
    renderMissionProgress();
    notifyJobDone(action);
  };
  const origRenderFleet = window.renderFleet;
  if (origRenderFleet) {
    window.renderFleet = () => {
      origRenderFleet();
      renderFleetTable();
      updateFleetBatchBar();
      window.GuestKitJourney?.renderFleetSummaryBar?.(
        window.GuestKitJourney?.deriveFleetStats?.(window.state?.vms)
      );
    };
  }
}

window.GuestKitFeatures = {
  initGuestKitFeatures,
  filterFleetVms,
  renderMissionProgress,
  renderRecentDisks,
  saveVmCachePersist,
  loadJobHistory,
  quickScan,
  populateCompareSelects,
  cleanupShadowRows,
  renderFleetTable,
  exportFleetReport,
  runFleetScanModal,
};
