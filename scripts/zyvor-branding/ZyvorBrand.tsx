/**
 * Zyvor product suite branding — text only (no logo image).
 * Embed ZyvorInline in headers/footers; use ZyvorFooter for a minimal app footer line.
 */
import React from 'react';

export const ZYVOR_URL = 'https://zyvor.dev';
export const ZYVOR_COPY = '© @zyvor 2026';

const ORANGE = '#f97316';
const MUTED = 'rgba(148, 163, 184, 0.75)';

const linkStyle: React.CSSProperties = {
  color: ORANGE,
  textDecoration: 'none',
  fontWeight: 500,
};

const linkHover = (e: React.MouseEvent<HTMLAnchorElement>) => {
  e.currentTarget.style.color = '#fb923c';
};

const linkLeave = (e: React.MouseEvent<HTMLAnchorElement>) => {
  e.currentTarget.style.color = ORANGE;
};

/** Single embedded line: zyvor.dev · © @zyvor 2026 · Product */
export function ZyvorInline({
  product,
  className = '',
  style,
}: {
  product?: string;
  className?: string;
  style?: React.CSSProperties;
}) {
  return (
    <span
      className={`zyvor-inline whitespace-normal ${className}`.trim()}
      style={{
        fontSize: '12px',
        lineHeight: 1.5,
        color: MUTED,
        ...style,
      }}
    >
      <a
        href={ZYVOR_URL}
        target="_blank"
        rel="noopener noreferrer"
        style={{ ...linkStyle, fontWeight: 600 }}
        onMouseEnter={linkHover}
        onMouseLeave={linkLeave}
      >
        zyvor.dev
      </a>
      <span aria-hidden> · </span>
      <span>{ZYVOR_COPY}</span>
      {product ? (
        <>
          <span aria-hidden> · </span>
          <span style={{ opacity: 0.85 }}>{product}</span>
        </>
      ) : null}
    </span>
  );
}

/** Minimal footer — one inline line, no logo, no extra background strip. */
export function ZyvorFooter({
  product,
  className = '',
}: {
  product?: string;
  className?: string;
}) {
  return (
    <footer
      className={`zyvor-footer shrink-0 py-2 text-center ${className}`.trim()}
      style={{
        marginTop: 'auto',
        background: 'transparent',
        border: 'none',
      }}
      role="contentinfo"
    >
      <ZyvorInline product={product} />
    </footer>
  );
}

/** @deprecated Use ZyvorInline — same inline text, no separate box. */
export function ZyvorHelpStrip({
  product,
  className = '',
}: {
  product?: string;
  className?: string;
}) {
  return <ZyvorInline product={product} className={className} />;
}

/** Header mark: zyvor.dev link. */
export function ZyvorLogoMark({ className = '' }: { className?: string }) {
  return (
    <a
      href={ZYVOR_URL}
      target="_blank"
      rel="noopener noreferrer"
      title="zyvor.dev"
      className={className}
      style={{
        fontWeight: 600,
        fontSize: '13px',
        color: ORANGE,
        textDecoration: 'none',
      }}
      onMouseEnter={linkHover}
      onMouseLeave={linkLeave}
    >
      zyvor.dev
    </a>
  );
}

export default ZyvorFooter;
