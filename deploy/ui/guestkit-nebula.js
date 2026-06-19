/* GuestKit Nebula — intake, pipeline, preview, brain UI */

const PIPELINE_STEPS = [
  { id: 'source', label: 'Source' },
  { id: 'fingerprint', label: 'Fingerprint' },
  { id: 'mount', label: 'Mount' },
  { id: 'inspect', label: 'Inspect' },
  { id: 'risk', label: 'Risk Score' },
  { id: 'report', label: 'Report' },
  { id: 'launch', label: 'Launch' },
];

function nb$(sel) { return document.querySelector(sel); }

function derivePipelineSteps(vm, cache) {
  cache = cache || {};
  const activeJob = window.state?.activeJob;
  const jobMap = { inspect: 'fingerprint', doctor: 'risk', 'repair-plan': 'report', 'migration-plan': 'report', provision: 'launch' };
  const running = activeJob ? jobMap[activeJob] : null;

  function status(id) {
    if (running === id) return 'running';
    if (!vm && id !== 'source') return 'locked';
    if (id === 'source') return vm ? 'complete' : 'idle';
    if (id === 'fingerprint') return cache.inspect ? 'complete' : (vm ? 'idle' : 'locked');
    if (id === 'mount') return cache.inspect ? 'complete' : 'locked';
    if (id === 'inspect') return cache.inspect ? 'complete' : (cache.lastOp === 'inspect' ? 'running' : 'locked');
    if (id === 'risk') {
      if (cache.bootScore != null) return cache.blockers?.length ? 'warning' : 'complete';
      return cache.inspect ? 'idle' : 'locked';
    }
    if (id === 'report') return cache.repairPlan || cache.migrationPlan || window.state?.lastYaml ? 'complete' : (cache.bootScore != null ? 'idle' : 'locked');
    if (id === 'launch') {
      if (cache.lastOp === 'provision') return 'complete';
      if (window.state?.lastYaml) return 'idle';
      return 'locked';
    }
    return 'locked';
  }

  const labels = {
    idle: 'Waiting', complete: 'Complete', running: 'Running', warning: 'Needs attention',
    failed: 'Failed', locked: 'Locked',
  };

  return PIPELINE_STEPS.map((s) => ({ ...s, status: status(s.id), statusLabel: labels[status(s.id)] || '—' }));
}

function renderPipeline(vm, cache) {
  const el = nb$('#pipelineRibbon');
  if (!el) return;
  const steps = derivePipelineSteps(vm, cache);
  el.innerHTML = steps.map((s) =>
    `<div class="pipeline-step ${s.status}" title="${s.label}">
      <span class="pipeline-step__label">${s.label}</span>
      <span class="pipeline-step__status">${s.statusLabel}</span>
    </div>`
  ).join('');
}

