// SPDX-License-Identifier: Apache-2.0
/** GuestKit auth — login, token storage, SSO, settings */

const AUTH_TOKEN_KEY = 'guestkit.authToken';
const AUTH_USER_KEY = 'guestkit.authUser';

function authApiBase() {
  return window.ZYVOR_API_URL || '/api/v1';
}

function defaultAuthConfig() {
  return {
    auth_enabled: false,
    oidc_enabled: false,
    saml_enabled: false,
    allow_local_bypass: true,
    oidc_button_label: 'Sign in with SSO',
    saml_button_label: 'Sign in with SAML',
  };
}

let cachedAuthConfig = null;

async function parseApiJson(res) {
  const text = await res.text();
  if (!text) return {};
  try {
    return JSON.parse(text);
  } catch {
    return null;
  }
}

function getAuthToken() {
  return localStorage.getItem(AUTH_TOKEN_KEY) || '';
}

function setAuthToken(token) {
  if (token) localStorage.setItem(AUTH_TOKEN_KEY, token);
  else localStorage.removeItem(AUTH_TOKEN_KEY);
}

function clearAuth() {
  localStorage.removeItem(AUTH_TOKEN_KEY);
  localStorage.removeItem(AUTH_USER_KEY);
}

function getAuthUser() {
  try {
    const raw = localStorage.getItem(AUTH_USER_KEY);
    return raw ? JSON.parse(raw) : null;
  } catch {
    return null;
  }
}

function isAdmin(user) {
  return (user?.role || '').toLowerCase() === 'admin';
}

function authHeaders(extra = {}) {
  const token = getAuthToken();
  const headers = { ...extra };
  if (token) headers.Authorization = `Bearer ${token}`;
  return headers;
}

async function fetchAuthConfig() {
  try {
    const res = await fetch(`${authApiBase()}/auth/config`);
    if (res.status === 404) {
      cachedAuthConfig = defaultAuthConfig();
      return cachedAuthConfig;
    }
    const data = await parseApiJson(res);
    if (!data) {
      cachedAuthConfig = defaultAuthConfig();
      return cachedAuthConfig;
    }
    if (!res.ok) throw new Error(data.message || 'Failed to load auth config');
    cachedAuthConfig = data.data || defaultAuthConfig();
    return cachedAuthConfig;
  } catch (e) {
    console.warn('auth config unavailable; continuing without auth', e);
    cachedAuthConfig = defaultAuthConfig();
    return cachedAuthConfig;
  }
}

async function fetchAuthMe() {
  const cfg = await fetchAuthConfig();
  if (!cfg.auth_enabled) {
    return { authenticated: true, user: null };
  }
  try {
    const res = await fetch(`${authApiBase()}/auth/me`, { headers: authHeaders() });
    if (res.status === 404) {
      return { authenticated: true, user: null };
    }
    const data = await parseApiJson(res);
    if (!data) throw new Error('Auth check returned invalid JSON');
    if (!res.ok) throw new Error(data.message || 'Auth check failed');
    if (data.data?.user) localStorage.setItem(AUTH_USER_KEY, JSON.stringify(data.data.user));
    return data.data;
  } catch (e) {
    console.warn('auth me check failed', e);
    return { authenticated: false, user: null };
  }
}

async function localBypassLogin() {
  const res = await fetch(`${authApiBase()}/auth/local`, {
    method: 'POST',
    headers: authHeaders({ 'Content-Type': 'application/json' }),
  });
  const data = await res.json();
  if (!res.ok) throw new Error(data.message || 'Local login failed');
  if (data.data?.token) setAuthToken(data.data.token);
  if (data.data?.user) localStorage.setItem(AUTH_USER_KEY, JSON.stringify(data.data.user));
  return data.data;
}

async function logout() {
  try {
    await fetch(`${authApiBase()}/auth/logout`, {
      method: 'POST',
      headers: authHeaders(),
    });
  } catch (e) {
    console.warn('logout request failed', e);
  }
  clearAuth();
}

function captureTokenFromUrl() {
  const params = new URLSearchParams(window.location.search);
  const token = params.get('token');
  if (!token) return false;
  setAuthToken(token);
  params.delete('token');
  const qs = params.toString();
  const next = `${window.location.pathname}${qs ? `?${qs}` : ''}${window.location.hash || ''}`;
  window.history.replaceState({}, document.title, next);
  return true;
}

