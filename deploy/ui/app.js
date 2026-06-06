const API_BASE = window.ZYVOR_API_URL || '/api/v1';

const WIZARD_STEPS = [
  { id: 'ingest', label: 'Ingest', hint: 'Upload disk' },
  { id: 'assure', label: 'Assure', hint: 'Boot score' },
  { id: 'plan', label: 'Plan', hint: 'Migration path' },
  { id: 'launch', label: 'Launch', hint: 'KubeVirt YAML' },
];

const WIZARD_ACTIONS = {
  assure: { action: 'doctor', label: 'Run Doctor' },
  plan: { action: 'migration-plan', label: 'Run Migrate Plan' },
  launch: { action: 'provision', label: 'Generate YAML' },
};

const state = {
  vms: [],
  selectedVm: null,
  activeJob: null,
  pollTimer: null,
  lastYaml: null,
  lastFailedAction: null,
  wizardChain: false,
  wizard: { step: 'ingest', completed: new Set() },
  vmCache: {},
};

const $ = (sel) => document.querySelector(sel);
const $$ = (sel) => document.querySelectorAll(sel);

async function api(path, options = {}) {
  const res = await fetch(`${API_BASE}${path}`, options);
  const data = await res.json().catch(() => ({}));
  if (!res.ok) throw new Error(data.message || data.error || res.statusText);
  return data;
}

function getTarget() {
  return $('#targetSelect')?.value || 'kubevirt';
}

function fmtBytes(n) {
  if (!n) return '0 B';
  const u = ['B', 'KB', 'MB', 'GB', 'TB'];
  let i = 0;
  let v = n;
  while (v >= 1024 && i < u.length - 1) { v /= 1024; i++; }
  return `${v.toFixed(i ? 1 : 0)} ${u[i]}`;
}

function fmtTime(d = new Date()) {
  return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
}

function toast(msg, type = 'ok') {
  const stack = $('#toastStack');
  const el = document.createElement('div');
  el.className = `toast ${type}`;
  el.textContent = msg;
  stack.appendChild(el);
  setTimeout(() => el.remove(), 4200);
}

function feed(msg, type = '') {
  const ul = $('#activityFeed');
  const li = document.createElement('li');
  li.className = type;
  li.innerHTML = `${msg}<span class="time">${fmtTime()}</span>`;
  ul.prepend(li);
  while (ul.children.length > 40) ul.lastChild.remove();
}

function setHealth(ok) {
  const dot = $('#healthDot');
  const label = $('#healthLabel');
  dot.className = 'pulse-dot ' + (ok ? 'ok' : 'err');
  label.textContent = ok ? 'API online' : 'API offline';
}

async function checkHealth() {
  try {
    await api('/health');
    setHealth(true);
  } catch {
    setHealth(false);
  }
}

function getVmCache(vmId) {
  return state.vmCache[vmId] || { status: 'imported' };
}

function updateVmCache(vmId, patch) {
  state.vmCache[vmId] = { ...getVmCache(vmId), ...patch };
  renderFleet();
}

function vmStatusLabel(cache) {
  if (cache.status === 'ready') return 'ready';
  if (cache.status === 'failed') return 'failed';
  if (cache.bootScore != null) return 'analyzed';
  return cache.status || 'imported';
}

function renderWizardBar() {
  const wrap = $('#wizardSteps');
  if (!wrap) return;
  wrap.innerHTML = '';
  WIZARD_STEPS.forEach((s, i) => {
    const btn = document.createElement('button');
    btn.type = 'button';
    btn.className = 'wizard-step';
    if (state.wizard.completed.has(s.id)) btn.classList.add('done');
    if (state.wizard.step === s.id) btn.classList.add('active');
    else if (!canReachStep(s.id)) btn.classList.add('locked');
    btn.dataset.step = s.id;
    btn.innerHTML = `<span class="wizard-num">${state.wizard.completed.has(s.id) ? '✓' : i + 1}</span><span class="wizard-label">${s.label}</span>`;
    btn.addEventListener('click', () => {
      if (canReachStep(s.id)) setWizardStep(s.id);
    });
    wrap.appendChild(btn);
  });

  const idx = WIZARD_STEPS.findIndex((s) => s.id === state.wizard.step);
  const cur = WIZARD_STEPS[idx] || WIZARD_STEPS[0];
  $('#wizardSubtitle').textContent = `Step ${idx + 1} of ${WIZARD_STEPS.length} — ${cur.hint}`;
}

