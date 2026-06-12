/* GuestKit — AI feature pack */

const AI_CHAT_KEY = 'zyvor.aiChat.v1';
const AI_PREFS_KEY = 'zyvor.aiPrefs.v1';

const AI_PROMPT_GROUPS = [
  {
    label: 'Boot & risk',
    prompts: [
      'Why will this VM fail to boot?',
      'Explain the top boot risks for this VM',
      'What blockers stop migration?',
      'What should I fix first?',
      'Am I ready to launch on KubeVirt?',
    ],
  },
  {
    label: 'Migration',
    prompts: [
      'Safest migration path?',
      'What driver changes are required?',
      'Do I need virtio-scsi or virtio-blk?',
      'Will cloud-init work on first boot?',
      'Compare repair vs migrate approach',
    ],
  },
  {
    label: 'Evidence',
    prompts: [
      'What evidence supports the diagnosis?',
      'Explain fstab and root mount risks',
      'Is initramfs virtio-ready?',
      'Summarize Disk DNA for an SRE',
    ],
  },
  {
    label: 'Launch',
    prompts: [
      'Generate VM YAML for 4 CPU / 8 GB',
      'What namespace should I use?',
      'Pre-launch checklist for this disk',
      'Will this boot without guest agent?',
    ],
  },
];

const FOLLOW_UPS = [
  'What should I fix first?',
  'Am I ready to launch on KubeVirt?',
  'What evidence supports the diagnosis?',
];

function aiPrefs() {
  try { return JSON.parse(localStorage.getItem(AI_PREFS_KEY) || '{}'); }
  catch { return {}; }
}

function saveAiPrefs(p) {
  localStorage.setItem(AI_PREFS_KEY, JSON.stringify({ ...aiPrefs(), ...p }));
}

function chatKey() {
  const vm = window.state?.selectedVm;
  const cv = window.state?.selectedClusterVm;
  if (vm) return `disk:${vm.id}`;
  if (cv) return `cluster:${cv.namespace}/${cv.name}`;
  return 'global';
}

function loadChat() {
  try {
    const all = JSON.parse(localStorage.getItem(AI_CHAT_KEY) || '{}');
    return all[chatKey()] || [];
  } catch { return []; }
}

function saveChat(messages) {
  try {
    const all = JSON.parse(localStorage.getItem(AI_CHAT_KEY) || '{}');
    all[chatKey()] = messages.slice(-40);
    localStorage.setItem(AI_CHAT_KEY, JSON.stringify(all));
  } catch { /* */ }
}

function readinessClass(r) {
  const x = (r || '').toLowerCase();
  if (x === 'ready') return 'ready';
  if (x === 'blocked') return 'blocked';
  if (x === 'caution' || x === 'high_risk') return 'caution';
  return '';
}

function renderPromptChips() {
  const wrap = document.getElementById('brainPromptChips');
  if (!wrap) return;
  wrap.innerHTML = AI_PROMPT_GROUPS.map((g) => `
    <div class="ai-prompt-group">
      <span class="ai-prompt-group__label">${window.escapeHtml?.(g.label)}</span>
      ${g.prompts.map((p) =>
        `<button type="button" class="brain-prompt-chip" data-prompt="${window.escapeHtml?.(p)}">${window.escapeHtml?.(p)}</button>`
      ).join('')}
    </div>`).join('');
  wrap.querySelectorAll('.brain-prompt-chip').forEach((btn) => {
    btn.addEventListener('click', () => askBrain(btn.dataset.prompt));
  });
}

function renderFollowUpChips() {
  const el = document.getElementById('brainFollowUpChips');
  if (!el) return;
  el.innerHTML = FOLLOW_UPS.map((p) =>
    `<button type="button" class="brain-prompt-chip sm" data-prompt="${window.escapeHtml?.(p)}">${window.escapeHtml?.(p)}</button>`
  ).join('');
  el.querySelectorAll('.brain-prompt-chip').forEach((btn) => {
    btn.addEventListener('click', () => askBrain(btn.dataset.prompt));
  });
}

