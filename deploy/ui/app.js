const API_BASE = window.ZYVOR_API_URL || '/api/v1';

const state = {
  vms: [],
  selectedVm: null,
  activeJob: null,
  pollTimer: null,
  lastYaml: null,
};

const $ = (sel) => document.querySelector(sel);
const $$ = (sel) => document.querySelectorAll(sel);

async function api(path, options = {}) {
  const res = await fetch(`${API_BASE}${path}`, options);
  const data = await res.json().catch(() => ({}));
  if (!res.ok) throw new Error(data.message || data.error || res.statusText);
  return data;
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

function setPipelineStep(step) {
  $$('.pipe-step').forEach((btn) => {
    btn.classList.toggle('active', btn.dataset.step === step);
  });
}

function scrollToPanel(step) {
  const map = { ingest: '#panel-ingest', analyze: '#panel-fleet', plan: '#panel-actions', launch: '#panel-results' };
  const el = document.querySelector(map[step]);
  if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' });
  setPipelineStep(step);
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
    const card = document.createElement('button');
    card.type = 'button';
    card.className = 'vm-card' + (state.selectedVm?.id === vm.id ? ' selected' : '');
    card.innerHTML = `
      <span class="vm-format">${vm.format || 'disk'}</span>
      <p class="vm-name">${vm.name || 'unnamed'}</p>
      <p class="vm-meta">${fmtBytes(vm.size_bytes)} · ${vm.id.slice(0, 8)}…</p>
    `;
    card.addEventListener('click', () => selectVm(vm));
    grid.appendChild(card);
  });
}