function canReachStep(stepId) {
  const idx = WIZARD_STEPS.findIndex((s) => s.id === stepId);
  if (idx <= 0) return true;
  for (let i = 0; i < idx; i++) {
    if (!state.wizard.completed.has(WIZARD_STEPS[i].id)) return false;
  }
  return true;
}

function markWizardComplete(stepId) {
  state.wizard.completed.add(stepId);
  renderWizardBar();
  updateWizardFooter();
}

function setPipelineStep(step) {
  const mapped = step === 'assure' ? 'assure' : step;
  $$('.pipe-step').forEach((btn) => {
    btn.classList.toggle('active', btn.dataset.step === mapped);
  });
}

function setWizardStep(step) {
  state.wizard.step = step;
  $('#panels')?.setAttribute('data-wizard-step', step);
  setPipelineStep(step);
  renderWizardBar();
  updateWizardFooter();

  const scrollMap = {
    ingest: '#panel-ingest',
    assure: '#panel-fleet',
    plan: '#panel-actions',
    launch: '#panel-results',
  };
  const el = document.querySelector(scrollMap[step]);
  if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' });
}

function updateWizardFooter() {
  const idx = WIZARD_STEPS.findIndex((s) => s.id === state.wizard.step);
  const back = $('#wizardBackBtn');
  const cont = $('#wizardContinueBtn');
  const action = $('#wizardActionBtn');
  const chain = $('#wizardChainBtn');

  back.disabled = idx <= 0;

  const stepCfg = WIZARD_ACTIONS[state.wizard.step];
  if (stepCfg && state.selectedVm) {
    action.textContent = stepCfg.label;
    action.classList.remove('hidden');
    action.dataset.action = stepCfg.action;
  } else {
    action.classList.add('hidden');
  }

  chain.classList.toggle('hidden', !(state.selectedVm && state.wizard.step === 'assure'));

  if (idx >= WIZARD_STEPS.length - 1) {
    cont.textContent = 'Done';
    cont.disabled = state.wizard.completed.has('launch');
  } else {
    cont.textContent = 'Continue';
    cont.disabled = !state.wizard.completed.has(state.wizard.step);
  }
}

function scrollToPanel(step) {
  setWizardStep(step);
}

function renderFleet() {
  const grid = $('#fleetGrid');
  const empty = $('#fleetEmpty');
  $('#fleetCount').textContent = `${state.vms.length} image${state.vms.length === 1 ? '' : 's'}`;

  grid.querySelectorAll('.vm-card').forEach((c) => c.remove());

  if (!state.vms.length) {
    empty.classList.remove('hidden');
    return;
  }
  empty.classList.add('hidden');

  state.vms.forEach((vm) => {
    const cache = getVmCache(vm.id);
    const status = vmStatusLabel(cache);
    const scoreChip = cache.bootScore != null
      ? `<span class="vm-score">${Math.round(cache.bootScore)}</span>`
      : '';

    const card = document.createElement('button');
    card.type = 'button';
    card.className = 'vm-card' + (state.selectedVm?.id === vm.id ? ' selected' : '');
    card.innerHTML = `
      <span class="vm-format">${vm.format || 'disk'}</span>
      <span class="vm-status ${status}">${status}</span>
      ${scoreChip}
      <p class="vm-name">${escapeHtml(vm.name || 'unnamed')}</p>
      <p class="vm-meta">${fmtBytes(vm.size_bytes)} · ${vm.id.slice(0, 8)}…</p>
    `;
    card.addEventListener('click', () => selectVm(vm));
    grid.appendChild(card);
  });
}

function setDockEnabled(on) {
  ['#dockInspect', '#dockDoctor', '#dockPlan', '#dockRepair', '#dockLaunch'].forEach((sel) => {
    const el = $(sel);
    if (el) el.disabled = !on;
  });
}