function appendChatMessage(role, text, meta) {
  const chat = document.getElementById('brainAskChat');
  if (!chat) return;
  const cls = role === 'user' ? '' : role === 'thinking' ? 'thinking' : 'cyan';
  const label = role === 'user' ? 'You' : role === 'thinking' ? 'GuestKit' : 'GuestKit';
  const div = document.createElement('div');
  div.className = `brain-chat-msg ${cls}`;
  div.innerHTML = `<strong>${label}:</strong> ${window.escapeHtml?.(text)}${meta ? `<span class="brain-chat-meta">${window.escapeHtml?.(meta)}</span>` : ''}`;
  if (role === 'thinking') div.dataset.thinking = '1';
  chat.appendChild(div);
  chat.scrollTop = chat.scrollHeight;
  return div;
}

function renderChatHistory() {
  const chat = document.getElementById('brainAskChat');
  if (!chat) return;
  const msgs = loadChat();
  chat.innerHTML = msgs.length
    ? msgs.map((m) => `<div class="brain-chat-msg ${m.role === 'assistant' ? 'cyan' : ''}"><strong>${m.role === 'user' ? 'You' : 'GuestKit'}:</strong> ${window.escapeHtml?.(m.text)}</div>`).join('')
    : '<p class="brain-rec">Ask about boot risks, migration blockers, or launch readiness.</p>';
}

function pushChat(role, text) {
  const msgs = loadChat();
  msgs.push({ role, text, at: Date.now() });
  saveChat(msgs);
}

function renderAiDeck() {
  const el = document.getElementById('brainAiDeck');
  if (!el) return;
  const vm = window.state?.selectedVm;
  const cache = vm ? window.getVmCache?.(vm.id) || {} : {};
  const b = window.state?.lastBriefing || window.state?.lastClusterBriefing || cache.briefing;
  if (!b) {
    el.innerHTML = '<p class="brain-rec">Run <strong>Doctor</strong> with explain to unlock the AI briefing deck.</p>';
    el.classList.remove('hidden');
    return;
  }
  el.classList.remove('hidden');
  const digest = b.evidence_digest;
  el.innerHTML = `
    <div class="ai-deck-head">
      <span class="readiness-pill ${readinessClass(b.readiness)}">${window.escapeHtml?.(b.readiness)}</span>
      <span class="ai-deck-scores">${Math.round(b.boot_score)} boot${b.migration_score != null ? ` · ${Math.round(b.migration_score)} migrate` : ''}</span>
    </div>
    <h4 class="ai-deck-headline">${window.escapeHtml?.(b.headline)}</h4>
    <p class="brain-rec">${window.escapeHtml?.(b.summary)}</p>
    ${digest ? `<p class="mono ai-deck-digest">${window.escapeHtml?.(digest.os)} · ${window.escapeHtml?.(digest.architecture)} · ${window.escapeHtml?.(digest.bootloader)}</p>` : ''}
    <div class="ai-evidence-grid">
      ${(b.evidence_highlights || []).slice(0, 4).map((h) => `
        <button type="button" class="ai-evidence-card" data-ask="Explain evidence: ${window.escapeHtml?.(h.label)} — ${window.escapeHtml?.(h.detail)}">
          <span class="mono">${window.escapeHtml?.(h.ref)}</span>
          <strong>${window.escapeHtml?.(h.label)}</strong>
          <p>${window.escapeHtml?.(h.detail)}</p>
        </button>`).join('')}
    </div>
    <div class="ai-insight-grid" id="brainInsightGrid">
      ${(b.insights || []).slice(0, 4).map((ins) => `
        <button type="button" class="ai-insight-card" data-insight-id="${window.escapeHtml?.(ins.id)}">
          <span class="ai-insight-q">${window.escapeHtml?.(ins.question)}</span>
          <span class="ai-insight-a">${window.escapeHtml?.(ins.answer.slice(0, 120))}${ins.answer.length > 120 ? '…' : ''}</span>
        </button>`).join('')}
    </div>`;

  el.querySelectorAll('.ai-evidence-card').forEach((btn) => {
    btn.addEventListener('click', () => askBrain(btn.dataset.ask));
  });
  el.querySelectorAll('.ai-insight-card').forEach((btn) => {
    btn.addEventListener('click', () => {
      const ins = (b.insights || []).find((i) => i.id === btn.dataset.insightId);
      if (ins) showInsightAnswer(ins);
    });
  });
}