function renderDiskPreview(vm, cache) {
  const el = nb$('#diskPreview');
  if (!el) return;
  if (!vm) {
    el.innerHTML = '<p class="brain-rec">Select a disk from the vault to see intelligence preview.</p>';
    return;
  }
  cache = cache || window.getVmCache?.(vm.id) || {};
  const inspect = cache.inspect || {};
  const os = inspect.os?.distribution || inspect.os?.os_type || 'Unknown';
  const boot = inspect.boot?.mode || inspect.boot?.boot_mode || '—';
  const score = cache.bootScore;
  const readiness = score != null ? Math.round(score) : null;

  let detected = '';
  if (cache.inspect) {
    detected = `
      <dl class="disk-preview__grid">
        <div><dt>Detected OS</dt><dd>${window.escapeHtml?.(os)}</dd></div>
        <div><dt>Boot</dt><dd>${window.escapeHtml?.(String(boot))}</dd></div>
        <div><dt>Kernel</dt><dd>${window.escapeHtml?.(inspect.boot?.kernel || inspect.kernel?.version || '—')}</dd></div>
        <div><dt>Cloud-init</dt><dd>${inspect.cloud_init?.present ? 'present' : 'none'}</dd></div>
        <div><dt>Risk</dt><dd>${cache.blockers?.length ? `${cache.blockers.length} blocker(s)` : (readiness != null ? 'low' : 'unknown')}</dd></div>
        <div><dt>Migration readiness</dt><dd>${readiness != null ? `${readiness}%` : '—'}</dd></div>
      </dl>`;
  } else {
    detected = '<p class="body-text">OS not detected — run Fingerprint to analyze bootloader, drivers, and migration readiness.</p>';
  }

  el.innerHTML = `
    <div class="disk-preview__head">
      <div>
        <h3 class="section-title truncate">${window.escapeHtml?.(vm.name || 'Unnamed')}</h3>
        <p class="caption mono">${vm.format || 'disk'} · ${window.fmtBytes?.(vm.size_bytes)} · ${vm.id.slice(0, 12)}…</p>
      </div>
      ${readiness != null ? `<span class="badge ready">boot ${readiness}</span>` : '<span class="badge muted">unscanned</span>'}
    </div>
    ${detected}
    <div class="disk-preview__actions">
      <button type="button" class="btn primary sm" data-preview-action="inspect">Run Fingerprint</button>
      <button type="button" class="btn secondary sm" data-preview-action="doctor">Boot Doctor</button>
      <button type="button" class="btn secondary sm" data-preview-action="migration-plan">Generate Report</button>
      <button type="button" class="btn secondary sm" data-preview-action="provision">Create VM</button>
      <button type="button" class="btn secondary sm" id="previewConvertBtn">Convert Disk</button>
      <button type="button" class="btn danger sm" id="previewDeleteBtn">Delete</button>
    </div>
    <div class="intel-log" id="intelLog"></div>`;

  el.querySelectorAll('[data-preview-action]').forEach((btn) => {
    btn.addEventListener('click', () => {
      const a = btn.dataset.previewAction;
      if (a === 'provision') window.GuestKitConsole?.showLaunchPreview?.(() => window.runAction?.('provision'));
      else window.runAction?.(a);
    });
  });
  nb$('#previewConvertBtn')?.addEventListener('click', () => window.GuestKitConsole?.showConvertStudio?.());
  nb$('#previewDeleteBtn')?.addEventListener('click', () => document.getElementById('deleteDiskBtn')?.click());
  appendIntelLog(cache);
}

function appendIntelLog(cache) {
  const log = nb$('#intelLog');
  if (!log) return;
  const lines = [];
  if (cache?.lastOp) lines.push(`[${cache.lastOp}] ${cache.status || ''}`);
  if (cache?.bootScore != null) lines.push(`boot_score=${Math.round(cache.bootScore)}`);
  if (cache?.blockers?.length) lines.push(`blockers=${cache.blockers.length}`);
  log.textContent = lines.length ? lines.join('\n') : 'Intelligence log — run Fingerprint to populate.';
}