function selectVm(vm) {
  state.selectedVm = vm;
  renderFleet();
  $('#selectedVmTitle').textContent = vm.name || 'Unnamed disk';
  $('#selectedVmMeta').textContent = `${vm.format} · ${fmtBytes(vm.size_bytes)} · ${vm.id}`;
  $$('.action-card').forEach((b) => { b.disabled = false; });
  setDockEnabled(true);
  if (state.wizard.step === 'ingest' && state.wizard.completed.has('ingest')) {
    setWizardStep('assure');
  }
  updateWizardFooter();
  feed(`Selected <strong>${escapeHtml(vm.name)}</strong>`, '');
}

async function loadFleet() {
  try {
    const data = await api('/vms');
    state.vms = data.data || [];
    renderFleet();
    if (state.selectedVm) {
      const fresh = state.vms.find((v) => v.id === state.selectedVm.id);
      if (fresh) state.selectedVm = fresh;
      else state.selectedVm = null;
    }
  } catch (e) {
    toast(`Fleet load failed: ${e.message}`, 'err');
  }
}

function setUploadProgress(pct, label) {
  const wrap = $('#uploadProgress');
  const bar = $('#uploadBar');
  wrap.classList.toggle('hidden', pct < 0);
  if (pct >= 0) {
    bar.style.setProperty('--pct', `${pct}%`);
    $('#uploadLabel').textContent = label;
  }
}

function uploadWithProgress(file) {
  return new Promise((resolve, reject) => {
    const xhr = new XMLHttpRequest();
    const form = new FormData();
    form.append('file', file);

    xhr.upload.addEventListener('progress', (e) => {
      if (e.lengthComputable) {
        const pct = Math.round((e.loaded / e.total) * 100);
        setUploadProgress(pct, `Uploading ${file.name}… ${pct}%`);
      }
    });

    xhr.addEventListener('load', () => {
      let data = {};
      try { data = JSON.parse(xhr.responseText); } catch { /* */ }
      if (xhr.status >= 200 && xhr.status < 300) resolve(data);
      else reject(new Error(data.message || data.error || xhr.statusText));
    });

    xhr.addEventListener('error', () => reject(new Error('Upload failed')));
    xhr.open('POST', `${API_BASE}/vms/import`);
    xhr.send(form);
  });
}

async function uploadFile(file) {
  if (!file) return;
  setUploadProgress(0, `Uploading ${file.name}…`);
  feed(`Ingesting <strong>${escapeHtml(file.name)}</strong>…`);

  try {
    const data = await uploadWithProgress(file);
    setUploadProgress(100, 'Complete');
    const vm = data.data;
    toast(`Ingested ${vm.name}`, 'ok');
    feed(`Ingest complete — <strong>${escapeHtml(vm.name)}</strong>`, 'ok');
    markWizardComplete('ingest');
    await loadFleet();
    selectVm(vm);
    setWizardStep('assure');
    setTimeout(() => setUploadProgress(-1), 1200);
  } catch (e) {
    setUploadProgress(-1);
    toast(e.message, 'err');
    feed(`Ingest failed: ${escapeHtml(e.message)}`, 'err');
  }
}

function setupDropzone() {
  const zone = $('#dropzone');
  const input = $('#fileInput');

  zone.addEventListener('click', (e) => {
    if (e.target.id !== 'browseBtn' && e.target.closest('#browseBtn')) return;
    if (e.target.id === 'browseBtn' || e.target === zone || e.target.closest('.dropzone-inner')) {
      input.click();
    }
  });

  $('#browseBtn').addEventListener('click', (e) => {
    e.stopPropagation();
    input.click();
  });

  input.addEventListener('change', () => {
    if (input.files[0]) uploadFile(input.files[0]);
    input.value = '';
  });

  ['dragenter', 'dragover'].forEach((ev) => {
    zone.addEventListener(ev, (e) => {
      e.preventDefault();
      zone.classList.add('dragover');
    });
  });

  ['dragleave', 'drop'].forEach((ev) => {
    zone.addEventListener(ev, (e) => {
      e.preventDefault();
      zone.classList.remove('dragover');
    });
  });

  zone.addEventListener('drop', (e) => {
    const file = e.dataTransfer?.files?.[0];
    if (file) uploadFile(file);
  });
}