function showInsightAnswer(ins) {
  appendChatMessage('user', ins.question);
  appendChatMessage('assistant', ins.answer);
  pushChat('user', ins.question);
  pushChat('assistant', ins.answer);
  renderFollowUpChips();
}

function renderAiNarrative() {
  const el = document.getElementById('brainAiNarrative');
  if (!el) return;
  const vm = window.state?.selectedVm;
  const cache = vm ? window.getVmCache?.(vm.id) || {} : {};
  const score = cache.bootScore;
  if (score == null) {
    el.innerHTML = '';
    el.classList.add('hidden');
    return;
  }
  el.classList.remove('hidden');
  const blockers = cache.blockers?.length || 0;
  let narrative;
  if (blockers) {
    narrative = `AI assessment: ${blockers} blocker(s) detected at boot score ${Math.round(score)}. Repair before launch.`;
  } else if (score >= 85) {
    narrative = `AI assessment: Strong boot confidence (${Math.round(score)}). This disk is a good launch candidate.`;
  } else if (score >= 70) {
    narrative = `AI assessment: Moderate confidence (${Math.round(score)}). Review warnings and consider repair-plan.`;
  } else {
    narrative = `AI assessment: Low confidence (${Math.round(score)}). Run migration-plan and address driver gaps.`;
  }
  el.innerHTML = `<p class="ai-narrative">${window.escapeHtml?.(narrative)}</p>`;
}

async function fetchBriefing(force) {
  const vm = window.state?.selectedVm;
  if (!vm) return null;
  const cache = window.getVmCache?.(vm.id) || {};
  if (!force && (window.state?.lastBriefing || cache.briefing)) {
    return window.state.lastBriefing || cache.briefing;
  }
  try {
    const data = await window.api?.(`/vms/${vm.id}/copilot/briefing`);
    const b = data?.data?.briefing;
    if (b) {
      window.state.lastBriefing = b;
      window.updateVmCache?.(vm.id, { briefing: b });
      if (data?.data?.job_id) window.state.lastJobId = data.data.job_id;
      renderAiDeck();
      window.renderCopilot?.(b);
    }
    return b;
  } catch { return null; }
}

async function askBrain(question) {
  const vm = window.state?.selectedVm;
  const clusterVm = window.state?.selectedClusterVm;
  if (!vm && !clusterVm) {
    window.toast?.('Select a disk or cluster VM first', 'err');
    return;
  }

  appendChatMessage('user', question);
  pushChat('user', question);
  const thinking = appendChatMessage('thinking', 'Analyzing disk intelligence…');

  try {
    let data;
    if (clusterVm) {
      if (!window.state?.lastClusterBriefing) {
        const info = window.state?.lastClusterGuestInfo;
        const boot = window.state?.lastClusterBootInspect;
        if (info) {
          window.state.lastClusterBriefing = await window.fetchClusterCopilotBriefing?.(info, clusterVm, boot);
        }
      }
      data = await window.api?.(
        `/kubevirt/vms/${encodeURIComponent(clusterVm.namespace)}/${encodeURIComponent(clusterVm.name)}/copilot/ask`,
        {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ question, briefing: window.state?.lastClusterBriefing }),
        },
      );
    } else {
      await fetchBriefing(false);
      const body = { question };
      if (window.state?.lastJobId) body.job_id = window.state.lastJobId;
      data = await window.api?.(`/vms/${vm.id}/copilot/ask`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      });
    }
    thinking?.remove();
    const insight = data?.data?.insight;
    const answer = insight?.answer || data?.data?.answer || data?.data?.response || JSON.stringify(data?.data);
    appendChatMessage('assistant', answer, insight?.id ? `#${insight.id}` : '');
    pushChat('assistant', answer);
    if (data?.data?.briefing) {
      window.state.lastBriefing = data.data.briefing;
      if (vm) window.updateVmCache?.(vm.id, { briefing: data.data.briefing });
      renderAiDeck();
    }
    if (data?.data?.job_id) window.state.lastJobId = data.data.job_id;
    renderFollowUpChips();
  } catch (e) {
    thinking?.remove();
    const local = window.answerCopilotLocal?.(question);
    const fallback = local?.answer || e.message;
    appendChatMessage('assistant', fallback);
    pushChat('assistant', fallback);
  }
}

