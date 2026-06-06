const API_BASE = window.ZYVOR_API_URL || '/api/v1';

async function api(path, options = {}) {
  const res = await fetch(`${API_BASE}${path}`, options);
  const data = await res.json().catch(() => ({}));
  if (!res.ok) throw new Error(data.message || res.statusText);
  return data;
}

document.getElementById('uploadBtn').addEventListener('click', async () => {
  const file = document.getElementById('file').files[0];
  const out = document.getElementById('importResult');
  if (!file) {
    out.textContent = 'Select a disk image first.';
    return;
  }
  const form = new FormData();
  form.append('file', file);
  try {
    const data = await api('/vms/import', { method: 'POST', body: form });
    out.textContent = JSON.stringify(data, null, 2);
    document.getElementById('vmId').value = data.data.id;
  } catch (e) {
    out.textContent = e.message;
  }
});

document.querySelectorAll('[data-action]').forEach((btn) => {
  btn.addEventListener('click', async () => {
    const vmId = document.getElementById('vmId').value.trim();
    const out = document.getElementById('workflowResult');
    if (!vmId) {
      out.textContent = 'Enter VM ID from import step.';
      return;
    }
    const action = btn.dataset.action;
    let path = `/vms/${vmId}/${action}`;
    if (action === 'doctor' || action === 'migration-plan') {
      path += '?target=kubevirt&explain=true';
    }
    try {
      const data = await api(path, { method: 'POST' });
      out.textContent = JSON.stringify(data, null, 2);
      if (data.data && data.data.job_id) {
        document.getElementById('jobId').value = data.data.job_id;
      }
    } catch (e) {
      out.textContent = e.message;
    }
  });
});

document.getElementById('pollBtn').addEventListener('click', async () => {
  const jobId = document.getElementById('jobId').value.trim();
  const out = document.getElementById('jobResult');
  if (!jobId) {
    out.textContent = 'Enter job ID.';
    return;
  }
  try {
    const data = await api(`/jobs/${jobId}`);
    out.textContent = JSON.stringify(data, null, 2);
  } catch (e) {
    out.textContent = e.message;
  }
});