function showJobTracker(op, jobId) {
  state.activeJob = { id: jobId, op, start: Date.now() };
  $('#jobTracker').classList.remove('hidden');
  $('#jobOp').textContent = op.replace(/-/g, ' ');
  $('#jobIdDisplay').textContent = jobId;
  $('#jobStatus').textContent = 'pending';
  $('#jobStatus').className = 'job-status';
  const bar = $('#jobBar');
  bar.className = 'job-bar-fill';
  bar.style.width = '8%';
  $('#jobBadge').textContent = 'running';
  $('#jobBadge').className = 'badge live running';
  $('#jobRetryBtn').classList.add('hidden');
}

function hideJobTracker(status) {
  const bar = $('#jobBar');
  bar.classList.add(status === 'failed' ? 'fail' : 'done');
  if (status === 'completed') bar.style.width = '100%';
  $('#jobStatus').textContent = status;
  $('#jobStatus').className = `job-status ${status}`;
  $('#jobBadge').textContent = status;
  $('#jobBadge').className = `badge live ${status === 'completed' ? 'done' : 'fail'}`;
  state.activeJob = null;
}

function setJobProgress(pct, message) {
  const bar = $('#jobBar');
  if (pct != null) {
    bar.classList.add('progress');
    bar.style.width = `${Math.max(8, Math.min(100, pct))}%`;
  }
  if (message) $('#jobStatus').textContent = message;
}

function setScore(score, reason) {
  const panel = $('#scorePanel');
  const ring = $('#scoreRing');
  const val = $('#scoreValue');
  const label = panel.querySelector('.score-label');

  if (score == null || Number.isNaN(score)) {
    if (reason) {
      panel.classList.remove('hidden');
      val.textContent = '—';
      ring.style.strokeDashoffset = 327;
      ring.style.stroke = 'var(--text-tertiary)';
      if (label) label.textContent = reason;
    } else {
      panel.classList.add('hidden');
    }
    return;
  }

  panel.classList.remove('hidden');
  if (label) label.textContent = 'boot';
  const pct = Math.max(0, Math.min(100, score));
  val.textContent = Math.round(pct);
  const offset = 327 - (327 * pct) / 100;
  ring.style.strokeDashoffset = offset;

  if (pct >= 75) ring.style.stroke = 'var(--success)';
  else if (pct >= 50) ring.style.stroke = 'var(--warn)';
  else ring.style.stroke = 'var(--danger)';
}

function extractPayload(data) {
  const jobResult = data?.data?.result;
  if (jobResult) {
    if (jobResult.data) return jobResult.data;
    return typeof jobResult === 'string' ? tryParse(jobResult) : jobResult;
  }
  const alt = data?.data?.live_status?.result || data?.result;
  return typeof alt === 'string' ? tryParse(alt) : alt;
}

function renderFinding(item, kind) {
  if (typeof item === 'string') {
    const icon = kind === 'blocker' ? '⛔' : '⚠';
    return `<p class="finding ${kind}">${icon} ${escapeHtml(item)}</p>`;
  }
  const title = item.title ? `<strong>${escapeHtml(item.title)}</strong>: ` : '';
  const msg = escapeHtml(item.message || JSON.stringify(item));
  const remed = item.remediation
    ? `<br><span class="remediation">→ ${escapeHtml(item.remediation)}</span>`
    : '';
  const icon = kind === 'blocker' ? '⛔' : '⚠';
  return `<p class="finding ${kind}">${icon} ${title}${msg}${remed}</p>`;
}