async function aiLaunchAdvice() {
  const vm = window.state?.selectedVm;
  if (!vm) { window.toast?.('Select a disk first', 'err'); return null; }
  try {
    const data = await window.api?.(`/vms/${vm.id}/copilot/launch-advice`, { method: 'POST' });
    const ins = data?.data;
    if (ins?.answer) window.GuestKitConsole?.injectLaunchAdvice?.(ins.answer);
    return ins?.answer;
  } catch (e) {
    window.toast?.(e.message, 'err');
    return null;
  }
}

async function aiCompareNarrative(beforeName, afterName, diff) {
  try {
    const data = await window.api?.('/vms/compare/copilot', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ before_name: beforeName, after_name: afterName, diff }),
    });
    const r = data?.data;
    const el = document.getElementById('compareAiNarrative');
    if (el && r) {
      el.classList.remove('hidden');
      el.innerHTML = `
        <h4 class="ai-deck-headline">${window.escapeHtml?.(r.headline)}</h4>
        <p class="brain-rec">${window.escapeHtml?.(r.summary)}</p>
        <p class="ai-narrative accent">${window.escapeHtml?.(r.recommendation)}</p>
        <div class="ai-insight-grid">
          ${(r.insights || []).map((ins) => `
            <button type="button" class="ai-insight-card" data-answer="${window.escapeHtml?.(ins.answer)}">
              <span class="ai-insight-q">${window.escapeHtml?.(ins.question)}</span>
            </button>`).join('')}
        </div>`;
      el.querySelectorAll('.ai-insight-card').forEach((btn) => {
        btn.addEventListener('click', () => askBrain(btn.dataset.answer || btn.textContent));
      });
    }
    return r;
  } catch { return null; }
}

async function aiFleetOverview() {
  const disks = (window.state?.vms || [])
    .filter((v) => window.isFleetDisk?.(v))
    .map((v) => {
      const c = window.getVmCache?.(v.id) || {};
      return {
        name: v.name,
        boot_score: c.bootScore,
        blockers: c.blockers?.length,
        readiness: c.briefing?.readiness || window.state?.lastBriefing?.readiness,
      };
    });
  try {
    const data = await window.api?.('/copilot/fleet-overview', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ disks }),
    });
    const r = data?.data;
    const body = document.getElementById('fleetAiBody');
    if (body && r) {
      body.innerHTML = `
        <h4>${window.escapeHtml?.(r.headline)}</h4>
        <p>${window.escapeHtml?.(r.summary)}</p>
        <ul>${(r.recommendations || []).map((x) => `<li>${window.escapeHtml?.(x)}</li>`).join('')}</ul>
        ${r.priority_disk ? `<p class="ai-narrative">Priority: <strong>${window.escapeHtml?.(r.priority_disk)}</strong></p>` : ''}`;
    }
    document.getElementById('fleetAiModal')?.classList.remove('hidden');
  } catch (e) {
    window.toast?.(e.message, 'err');
  }
}

async function explainBootCheck(checkId, message) {
  const vm = window.state?.selectedVm;
  if (!vm) return;
  const q = `Explain boot check ${checkId}${message ? `: ${message}` : ''}`;
  try {
    const data = await window.api?.(`/vms/${vm.id}/copilot/explain-check`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ check_id: checkId, message }),
    });
    const ins = data?.data;
    if (ins?.answer) showInsightAnswer(ins);
    else askBrain(q);
  } catch {
    askBrain(q);
  }
}