function renderBrainPanel(vm, cache) {
  const targets = [nb$('#brainPanelContent'), nb$('#brainDrawerContent')];
  cache = cache || (vm ? window.getVmCache?.(vm.id) : {}) || {};
  const state = window.GuestKitJourney?.deriveJourneyState?.(vm, cache, window.state?.vms, window.state?.systemStatus) || {};
  const inspect = cache.inspect || {};
  const os = inspect.os?.distribution || inspect.os?.os_type || 'Unknown';
  const score = cache.bootScore;
  const pct = score != null ? Math.round(score) : 0;

  const blockers = cache.blockers || [];
  const warnings = (cache.checks || []).filter((c) => !c.passed).map((c) => c.message || c.id);
  const warnHtml = blockers.length || warnings.length
    ? [...blockers.map((b) => typeof b === 'string' ? b : (b.message || b.id)), ...warnings].slice(0, 6)
      .map((m) => `<p class="brain-warn-item">${window.escapeHtml?.(m)}</p>`).join('')
    : '<p class="brain-rec">No warnings detected.</p>';

  const html = `
    <div class="brain-block">
      <h4 class="brain-block__title">GuestKit Brain</h4>
      <p class="caption">Offline VM Intelligence</p>
    </div>
    <div class="brain-block">
      <h4 class="brain-block__title">Readiness</h4>
      <div class="brain-readiness">
        <span class="brain-readiness__pct">${score != null ? pct : '—'}</span>
        <div class="brain-readiness__bar"><div class="brain-readiness__fill" style="width:${score != null ? pct : 0}%"></div></div>
      </div>
    </div>
    <div class="brain-block brain-detected">
      <h4 class="brain-block__title">Detected</h4>
      <dl>
        <dt>OS</dt><dd>${window.escapeHtml?.(os)}</dd>
        <dt>Boot</dt><dd>${window.escapeHtml?.(inspect.boot?.mode || inspect.boot?.boot_mode || '—')}</dd>
        <dt>Disk</dt><dd>${vm ? (vm.format || '—') : '—'}</dd>
        <dt>Network</dt><dd>${window.escapeHtml?.(inspect.network?.manager || inspect.network?.config || '—')}</dd>
        <dt>Agent</dt><dd>${inspect.agent?.present ? 'present' : 'not installed'}</dd>
      </dl>
    </div>
    <div class="brain-block">
      <h4 class="brain-block__title">Warnings</h4>
      ${warnHtml}
    </div>
    <div class="brain-block">
      <h4 class="brain-block__title">Recommended</h4>
      <button type="button" class="btn primary sm" id="brainPrimaryAction">${window.escapeHtml?.(state.next?.label || 'Select a disk')}</button>
      <p class="caption" style="margin-top:8px;">${window.escapeHtml?.(state.summary?.slice(0, 120) || '')}</p>
    </div>
    <div class="brain-block">
      <h4 class="brain-block__title">Ask GuestKit</h4>
      <div class="brain-prompt-chips" id="brainPromptChipsMini"></div>
      <div id="brainAskChatMini" class="brain-ask-chat"></div>
      <form class="brain-ask-input" id="brainAskFormMini">
        <input type="text" id="brainAskInputMini" placeholder="Ask about boot risks…" autocomplete="off" />
        <button type="submit" class="btn primary sm">Ask</button>
      </form>
    </div>
    <div class="job-tracker hidden" id="jobTrackerMini"></div>`;

  targets.forEach((t) => { if (t) t.innerHTML = html; });

  nb$('#brainPrimaryAction')?.addEventListener('click', () => {
    if (state.next?.workflow) window.runAction?.(state.next.workflow);
    else if (!vm) nb$('#fileInput')?.click();
  });
  syncBrainAskMini();
}

function syncBrainAskMini() {
  const chat = nb$('#brainAskChatMini');
  const main = nb$('#brainAskChat');
  if (chat && main) chat.innerHTML = main.innerHTML;
  const form = nb$('#brainAskFormMini');
  form?.addEventListener('submit', (e) => {
    e.preventDefault();
    const q = nb$('#brainAskInputMini')?.value?.trim();
    if (q) window.GuestKitAi?.askBrain?.(q);
    nb$('#brainAskInputMini').value = '';
  });
}

function renderAssetCard(entry, opts) {
  opts = opts || {};
  const isDir = entry.kind === 'directory';
  const size = entry.size_bytes != null ? window.fmtBytes?.(entry.size_bytes) : '';
  const registered = entry.registered;
  const cache = entry.vmId ? window.getVmCache?.(entry.vmId) : null;
  const os = cache?.inspect?.os?.distribution || '';
  const stateLabel = !isDir
    ? (cache?.inspect ? (cache.bootScore != null ? `boot ${Math.round(cache.bootScore)}` : 'fingerprinted') : (registered ? 'imported · fingerprint pending' : 'on server'))
    : `${entry.file_count != null ? entry.file_count + ' files' : 'folder'}${entry.modified ? ' · ' + entry.modified : ''}`;

  if (isDir) {
    return `<article class="asset-card asset-card--folder" data-kind="directory" data-path="${window.escapeHtml?.(entry.path)}" data-root="${opts.rootId || 0}">
      <span class="asset-card__icon">📁</span>
      <div class="asset-card__body">
        <p class="asset-card__name truncate">${window.escapeHtml?.(entry.name)}</p>
        <p class="asset-card__meta">${window.escapeHtml?.(stateLabel)}</p>
      </div>
      <button type="button" class="btn secondary sm asset-open-btn">Open</button>
    </article>`;
  }

  return `<article class="asset-card asset-card--disk" data-kind="file" data-path="${window.escapeHtml?.(entry.path)}" data-root="${opts.rootId || 0}">
    <span class="asset-card__icon">💿</span>
    <div class="asset-card__body">
      <p class="asset-card__name truncate">${window.escapeHtml?.(entry.name)}</p>
      <p class="asset-card__meta">${window.escapeHtml?.(entry.format || 'disk')}${size ? ' · ' + size : ''}${entry.modified ? ' · ' + entry.modified : ''}</p>
      <p class="asset-card__state">${window.escapeHtml?.(stateLabel)}${os ? ' · ' + os : ''}</p>
    </div>
    <div class="asset-card__actions">
      ${registered ? '<button type="button" class="btn secondary sm asset-inspect-btn">Inspect</button>' : ''}
      <button type="button" class="btn secondary sm asset-import-btn">${registered ? 'Open' : 'Import'}</button>
      ${registered ? '<button type="button" class="btn primary sm asset-launch-btn">Launch</button>' : ''}
    </div>
  </article>`;
}