function renderSummary(data, action) {
  const ph = $('#summaryPlaceholder');
  const content = $('#summaryContent');
  ph.classList.add('hidden');
  content.classList.remove('hidden');
  content.innerHTML = '';

  if (action === 'provision' && data?.data?.yaml) {
    content.innerHTML = `<p class="finding ok">KubeVirt manifests generated — <strong>${(data.data.yaml.match(/^kind:/gm) || []).length || 2}</strong> resources ready for CDI import.</p>`;
    setScore(null);
    return;
  }

  const payload = extractPayload(data);
  const boot = payload?.bootability;
  const migrate = payload?.migration_score;
  const repair = payload?.fix_plan || (payload?.message && payload?.before_score != null ? payload : null);

  if (boot?.score != null) setScore(boot.score);
  else if (migrate?.score != null) setScore(migrate.score);
  else if (payload?.before_score != null) setScore(payload.before_score);
  else setScore(null);

  if (boot?.summary) {
    content.innerHTML += `<p class="finding ok">${escapeHtml(boot.summary)}</p>`;
  }

  (boot?.blockers || []).forEach((b) => {
    content.innerHTML += renderFinding(b, 'blocker');
  });

  (boot?.warnings || []).forEach((w) => {
    content.innerHTML += renderFinding(w, 'warn');
  });

  if (migrate) {
    if (migrate.driver_injections?.length) {
      content.innerHTML += `<p><strong>Driver injections</strong></p>`;
      migrate.driver_injections.forEach((d) => {
        content.innerHTML += `<p class="finding ok">→ ${escapeHtml(d)}</p>`;
      });
    }
    if (migrate.required_changes?.length) {
      content.innerHTML += `<p><strong>Required changes</strong></p>`;
      migrate.required_changes.forEach((c) => {
        content.innerHTML += `<p class="finding warn">→ ${escapeHtml(c)}</p>`;
      });
    }
    if (migrate.licensing_warnings?.length) {
      migrate.licensing_warnings.forEach((w) => {
        content.innerHTML += `<p class="finding warn">⚠ ${escapeHtml(w)}</p>`;
      });
    }
    if (migrate.estimated_downtime_minutes != null) {
      content.innerHTML += `<p class="finding ok">Est. downtime: <strong>${migrate.estimated_downtime_minutes} min</strong></p>`;
    }
  }

  if (payload?.root_cause) {
    const rc = payload.root_cause;
    content.innerHTML += `<p><strong>Root cause</strong> (${Math.round((rc.confidence || 0) * 100)}% confidence)</p>`;
    if (rc.summary) content.innerHTML += `<p class="finding ok">${escapeHtml(rc.summary)}</p>`;
    if (rc.primary_cause) content.innerHTML += `<p class="finding warn">${escapeHtml(rc.primary_cause)}</p>`;
    (rc.chain || []).forEach((step) => {
      content.innerHTML += `<p class="finding ok">${step.step}. ${escapeHtml(step.description)}</p>`;
    });
  }

  if (repair?.fix_plan?.operations?.length) {
    const fp = repair.fix_plan;
    content.innerHTML += `<p><strong>Fix plan</strong> — ${fp.operations.length} operation(s), risk: ${escapeHtml(fp.overall_risk || 'unknown')}</p>`;
    fp.operations.slice(0, 5).forEach((op) => {
      const name = op.name || op.kind || op.id || 'operation';
      content.innerHTML += `<p class="finding ok">→ ${escapeHtml(name)}</p>`;
    });
    if (fp.operations.length > 5) {
      content.innerHTML += `<p class="finding ok">…and ${fp.operations.length - 5} more</p>`;
    }
  } else if (payload?.message) {
    content.innerHTML += `<p class="finding ok">${escapeHtml(payload.message)}</p>`;
    if (payload.after_score != null) {
      content.innerHTML += `<p class="finding ok">Projected score: <strong>${Math.round(payload.after_score)}</strong></p>`;
    }
  }

  const err = data?.data?.live_status?.error || data?.data?.result?.error?.message;
  if (err) {
    content.innerHTML += `<p class="finding blocker">${escapeHtml(err)}</p>`;
    setScore(null, 'failed');
  }

  if (!content.innerHTML) {
    ph.classList.remove('hidden');
    content.classList.add('hidden');
    ph.textContent = 'Run an action to see results.';
  }
}

function tryParse(s) {
  try { return JSON.parse(s); } catch { return null; }
}

function escapeHtml(s) {
  const d = document.createElement('div');
  d.textContent = s;
  return d.innerHTML;
}

function showRaw(data) {
  $('#rawJson').textContent = JSON.stringify(data, null, 2);
}

function showYaml(yaml) {
  state.lastYaml = yaml;
  $('#yamlContent').textContent = yaml || '';
  $('#yamlTab').classList.toggle('hidden', !yaml);
  $('#copyYamlBtn').classList.toggle('hidden', !yaml);
}

function setActiveTab(name) {
  $$('.tab').forEach((t) => t.classList.toggle('active', t.dataset.tab === name));
  $$('.tab-pane').forEach((p) => p.classList.remove('active'));
  const pane = name === 'summary' ? '#pane-summary' : name === 'yaml' ? '#pane-yaml' : '#pane-raw';
  document.querySelector(pane)?.classList.add('active');
}

