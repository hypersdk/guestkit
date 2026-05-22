/**
 * Zyvor product suite branding.
 * Place in src/components/ (or sdk/components/). Copy zyvor-logo.png to app public/.
 * Vite: respects import.meta.env.BASE_URL (e.g. /web/dashboard/ for HyperSDK).
 */
import React from 'react';

export const ZYVOR_URL = 'https://zyvor.dev';
export const ZYVOR_COPY = '© @zyvor 2026';

/** Resolve logo URL for Vite, subpath-hosted SPAs, and plain static HTML. */
export function zyvorLogoSrc(): string {
  const viteBase =
    typeof import.meta !== 'undefined'
      ? (import.meta as { env?: { BASE_URL?: string } }).env?.BASE_URL
      : undefined;
  if (viteBase) {
    const base = String(viteBase);
    return `${base.endsWith('/') ? base : `${base}/`}zyvor-logo.png`;
  }
  if (typeof window !== 'undefined' && window.location?.pathname) {
    const parts = window.location.pathname.split('/').filter(Boolean);
    if (parts.length > 0) {
      return `/${parts[0]}/zyvor-logo.png`;
    }
  }
  return '/zyvor-logo.png';
}

const linkStyle: React.CSSProperties = {
  color: '#f97316',
  textDecoration: 'none',
  fontWeight: 600,
};

const linkHover = (e: React.MouseEvent<HTMLAnchorElement>) => {
  e.currentTarget.style.color = '#fb923c';
};

const linkLeave = (e: React.MouseEvent<HTMLAnchorElement>) => {
  e.currentTarget.style.color = '#f97316';
};

const logoImgStyle = (height: number): React.CSSProperties => ({
  height,
  width: 'auto',
  display: 'block',
  maxWidth: 'min(200px, 42vw)',
});

const logoWrapStyle: React.CSSProperties = {
  display: 'inline-flex',
  alignItems: 'center',
  justifyContent: 'center',
  padding: '6px 12px',
  borderRadius: 8,
  background: 'rgba(255, 255, 255, 0.96)',
  boxShadow: '0 1px 3px rgba(0,0,0,0.25)',
  lineHeight: 0,
};

/** Prominent strip for Help, shortcuts, and about panels — logo stays visible on dark UI. */
export function ZyvorHelpStrip({
  product,
  className = '',
}: {
  product?: string;
  className?: string;
}) {
  return (
    <div
      className={`zyvor-help-strip ${className}`.trim()}
      style={{
        display: 'flex',
        flexWrap: 'wrap',
        alignItems: 'center',
        gap: '12px 16px',
        padding: '12px 16px',
        marginBottom: 16,
        borderRadius: 10,
        border: '1px solid rgba(249, 115, 22, 0.35)',
        background: 'linear-gradient(135deg, rgba(30,41,59,0.95) 0%, rgba(15,23,42,0.98) 100%)',
      }}
      role="complementary"
      aria-label="Zyvor product suite"
    >
      <a
        href={ZYVOR_URL}
        target="_blank"
        rel="noopener noreferrer"
        title="Zyvor — zyvor.dev"
        style={logoWrapStyle}
      >
        <img src={zyvorLogoSrc()} alt="Zyvor" style={logoImgStyle(32)} />
      </a>
      <div style={{ fontSize: 13, color: 'rgba(226, 232, 240, 0.95)', lineHeight: 1.45 }}>
        <span style={{ fontWeight: 600, color: '#f8fafc' }}>Zyvor</span>
        {' · '}
        <a
          href={ZYVOR_URL}
          target="_blank"
          rel="noopener noreferrer"
          style={linkStyle}
          onMouseEnter={linkHover}
          onMouseLeave={linkLeave}
        >
          zyvor.dev
        </a>
        <span style={{ opacity: 0.75 }}> · {ZYVOR_COPY}</span>
        {product ? (
          <span style={{ display: 'block', opacity: 0.6, fontSize: 12, marginTop: 2 }}>{product}</span>
        ) : null}
      </div>
    </div>
  );
}

/** Compact footer for app shells. */
export function ZyvorFooter({
  product,
  className = '',
}: {
  product?: string;
  className?: string;
}) {
  return (
    <footer
      className={`zyvor-footer ${className}`.trim()}
      style={{
        display: 'flex',
        flexWrap: 'wrap',
        alignItems: 'center',
        justifyContent: 'center',
        gap: '10px 16px',
        padding: '14px 20px',
        marginTop: 'auto',
        borderTop: '1px solid rgba(148, 163, 184, 0.15)',
        fontSize: '12px',
        color: 'rgba(148, 163, 184, 0.9)',
        background: 'transparent',
      }}
      role="contentinfo"
    >
      <a
        href={ZYVOR_URL}
        target="_blank"
        rel="noopener noreferrer"
        title="Zyvor — zyvor.dev"
        style={logoWrapStyle}
      >
        <img src={zyvorLogoSrc()} alt="Zyvor" style={logoImgStyle(28)} />
      </a>
      <span>
        <a
          href={ZYVOR_URL}
          target="_blank"
          rel="noopener noreferrer"
          style={{ ...linkStyle, fontSize: 12 }}
          onMouseEnter={linkHover}
          onMouseLeave={linkLeave}
        >
          zyvor.dev
        </a>
        <span style={{ opacity: 0.75, marginLeft: 6 }}>{ZYVOR_COPY}</span>
        {product ? (
          <span style={{ opacity: 0.55, marginLeft: 8 }}>· {product}</span>
        ) : null}
      </span>
    </footer>
  );
}

/** Small logo in nav/header. */
export function ZyvorLogoMark({ height = 24 }: { height?: number }) {
  return (
    <a
      href={ZYVOR_URL}
      target="_blank"
      rel="noopener noreferrer"
      title="Zyvor — zyvor.dev"
      style={logoWrapStyle}
    >
      <img src={zyvorLogoSrc()} alt="Zyvor" style={logoImgStyle(height)} />
    </a>
  );
}

export default ZyvorFooter;