function redirectToLogin(error) {
  const base = 'login.html';
  window.location.href = error ? `${base}?error=${encodeURIComponent(error)}` : base;
}

async function requireAuthOrRedirect() {
  captureTokenFromUrl();
  const cfg = await fetchAuthConfig();
  if (!cfg.auth_enabled) {
    return { authenticated: true, user: null };
  }
  try {
    const me = await fetchAuthMe();
    if (me.authenticated) return me;
    if (cfg.allow_local_bypass) return me;
  } catch (e) {
    console.warn('auth check failed', e);
  }
  redirectToLogin();
  return null;
}

function startOidcLogin() {
  window.location.href = `${authApiBase()}/auth/oidc/login`;
}

function startSamlLogin() {
  window.location.href = `${authApiBase()}/auth/saml/login`;
}

async function loadIdentitySettings() {
  const res = await fetch(`${authApiBase()}/settings/identity`, { headers: authHeaders() });
  const data = await res.json();
  if (!res.ok) throw new Error(data.message || 'Failed to load identity settings');
  return data.data;
}

async function saveIdentitySettings(body) {
  const res = await fetch(`${authApiBase()}/settings/identity`, {
    method: 'PUT',
    headers: authHeaders({ 'Content-Type': 'application/json' }),
    body: JSON.stringify(body),
  });
  const data = await res.json();
  if (!res.ok) throw new Error(data.message || 'Failed to save identity settings');
  return data.data;
}

async function loadSsoSettings() {
  const res = await fetch(`${authApiBase()}/settings/sso`, { headers: authHeaders() });
  const data = await res.json();
  if (!res.ok) throw new Error(data.message || 'Failed to save SSO settings');
  return data.data;
}

async function saveSsoSettings(body) {
  const res = await fetch(`${authApiBase()}/settings/sso`, {
    method: 'PUT',
    headers: authHeaders({ 'Content-Type': 'application/json' }),
    body: JSON.stringify(body),
  });
  const data = await res.json();
  if (!res.ok) throw new Error(data.message || 'Failed to save SSO settings');
  return data.data;
}

function initUserMenu() {
  const userChip = document.getElementById('authUserChip');
  const signOutBtn = document.getElementById('authSignOutBtn');
  const settingsBtn = document.getElementById('openSettingsBtn');
  const user = getAuthUser();
  if (userChip && user) {
    const label = user.name || user.email || user.sub || 'Signed in';
    userChip.textContent = label;
    userChip.title = `${label} (${user.role || 'operator'})`;
    userChip.classList.remove('hidden');
  }
  if (signOutBtn && user) {
    signOutBtn.classList.remove('hidden');
  }
  if (settingsBtn && user && !isAdmin(user)) {
    settingsBtn.classList.add('hidden');
  }
  signOutBtn?.addEventListener('click', async () => {
    await logout();
    redirectToLogin();
  });
}