function exportAiChat() {
  const msgs = loadChat();
  const blob = new Blob([JSON.stringify(msgs, null, 2)], { type: 'application/json' });
  const a = document.createElement('a');
  a.href = URL.createObjectURL(blob);
  a.download = `guestkit-ai-chat-${chatKey().replace(/[/:]/g, '-')}.json`;
  a.click();
}

function clearAiChat() {
  saveChat([]);
  renderChatHistory();
}

function showAiCommandPalette() {
  document.getElementById('aiCommandModal')?.classList.remove('hidden');
  document.getElementById('aiCommandInput')?.focus();
}

function setupAiCommandPalette() {
  const form = document.getElementById('aiCommandForm');
  const input = document.getElementById('aiCommandInput');
  form?.addEventListener('submit', (e) => {
    e.preventDefault();
    const q = input?.value?.trim();
    if (!q) return;
    document.getElementById('aiCommandModal')?.classList.add('hidden');
    input.value = '';
    if (q.startsWith('/doctor')) window.runAction?.('doctor');
    else if (q.startsWith('/repair')) window.runAction?.('repair-plan');
    else if (q.startsWith('/launch')) window.GuestKitConsole?.showLaunchPreview?.(() => window.runAction?.('provision'));
    else if (q.startsWith('/fleet')) aiFleetOverview();
    else askBrain(q);
  });
  document.getElementById('aiCommandClose')?.addEventListener('click', () => {
    document.getElementById('aiCommandModal')?.classList.add('hidden');
  });
}

function setupAiKeyboard() {
  document.addEventListener('keydown', (e) => {
    if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
      e.preventDefault();
      showAiCommandPalette();
    }
    if (e.key === 'a' && !e.metaKey && !e.ctrlKey && !e.target.matches('input, textarea, select')) {
      e.preventDefault();
      document.getElementById('brainAskInput')?.focus();
    }
  });
}

function onDoctorComplete(vmId) {
  fetchBriefing(true);
  renderAiNarrative();
  renderAiDeck();
  if (aiPrefs().autoExplain) {
    askBrain('Summarize the top migration risks in plain language');
  }
}

function initGuestKitAi() {
  renderPromptChips();
  renderFollowUpChips();
  renderChatHistory();
  renderAiDeck();
  renderAiNarrative();
  setupAiCommandPalette();
  setupAiKeyboard();

  document.getElementById('brainAiRefresh')?.addEventListener('click', () => fetchBriefing(true));
  document.getElementById('brainAiLaunchAdvice')?.addEventListener('click', aiLaunchAdvice);
  document.getElementById('brainAiFleetBtn')?.addEventListener('click', aiFleetOverview);
  document.getElementById('brainAiExportChat')?.addEventListener('click', exportAiChat);
  document.getElementById('brainAiClearChat')?.addEventListener('click', clearAiChat);
  document.getElementById('brainAiCommandBtn')?.addEventListener('click', showAiCommandPalette);
  document.getElementById('fleetAiClose')?.addEventListener('click', () => {
    document.getElementById('fleetAiModal')?.classList.add('hidden');
  });
  document.getElementById('prefsAutoExplain')?.addEventListener('change', (e) => {
    saveAiPrefs({ autoExplain: e.target.checked });
  });
  const autoEl = document.getElementById('prefsAutoExplain');
  if (autoEl) autoEl.checked = Boolean(aiPrefs().autoExplain);
}

function onSelectVmAi() {
  renderChatHistory();
  renderAiDeck();
  renderAiNarrative();
  fetchBriefing(false);
}

window.GuestKitAi = {
  initGuestKitAi,
  askBrain,
  renderAiDeck,
  renderAiNarrative,
  aiCompareNarrative,
  aiFleetOverview,
  aiLaunchAdvice,
  explainBootCheck,
  onDoctorComplete,
  onSelectVmAi,
  fetchBriefing,
};