function selectVm(vm) {
  state.selectedVm = vm;
  renderFleet();
  $('#selectedVmTitle').textContent = vm.name || 'Unnamed disk';
  $('#selectedVmMeta').textContent = `${vm.format} · ${fmtBytes(vm.size_bytes)} · ${vm.id}`;
  $$('.action-card').forEach((b) => { b.disabled = false; });
  setPipelineStep('analyze');
  feed(`Selected <strong>${vm.name}</strong>`, '');
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

async function uploadFile(file) {
  if (!file) return;
  setUploadProgress(8, `Uploading ${file.name}…`);
  feed(`Ingesting <strong>${file.name}</strong>…`);

  const form = new FormData();
  form.append('file', file);

  try {
    setUploadProgress(40, 'Transferring to object store…');
    const data = await api('/vms/import', { method: 'POST', body: form });
    setUploadProgress(100, 'Complete');
    const vm = data.data;
    toast(`Ingested ${vm.name}`, 'ok');
    feed(`Ingest complete — <strong>${vm.name}</strong>`, 'ok');
    await loadFleet();
    selectVm(vm);
    scrollToPanel('analyze');
    setTimeout(() => setUploadProgress(-1), 1200);
  } catch (e) {
    setUploadProgress(-1);
    toast(e.message, 'err');
    feed(`Ingest failed: ${e.message}`, 'err');
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
  $('#jobBar').className = 'job-bar-fill';
  $('#jobBadge').textContent = 'running';
  $('#jobBadge').className = 'badge live running';
}

function hideJobTracker(status) {
  const bar = $('#jobBar');
  bar.classList.add(status === 'failed' ? 'fail' : 'done');
  $('#jobStatus').textContent = status;
  $('#jobStatus').className = `job-status ${status}`;
  $('#jobBadge').textContent = status;
  $('#jobBadge').className = `badge live ${status === 'completed' ? 'done' : 'fail'}`;
  state.activeJob = null;
}

function setScore(score) {
  const panel = $('#scorePanel');
  const ring = $('#scoreRing');
  const val = $('#scoreValue');

  if (score == null || Number.isNaN(score)) {
    panel.classList.add('hidden');
    return;
  }

  panel.classList.remove('hidden');
  const pct = Math.max(0, Math.min(100, score));
  val.textContent = Math.round(pct);
  const offset = 327 - (327 * pct) / 100;
  ring.style.strokeDashoffset = offset;

  if (pct >= 75) ring.style.stroke = 'var(--success)';
  else if (pct >= 50) ring.style.stroke = 'var(--warn)';
  else ring.style.stroke = 'var(--danger)';
}

function renderSummary(data, action) {
  const ph = $('#summaryPlaceholder');
  const content = $('#summaryContent');
  ph.classList.add('hidden');
  content.classList.remove('hidden');
  content.innerHTML = '';

  const result = data?.data?.result || data?.data?.live_status?.result || data?.result;
  const payload = typeof result === 'string' ? tryParse(result) : result;

  if (action === 'provision' && data?.data?.yaml) {
    content.innerHTML = `<p class="finding ok">KubeVirt manifests generated — <strong>${(data.data.yaml.match(/^kind:/gm) || []).length || 2}</strong> resources ready for CDI import.</p>`;
    setScore(null);
    return;
  }

  const boot = payload?.bootability || payload?.data?.bootability;
  const migrate = payload?.migration_score;

  if (boot?.score != null) setScore(boot.score);
  else if (migrate?.score != null) setScore(migrate.score);
  else setScore(null);

  if (boot?.summary) {
    content.innerHTML += `<p class="finding ok">${escapeHtml(boot.summary)}</p>`;
  }

  (boot?.blockers || []).forEach((b) => {
    const msg = typeof b === 'string' ? b : b.message || JSON.stringify(b);
    content.innerHTML += `<p class="finding blocker">⛔ ${escapeHtml(msg)}</p>`;
  });

  (boot?.warnings || []).forEach((w) => {
    const msg = typeof w === 'string' ? w : w.message || JSON.stringify(w);
    content.innerHTML += `<p class="finding warn">⚠ ${escapeHtml(msg)}</p>`;
  });

  if (migrate?.required_changes?.length) {
    content.innerHTML += `<p><strong>Required changes</strong></p>`;
    migrate.required_changes.forEach((c) => {
      content.innerHTML += `<p class="finding warn">→ ${escapeHtml(c)}</p>`;
    });
  }

  if (migrate?.estimated_downtime_minutes != null) {
    content.innerHTML += `<p class="finding ok">Est. downtime: <strong>${migrate.estimated_downtime_minutes} min</strong></p>`;
  }

  const err = data?.data?.live_status?.error;
  if (err) {
    content.innerHTML += `<p class="finding blocker">${escapeHtml(err)}</p>`;
    setScore(null);
  }

  if (!content.innerHTML) {
    ph.classList.remove('hidden');
    content.classList.add('hidden');
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

async function pollJob(jobId, action) {
  if (state.pollTimer) clearInterval(state.pollTimer);

  const tick = async () => {
    try {
      const data = await api(`/jobs/${jobId}`);
      showRaw(data);
      renderSummary(data, action);

      const status = data?.data?.live_status?.status || data?.data?.status || 'pending';
      $('#jobStatus').textContent = status;

      if (status === 'completed') {
        clearInterval(state.pollTimer);
        hideJobTracker('completed');
        feed(`Job <span class="mono">${jobId.slice(0, 8)}…</span> completed`, 'ok');
        toast(`${action} finished`, 'ok');

        if (action === 'provision' && data?.data?.yaml) {
          showYaml(data.data.yaml);
          setActiveTab('yaml');
        }
      } else if (status === 'failed') {
        clearInterval(state.pollTimer);
        hideJobTracker('failed');
        const err = data?.data?.live_status?.error || 'Job failed';
        feed(`Job failed: ${escapeHtml(err)}`, 'err');
        toast(err, 'err');
      }
    } catch (e) {
      /* keep polling */
    }
  };

  await tick();
  state.pollTimer = setInterval(tick, 2500);
}

async function runAction(action) {
  const vm = state.selectedVm;
  if (!vm) {
    toast('Select a VM from the fleet first', 'err');
    return;
  }

  let path = `/vms/${vm.id}/${action}`;
  if (action === 'doctor' || action === 'migration-plan') {
    path += '?target=kubevirt&explain=true';
  }

  feed(`Enqueueing <strong>${action}</strong>…`);
  scrollToPanel(action === 'provision' ? 'launch' : action.includes('plan') ? 'plan' : 'analyze');

  try {
    const data = await api(path, { method: 'POST' });
    showRaw(data);
    $('#summaryPlaceholder').textContent = 'Job running — results will stream in…';
    $('#summaryPlaceholder').classList.remove('hidden');
    $('#summaryContent').classList.add('hidden');
    setScore(null);
    showYaml(null);

    if (action === 'provision' && data?.data?.yaml) {
      renderSummary({ data: data.data }, action);
      showYaml(data.data.yaml);
      setActiveTab('yaml');
      toast('KubeVirt YAML ready', 'ok');
      feed('Provision YAML generated (sync)', 'ok');
      return;
    }

    const jobId = data?.data?.job_id;
    if (jobId) {
      showJobTracker(action, jobId);
      toast(`Job queued`, 'ok');
      await pollJob(jobId, action);
    }
  } catch (e) {
    toast(e.message, 'err');
    feed(`${action} failed: ${e.message}`, 'err');
  }
}

function setupActions() {
  $$('.action-card').forEach((btn) => {
    btn.addEventListener('click', () => runAction(btn.dataset.action));
  });

  $$('[data-goto]').forEach((btn) => {
    btn.addEventListener('click', () => scrollToPanel(btn.dataset.goto));
  });

  $$('.pipe-step').forEach((btn) => {
    btn.addEventListener('click', () => scrollToPanel(btn.dataset.step));
  });

  $$('.tab').forEach((tab) => {
    tab.addEventListener('click', () => setActiveTab(tab.dataset.tab));
  });

  $('#refreshFleetBtn').addEventListener('click', () => {
    loadFleet();
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
  setupActions();
  await checkHealth();
  await loadFleet();
  setInterval(checkHealth, 30000);
  feed('Observatory online — ingest a disk to begin', 'ok');
}

init();
