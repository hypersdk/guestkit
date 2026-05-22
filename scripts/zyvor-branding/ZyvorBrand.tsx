/**
 * Zyvor product suite branding — place in src/components/ or components/
 * Logo: copy zyvor-logo.png to your app public/ folder (Vite: /zyvor-logo.png).
 */
import React from 'react';

const ZYVOR_URL = 'https://zyvor.dev';
const ZYVOR_COPY = '© @zyvor 2026';

const linkStyle: React.CSSProperties = {
  color: 'inherit',
  textDecoration: 'none',
  opacity: 0.9,
};

const linkHover = (e: React.MouseEvent<HTMLAnchorElement>) => {
  e.currentTarget.style.opacity = '1';
  e.currentTarget.style.color = '#f97316';
};

const linkLeave = (e: React.MouseEvent<HTMLAnchorElement>) => {
  e.currentTarget.style.opacity = '0.9';
  e.currentTarget.style.color = 'inherit';
};

/** Compact footer for app shells — use once per layout below <Outlet /> or page content. */
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
        style={{ display: 'flex', alignItems: 'center', lineHeight: 0 }}
      >
        <img
          src="/zyvor-logo.png"
          alt="Zyvor"
          style={{ height: 26, width: 'auto', display: 'block' }}
        />
      </a>
      <span>
        <a
          href={ZYVOR_URL}
          target="_blank"
          rel="noopener noreferrer"
          style={linkStyle}
          onMouseEnter={linkHover}
          onMouseLeave={linkLeave}
        >
          {ZYVOR_COPY}
        </a>
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
      style={{ display: 'inline-flex', alignItems: 'center', lineHeight: 0 }}
    >
      <img
        src="/zyvor-logo.png"
        alt="Zyvor"
        style={{ height, width: 'auto', display: 'block' }}
      />
    </a>
  );
}

export default ZyvorFooter;