function bindAssetCards(container, opts) {
  if (!container) return;
  opts = opts || {};
  container.querySelectorAll('.asset-card--folder').forEach((card) => {
    card.querySelector('.asset-open-btn')?.addEventListener('click', (e) => {
      e.stopPropagation();
      window.browseServerStorage?.(card.dataset.path, Number(card.dataset.root));
    });
    card.addEventListener('click', () => window.browseServerStorage?.(card.dataset.path, Number(card.dataset.root)));
  });
  container.querySelectorAll('.asset-card--disk').forEach((card) => {
    card.querySelector('.asset-import-btn')?.addEventListener('click', (e) => {
      e.stopPropagation();
      const path = card.dataset.path;
      const root = Number(card.dataset.root);
      const vm = window.state?.vms?.find((v) => v.path === path || v.name === path.split('/').pop());
      if (vm) window.selectVm?.(vm);
      else window.importServerDisk?.(path, root);
    });
    card.querySelector('.asset-inspect-btn')?.addEventListener('click', (e) => {
      e.stopPropagation();
      card.querySelector('.asset-import-btn')?.click();
      setTimeout(() => window.runAction?.('inspect'), 300);
    });
    card.querySelector('.asset-launch-btn')?.addEventListener('click', (e) => {
      e.stopPropagation();
      card.querySelector('.asset-import-btn')?.click();
      setTimeout(() => window.GuestKitConsole?.showLaunchPreview?.(() => window.runAction?.('provision')), 300);
    });
  });
}

function filterVaultEntries(entries) {
  const q = (nb$('#vaultSearch')?.value || '').trim().toLowerCase();
  const type = nb$('#vaultTypeFilter')?.value || '';
  const sort = nb$('#vaultSort')?.value || 'name';
  let list = [...(entries || [])];
  if (q) list = list.filter((e) => (e.name || '').toLowerCase().includes(q));
  if (type === 'folder') list = list.filter((e) => e.kind === 'directory');
  else if (type && type !== 'all') list = list.filter((e) => e.kind !== 'directory' && (e.format || '').toLowerCase() === type);
  if (sort === 'size') list.sort((a, b) => (b.size_bytes || 0) - (a.size_bytes || 0));
  else if (sort === 'date') list.sort((a, b) => String(b.modified || '').localeCompare(String(a.modified || '')));
  else list.sort((a, b) => (a.name || '').localeCompare(b.name || ''));
  return list;
}

function renderServerVaultBrowser(result) {
  const browser = nb$('#serverStorageBrowser');
  const crumb = nb$('#serverStorageBreadcrumb');
  if (!browser || !result) return;

  const parts = result.path ? result.path.split('/') : [];
  let crumbHtml = `<button type="button" class="crumb-link" data-path="" data-root="${result.root_id}">${window.escapeHtml?.(result.root_label || 'root')}</button>`;
  parts.forEach((part, i) => {
    const sub = parts.slice(0, i + 1).join('/');
    crumbHtml += ` <span class="crumb-sep">/</span> <button type="button" class="crumb-link" data-path="${window.escapeHtml?.(sub)}" data-root="${result.root_id}">${window.escapeHtml?.(part)}</button>`;
  });
  if (crumb) {
    crumb.innerHTML = crumbHtml;
    crumb.querySelectorAll('.crumb-link').forEach((btn) => {
      btn.addEventListener('click', () => window.browseServerStorage?.(btn.dataset.path || '', Number(btn.dataset.root || 0)));
    });
  }

  const entries = filterVaultEntries(result.entries || []);
  if (!entries.length) {
    browser.innerHTML = '<p class="body-text">No images in this folder — upload or pick a subdirectory.</p>';
    return;
  }

  browser.innerHTML = entries.map((e) => renderAssetCard(e, { rootId: result.root_id })).join('');
  bindAssetCards(browser, { rootId: result.root_id });
}