function onJobComplete(action, data) {
  const vm = state.selectedVm;
  if (!vm) return;

  const payload = extractPayload(data);
  const boot = payload?.bootability;
  const migrate = payload?.migration_score;
  const blockers = boot?.blockers || [];

  const patch = { lastOp: action, status: 'imported' };

  if (action === 'doctor') {
    patch.bootScore = boot?.score;
    patch.blockers = blockers;
    patch.status = blockers.length ? 'failed' : 'analyzed';
    markWizardComplete('assure');
    if (!state.wizardChain && !blockers.length) setWizardStep('plan');
  } else if (action === 'migration-plan') {
    patch.migrateScore = migrate?.score ?? boot?.score;
    patch.status = 'ready';
    markWizardComplete('plan');
    if (!state.wizardChain) setWizardStep('launch');
  } else if (action === 'repair-plan') {
    patch.status = payload?.before_score != null ? 'analyzed' : patch.status;
  } else if (action === 'provision') {
    markWizardComplete('launch');
    setWizardStep('launch');
  }

  updateVmCache(vm.id, patch);
  updateWizardFooter();
}

function pollJob(jobId, action) {
  if (state.pollTimer) clearInterval(state.pollTimer);

  return new Promise((resolve) => {
    const tick = async () => {
      try {
        const data = await api(`/jobs/${jobId}`);
        showRaw(data);
        renderSummary(data, action);

        const live = data?.data?.live_status || {};
        const status = live.status || data?.data?.status || 'pending';
        $('#jobStatus').textContent = status;

        const progress = live.progress ?? live.progress_percent;
        if (progress != null) setJobProgress(progress, live.message || status);
        else if (live.message) setJobProgress(null, live.message);

        if (status === 'completed') {
          clearInterval(state.pollTimer);
          hideJobTracker('completed');
          feed(`Job <span class="mono">${jobId.slice(0, 8)}…</span> completed`, 'ok');
          toast(`${action} finished`, 'ok');
          onJobComplete(action, data);

          if (action === 'provision' && data?.data?.yaml) {
            showYaml(data.data.yaml);
            setActiveTab('yaml');
          }
          resolve({ ok: true, data });
        } else if (status === 'failed') {
          clearInterval(state.pollTimer);
          hideJobTracker('failed');
          const err = live.error || data?.data?.result?.error?.message || 'Job failed';
          feed(`Job failed: ${escapeHtml(err)}`, 'err');
          toast(err, 'err');
          state.lastFailedAction = action;
          $('#jobRetryBtn').classList.remove('hidden');
          if (state.selectedVm) {
            updateVmCache(state.selectedVm.id, { status: 'failed', lastOp: action });
          }
          setScore(null, 'failed');
          resolve({ ok: false, error: err });
        }
      } catch {
        /* keep polling */
      }
    };

    tick();
    state.pollTimer = setInterval(tick, 2500);
  });
}

async function runAction(action) {
  const vm = state.selectedVm;
  if (!vm) {
    toast('Select a VM from the fleet first', 'err');
    return { ok: false };
  }

  let path = `/vms/${vm.id}/${action}`;
  const target = getTarget();
  if (action === 'doctor' || action === 'migration-plan') {
    path += `?target=${encodeURIComponent(target)}&explain=true`;
  }

  feed(`Enqueueing <strong>${escapeHtml(action)}</strong>…`);

  const stepMap = { provision: 'launch', 'migration-plan': 'plan', 'repair-plan': 'plan', doctor: 'assure', inspect: 'assure' };
  setWizardStep(stepMap[action] || 'assure');

  try {
    const data = await api(path, { method: 'POST' });
    showRaw(data);
    $('#summaryPlaceholder').textContent = 'Job running — results will stream in…';
    $('#summaryPlaceholder').classList.remove('hidden');
    $('#summaryContent').classList.add('hidden');
    setScore(null);
    state.lastFailedAction = null;
    $('#jobRetryBtn').classList.add('hidden');

    if (action !== 'provision') showYaml(null);

    if (action === 'provision' && data?.data?.yaml) {
      renderSummary({ data: data.data }, action);
      showYaml(data.data.yaml);
      setActiveTab('yaml');
      toast('KubeVirt YAML ready', 'ok');
      feed('Provision YAML generated (sync)', 'ok');
      onJobComplete(action, { data: data.data });
      return { ok: true, data };
    }

    const jobId = data?.data?.job_id;
    if (jobId) {
      showJobTracker(action, jobId);
      toast('Job queued', 'ok');
      return await pollJob(jobId, action);
    }
    return { ok: true, data };
  } catch (e) {
    toast(e.message, 'err');
    feed(`${escapeHtml(action)} failed: ${escapeHtml(e.message)}`, 'err');
    return { ok: false, error: e.message };
  }
}

