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

// ── TUI-style intelligence report ────────────────────────────────────────────
function irEsc(s) { return window.escapeHtml ? window.escapeHtml(String(s ?? '')) : String(s ?? ''); }
function irOsEmoji(os) {
  const d = String(os?.distribution || os?.type || '').toLowerCase();
  if (d.includes('ubuntu')) return '🟠'; if (d.includes('debian')) return '🌀';
  if (d.includes('photon')) return '🪷'; if (d.includes('rhel') || d.includes('centos') || d.includes('fedora')) return '🎩';
  if (d.includes('suse')) return '🦎'; if (d.includes('arch')) return '🏔️';
  if (d.includes('windows')) return '🪟'; return '🐧';
}
function irScoreCol(n) { return n == null ? 'var(--text-muted)' : n >= 75 ? 'var(--success)' : n >= 50 ? 'var(--warn)' : 'var(--danger)'; }
function irRing(score, label) {
  if (score == null) return '';
  const pct = Math.max(0, Math.min(100, Math.round(score)));
  return `<div class="ir-ring" style="--p:${pct};--rc:${irScoreCol(pct)}"><span class="ir-ring__n">${pct}</span><span class="ir-ring__l">${irEsc(label || '')}</span></div>`;
}
function irKV(k, v) { return (v == null || v === '') ? '' : `<div><dt>${irEsc(k)}</dt><dd>${irEsc(v)}</dd></div>`; }
function irCard(title, body, badge) {
  if (!body || !body.trim()) return '';
  return `<section class="ir-sec"><header class="ir-sec__h">${badge ? `<span class="ir-sec__ic">${badge}</span>` : ''}<h4>${irEsc(title)}</h4></header>${body}</section>`;
}
function irChips(arr, cls) { return (arr || []).map((x) => `<span class="ir-chip ${cls || ''}">${irEsc(x)}</span>`).join(''); }
function irBullets(arr) { return `<ul class="ir-list">${(arr || []).slice(0, 12).map((x) => `<li>${irEsc(x)}</li>`).join('')}</ul>`; }
function irSevOf(sev) { const s = String(sev || '').toLowerCase(); if (s.startsWith('block') || s === 'critical' || s === 'high') return 'crit'; if (s.startsWith('warn') || s === 'medium') return 'warn'; return 'ok'; }
function irFinding(f, cls) {
  const t = typeof f === 'string' ? f : (f.title || f.check_id || '');
  const m = typeof f === 'string' ? '' : (f.message || '');
  const rem = typeof f === 'string' ? '' : (f.remediation || '');
  return `<div class="ir-find ${cls}"><div class="ir-find__t">${irEsc(t)}</div>${m ? `<div class="ir-find__m">${irEsc(m)}</div>` : ''}${rem ? `<div class="ir-find__r">→ ${irEsc(rem)}</div>` : ''}</div>`;
}
function irCheck(c) {
  const cls = c.passed ? 'ok' : irSevOf(c.severity);
  return `<div class="ir-check ${cls}"><span class="ir-check__s">${c.passed ? '✓' : '✗'}</span><span class="ir-check__id">${irEsc(c.id || '')}</span><span class="ir-check__n">${irEsc(c.name || c.message || '')}</span></div>`;
}
function irPriCls(p) { const s = String(p || '').toLowerCase(); if (s === 'critical') return 'crit'; if (s === 'high') return 'warn'; if (s === 'medium') return 'warn'; return 'ok'; }
function irOp(op) {
  return `<div class="ir-op"><span class="ir-chip ${irPriCls(op.priority)}">${irEsc(op.priority || '—')}</span><span class="ir-op__d">${irEsc(op.description || op.id || '')}</span>${op.risk ? `<span class="ir-op__risk">${irEsc(op.risk)}</span>` : ''}</div>`;
}
function renderIntelligenceReport(vm, cache) {
  const ins = cache.inspect || {};
  const hasInspect = !!cache.inspect;
  const os = ins.operating_system || {};
  const bs = cache.bootScore;
  const mp = cache.migrationPlan || {};
  const mscore = mp.migration_score || {};
  const fix = mp.fix_plan || {};
  const cop = cache.briefing || {};
  const checks = cache.checks || [];
  const blockers = cache.blockers || [];
  const warnings = cache.warnings || [];

  if (!hasInspect && bs == null && !cache.migrationPlan) {
    return '<p class="body-text">No intelligence yet — click <strong>Analyze</strong> to fingerprint the OS, score bootability, and plan migration.</p>';
  }

  const secs = [];

  if (hasInspect) {
    secs.push(irCard('System', `<dl class="ir-grid">
      ${irKV('OS', os.product_name || os.distribution || os.type || 'Unknown')}
      ${irKV('Version', os.version)}
      ${irKV('Arch', os.arch)}
      ${irKV('Hostname', os.hostname || ins.network?.hostname)}
      ${irKV('Packaging', os.package_format || ins.packages?.manager)}
      ${irKV('Mounts', ins.mountpoints?.count)}
    </dl>`, irOsEmoji(os)));
  }

  if (bs != null || blockers.length || checks.length) {
    const conf = cache.confidence != null ? ` · confidence ${Math.round(cache.confidence * 100)}%` : '';
    const prob = bs != null ? `${Math.round(bs)}% chance of a clean first boot${conf}` : (cache.bootSummary || '');
    const blk = blockers.length ? `<div class="ir-sub">Blockers</div>${blockers.map((b) => irFinding(b, 'crit')).join('')}` : '';
    const wrn = warnings.length ? `<div class="ir-sub">Warnings</div>${warnings.map((w) => irFinding(w, 'warn')).join('')}` : '';
    const chk = checks.length ? `<div class="ir-checks">${checks.map(irCheck).join('')}</div>` : '';
    secs.push(irCard('Boot gate', `<div class="ir-gate">${irRing(bs, 'boot')}<div class="ir-gate__b"><p class="ir-prob">${irEsc(prob)}</p>${blk}${wrn}</div></div>${chk}`, '◉'));
  }

  if (cache.migrationPlan) {
    const dt = mscore.estimated_downtime_minutes;
    secs.push(irCard('Migration readiness', `<div class="ir-gate">${irRing(mscore.score, 'migrate')}<div class="ir-gate__b">
      ${dt != null ? `<p class="ir-prob">~${dt} min estimated downtime → ${irEsc(mp.target || 'kubevirt')}</p>` : ''}
      ${mscore.driver_injections?.length ? `<div class="ir-sub">Driver injections</div><div class="ir-chiprow">${irChips(mscore.driver_injections, 'ok')}</div>` : ''}
      ${mscore.required_changes?.length ? `<div class="ir-sub">Required changes</div>${irBullets(mscore.required_changes)}` : ''}
      ${mscore.licensing_warnings?.length ? `<div class="ir-sub">Licensing</div>${irBullets(mscore.licensing_warnings)}` : ''}
    </div></div>`, '⚙'));
  }

  if (fix.operations?.length) {
    secs.push(irCard(`Fix plan · ${fix.operations.length} ops`, `<div class="ir-ops">${fix.operations.slice(0, 40).map(irOp).join('')}</div>`, '✦'));
  }

  if (hasInspect && ins.security) {
    const se = ins.security.selinux, aa = ins.security.apparmor;
    secs.push(irCard('Security', `<dl class="ir-grid">
      ${irKV('SELinux', se ? (se.status || (se.enabled ? 'enabled' : 'disabled')) : '—')}
      ${irKV('AppArmor', aa ? (aa.enabled ? 'enabled' : 'disabled') : '—')}
    </dl>`, '⛨'));
  }

  if (hasInspect && (ins.packages || ins.services || ins.network)) {
    const pk = ins.packages || {}, sv = ins.services || {}, nw = ins.network || {};
    secs.push(irCard('Inventory', `
      ${pk.count != null ? `<div class="ir-sub">Packages · ${pk.count} (${irEsc(pk.manager || '?')})</div><div class="ir-chiprow">${irChips((pk.sample || []).slice(0, 24))}</div>` : ''}
      ${sv.count != null ? `<div class="ir-sub">Enabled services · ${sv.count}</div><div class="ir-chiprow">${irChips((sv.sample || []).slice(0, 24))}</div>` : ''}
      ${nw.interfaces?.length ? `<div class="ir-sub">Network interfaces</div><div class="ir-chiprow">${irChips(nw.interfaces, 'ok')}</div>` : ''}
    `, '▤'));
  }

  if (cop.recommended_actions?.length || cop.headline) {
    const recs = (cop.recommended_actions || []).slice(0, 8).map((r) => `<div class="ir-rec"><span class="ir-chip ${r.priority <= 1 ? 'crit' : r.priority <= 2 ? 'warn' : 'ok'}">P${r.priority ?? '-'}</span><div><strong>${irEsc(r.title)}</strong>${r.detail ? `<p>${irEsc(r.detail)}</p>` : ''}</div></div>`).join('');
    secs.push(irCard('Recommendations', `${cop.headline ? `<p class="ir-prob">${irEsc(cop.headline)}</p>` : ''}${recs}`, '✦'));
  }

  return `<div class="intel-report">${secs.join('')}</div>`;
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

  const detected = renderIntelligenceReport(vm, cache);

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
      <button type="button" class="btn primary sm" id="previewAnalyzeBtn">⚡ Analyze</button>
      <button type="button" class="btn secondary sm" data-preview-action="inspect">Fingerprint</button>
      <button type="button" class="btn secondary sm" data-preview-action="doctor">Boot Doctor</button>
      <button type="button" class="btn secondary sm" data-preview-action="migration-plan">Migration Plan</button>
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
  nb$('#previewAnalyzeBtn')?.addEventListener('click', () => window.runFullAnalysis?.());
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
      <h4 class="brain-block__title">⚡ Ask Zeus</h4>
      <p class="caption">Zeus AI · GuestKit engine</p>
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
      <h4 class="brain-block__title">Ask Zeus</h4>
      <div class="brain-prompt-chips" id="brainPromptChipsMini"></div>
      <div id="brainAskChatMini" class="brain-ask-chat"></div>
      <form class="brain-ask-input" id="brainAskFormMini">
        <input type="text" id="brainAskInputMini" placeholder="Ask Zeus about boot risks…" autocomplete="off" />
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