function initSettingsModal() {
  const openBtn = document.getElementById('openSettingsBtn');
  const modal = document.getElementById('settingsModal');
  const closeBtn = document.getElementById('settingsCloseBtn');
  const saveIdentityBtn = document.getElementById('saveIdentityBtn');
  const saveSsoBtn = document.getElementById('saveSsoBtn');
  const copySamlMetaBtn = document.getElementById('copySamlMetadataBtn');
  if (!modal) return;

  const tabs = modal.querySelectorAll('[data-settings-tab]');
  tabs.forEach((tab) => {
    tab.addEventListener('click', () => {
      tabs.forEach((t) => t.classList.toggle('active', t === tab));
      modal.querySelectorAll('[data-settings-panel]').forEach((panel) => {
        panel.classList.toggle('hidden', panel.dataset.settingsPanel !== tab.dataset.settingsTab);
      });
    });
  });

  async function populate() {
    try {
      const [identity, sso] = await Promise.all([loadIdentitySettings(), loadSsoSettings()]);
      document.getElementById('identityAllowBypass').checked = identity.allow_local_bypass;
      document.getElementById('identityDefaultRole').value = identity.default_role;
      document.getElementById('identitySessionHours').value = identity.session_hours;
      document.getElementById('identityRoleClaim').value = identity.role_claim || 'groups';
      document.getElementById('identityAdminRoles').value = (identity.admin_roles || []).join(', ');
      document.getElementById('identityAdminEmails').value = (identity.admin_emails || []).join(', ');
      document.getElementById('oidcEnabled').checked = sso.oidc.enabled;
      document.getElementById('oidcIssuerUrl').value = sso.oidc.issuer_url;
      document.getElementById('oidcClientId').value = sso.oidc.client_id;
      document.getElementById('oidcClientSecret').value = '';
      document.getElementById('oidcClientSecret').placeholder = sso.oidc.client_secret_set
        ? '•••••••• (unchanged if empty)'
        : 'Client secret';
      document.getElementById('oidcScopes').value = (sso.oidc.scopes || []).join(' ');
      document.getElementById('oidcButtonLabel').value = sso.oidc.button_label || 'Sign in with SSO';
      document.getElementById('samlEnabled').checked = sso.saml.enabled;
      document.getElementById('samlEntityId').value = sso.saml.entity_id;
      document.getElementById('samlSsoUrl').value = sso.saml.sso_url;
      document.getElementById('samlMetadataUrl').value = sso.saml.metadata_url;
      document.getElementById('samlCertificatePem').value = sso.saml.certificate_pem;
      document.getElementById('samlNameIdFormat').value = sso.saml.name_id_format;
      document.getElementById('samlButtonLabel').value = sso.saml.button_label || 'Sign in with SAML';
    } catch (e) {
      alert(e.message || String(e));
    }
  }

  openBtn?.addEventListener('click', () => {
    modal.classList.remove('hidden');
    populate();
  });
  closeBtn?.addEventListener('click', () => modal.classList.add('hidden'));
  modal.addEventListener('click', (ev) => {
    if (ev.target === modal) modal.classList.add('hidden');
  });

  saveIdentityBtn?.addEventListener('click', async () => {
    try {
      await saveIdentitySettings({
        allow_local_bypass: document.getElementById('identityAllowBypass').checked,
        default_role: document.getElementById('identityDefaultRole').value.trim(),
        session_hours: Number(document.getElementById('identitySessionHours').value) || 24,
        role_claim: document.getElementById('identityRoleClaim').value.trim() || 'groups',
        admin_roles: document.getElementById('identityAdminRoles').value.split(',').map((s) => s.trim()).filter(Boolean),
        admin_emails: document.getElementById('identityAdminEmails').value.split(',').map((s) => s.trim()).filter(Boolean),
      });
      alert('Identity settings saved');
    } catch (e) {
      alert(e.message || String(e));
    }
  });

  saveSsoBtn?.addEventListener('click', async () => {
    try {
      await saveSsoSettings({
        oidc: {
          enabled: document.getElementById('oidcEnabled').checked,
          issuer_url: document.getElementById('oidcIssuerUrl').value.trim(),
          client_id: document.getElementById('oidcClientId').value.trim(),
          client_secret: document.getElementById('oidcClientSecret').value,
          scopes: document.getElementById('oidcScopes').value.trim().split(/\s+/).filter(Boolean),
          button_label: document.getElementById('oidcButtonLabel').value.trim(),
        },
        saml: {
          enabled: document.getElementById('samlEnabled').checked,
          entity_id: document.getElementById('samlEntityId').value.trim(),
          sso_url: document.getElementById('samlSsoUrl').value.trim(),
          metadata_url: document.getElementById('samlMetadataUrl').value.trim(),
          certificate_pem: document.getElementById('samlCertificatePem').value.trim(),
          name_id_format: document.getElementById('samlNameIdFormat').value.trim(),
          button_label: document.getElementById('samlButtonLabel').value.trim(),
        },
      });
      alert('SSO settings saved');
    } catch (e) {
      alert(e.message || String(e));
    }
  });

  copySamlMetaBtn?.addEventListener('click', async () => {
    const url = `${authApiBase()}/settings/sso/saml/metadata`;
    try {
      const res = await fetch(url);
      const xml = await res.text();
      await navigator.clipboard.writeText(xml);
      alert('SAML SP metadata copied to clipboard');
    } catch (e) {
      alert(e.message || String(e));
    }
  });
}

window.GuestKitAuth = {
  getAuthToken,
  setAuthToken,
  clearAuth,
  getAuthUser,
  isAdmin,
  authHeaders,
  fetchAuthConfig,
  fetchAuthMe,
  localBypassLogin,
  logout,
  captureTokenFromUrl,
  redirectToLogin,
  requireAuthOrRedirect,
  startOidcLogin,
  startSamlLogin,
  initSettingsModal,
  initUserMenu,
};