async function runWizardChain() {
  if (!state.selectedVm) {
    toast('Select a VM from the fleet first', 'err');
    return;
  }
  state.wizardChain = true;
  feed('Starting <strong>full migration</strong> chain…', '');

  try {
    const doctor = await runAction('doctor');
    if (!doctor.ok) return;

    const cache = getVmCache(state.selectedVm.id);
    if (cache.blockers?.length) {
      toast('Migration blocked — resolve blockers first', 'err');
      return;
    }

    const plan = await runAction('migration-plan');
    if (!plan.ok) return;

    setWizardStep('launch');
    toast('Plan complete — ready to provision', 'ok');
    feed('Migration chain complete — generate YAML when ready', 'ok');
  } finally {
    state.wizardChain = false;
  }
}

function setupWizard() {
  renderWizardBar();
  updateWizardFooter();

  $('#wizardBackBtn').addEventListener('click', () => {
    const idx = WIZARD_STEPS.findIndex((s) => s.id === state.wizard.step);
    if (idx > 0) setWizardStep(WIZARD_STEPS[idx - 1].id);
  });

  $('#wizardContinueBtn').addEventListener('click', () => {
    const idx = WIZARD_STEPS.findIndex((s) => s.id === state.wizard.step);
    if (idx < WIZARD_STEPS.length - 1 && state.wizard.completed.has(state.wizard.step)) {
      setWizardStep(WIZARD_STEPS[idx + 1].id);
    }
  });

  $('#wizardActionBtn').addEventListener('click', () => {
    const action = $('#wizardActionBtn').dataset.action;
    if (action) runAction(action);
  });

  $('#wizardChainBtn').addEventListener('click', () => runWizardChain());

  $('#jobRetryBtn').addEventListener('click', () => {
    if (state.lastFailedAction) runAction(state.lastFailedAction);
  });
}

function setupGlassToggle() {
  $$('[data-glass-mode]').forEach((btn) => {
    btn.addEventListener('click', () => {
      const mode = btn.dataset.glassMode;
      document.documentElement.dataset.glass = mode;
      $$('[data-glass-mode]').forEach((b) => b.classList.toggle('active', b.dataset.glassMode === mode));
      toast(mode === 'clear' ? 'Clear glass' : 'Tinted glass', 'ok');
    });
  });
}

function setupActions() {
  const bindAction = (el) => {
    if (!el.dataset.action) return;
    el.addEventListener('click', () => {
      if (el.disabled) return;
      runAction(el.dataset.action);
    });
  };

  $$('.action-card').forEach(bindAction);
  $$('.dock-item[data-action]').forEach(bindAction);

  $$('[data-goto]').forEach((btn) => {
    btn.addEventListener('click', () => scrollToPanel(btn.dataset.goto));
  });

  $$('.pipe-step').forEach((btn) => {
    btn.addEventListener('click', () => {
      if (canReachStep(btn.dataset.step)) scrollToPanel(btn.dataset.step);
    });
  });

  $$('.tab').forEach((tab) => {
    tab.addEventListener('click', () => setActiveTab(tab.dataset.tab));
  });

  $('#refreshFleetBtn').addEventListener('click', () => {
    loadFleet();
    checkHealth();
    toast('Fleet refreshed', 'ok');
  });

  $('#copyYamlBtn').addEventListener('click', async () => {
    if (!state.lastYaml) return;
    try {
      await navigator.clipboard.writeText(state.lastYaml);
      toast('YAML copied', 'ok');
    } catch {
      toast('Copy failed', 'err');
    }
  });
}

async function init() {
  setupDropzone();
  setupGlassToggle();
  setupWizard();
  setupActions();
  setDockEnabled(false);
  setWizardStep('ingest');
  await checkHealth();
  await loadFleet();
  setInterval(checkHealth, 30000);
  feed('Ready — ingest a disk to begin', 'ok');
}

init();
