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
  let list = (vms || []).filter((v) => window.isFleetDisk?.(v) ?? true);
  if (q) {
    list = list.filter((v) =>
      (v.name || '').toLowerCase().includes(q)
      || (v.format || '').toLowerCase().includes(q)
      || v.id.toLowerCase().includes(q));
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
  document.getElementById('fleetAiOverviewBtn')?.addEventListener('click', () => window.GuestKitAi?.aiFleetOverview?.());
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
};