function setIntakeSource(source) {
  if (!window.state) return;
  window.state.intakeSource = source;
  document.querySelectorAll('.source-segment').forEach((btn) => {
    btn.classList.toggle('active', btn.dataset.source === source);
    btn.setAttribute('aria-selected', btn.dataset.source === source ? 'true' : 'false');
  });
  document.querySelectorAll('.source-panel').forEach((p) => {
    p.classList.toggle('active', p.dataset.sourcePanel === source);
  });
}

function renderAllNebula(vm, cache) {
  vm = vm ?? window.state?.selectedVm;
  cache = cache || (vm ? window.getVmCache?.(vm.id) : {}) || {};
  renderPipeline(vm, cache);
  renderDiskPreview(vm, cache);
  renderBrainPanel(vm, cache);
}

function initGuestKitNebula() {
  if (!window.state) window.state = {};
  if (!window.state.intakeSource) window.state.intakeSource = 'local';

  document.querySelectorAll('.source-segment').forEach((btn) => {
    btn.addEventListener('click', () => setIntakeSource(btn.dataset.source));
  });
  setIntakeSource(window.state.intakeSource);

  nb$('#nebulaBrowseVault')?.addEventListener('click', () => setIntakeSource('server'));
  nb$('#nebulaImportUrl')?.addEventListener('click', () => nb$('#importUrlBtn')?.click());
  nb$('#nebulaUpload')?.addEventListener('click', () => nb$('#fileInput')?.click());
  nb$('#browseBtn')?.addEventListener('click', () => nb$('#fileInput')?.click());
  nb$('#importUrlBtnAlt')?.addEventListener('click', () => nb$('#importUrlBtn')?.click());
  nb$('#importS3BtnAlt')?.addEventListener('click', () => nb$('#importS3Btn')?.click());
  nb$('#importNfsBtnAlt')?.addEventListener('click', () => nb$('#importNfsBtn')?.click());

  nb$('#brainDrawerToggle')?.addEventListener('click', () => nb$('#nebulaBrainDrawer')?.classList.toggle('open'));
  nb$('#brainDrawerClose')?.addEventListener('click', () => nb$('#nebulaBrainDrawer')?.classList.remove('open'));
  nb$('#nebulaBrainDrawer')?.querySelector('.nebula-brain-drawer__overlay')?.addEventListener('click', () => {
    nb$('#nebulaBrainDrawer')?.classList.remove('open');
  });

  nb$('#headerMoreBtn')?.addEventListener('click', () => nb$('#headerMore')?.classList.toggle('open'));
  document.addEventListener('click', (e) => {
    if (!e.target.closest('#headerMore')) nb$('#headerMore')?.classList.remove('open');
  });

  ['vaultSearch', 'vaultTypeFilter', 'vaultSort'].forEach((id) => {
    nb$(`#${id}`)?.addEventListener('input', () => {
      if (window.state?.serverStorage?.lastResult) renderServerVaultBrowser(window.state.serverStorage.lastResult);
    });
    nb$(`#${id}`)?.addEventListener('change', () => {
      if (window.state?.serverStorage?.lastResult) renderServerVaultBrowser(window.state.serverStorage.lastResult);
    });
  });

  const dropzone = nb$('#dropzone');
  if (dropzone) {
    ['dragenter', 'dragover'].forEach((ev) => {
      dropzone.addEventListener(ev, (e) => { e.preventDefault(); dropzone.classList.add('dragover'); });
    });
    ['dragleave', 'drop'].forEach((ev) => {
      dropzone.addEventListener(ev, () => dropzone.classList.remove('dragover'));
    });
  }

  renderAllNebula();
}

window.GuestKitNebula = {
  initGuestKitNebula,
  renderAllNebula,
  renderPipeline,
  renderDiskPreview,
  renderBrainPanel,
  renderServerVaultBrowser,
  renderAssetCard,
  setIntakeSource,
  derivePipelineSteps,
};
