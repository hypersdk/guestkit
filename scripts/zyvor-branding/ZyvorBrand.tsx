/**
 * Zyvor suite branding — text only.
 * Footer: zyvor.dev · HyperSDK · © 2026
 * Header: use ZyvorLogoMark (link only) — no copyright, no product name.
 */
import React from 'react';

export const ZYVOR_URL = 'https://zyvor.dev';
export const ZYVOR_BRAND = 'HyperSDK';
export const ZYVOR_COPY = '© 2026';
export const ZYVOR_LINE = `zyvor.dev · ${ZYVOR_BRAND} · ${ZYVOR_COPY}`;

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

const sep = <span aria-hidden> · </span>;

function ZyvorDevLink({ className = '' }: { className?: string }) {
  return (
    <a
      href={ZYVOR_URL}
      target="_blank"
      rel="noopener noreferrer"
      className={className}
      style={{ ...linkStyle, fontWeight: 600 }}
      onMouseEnter={linkHover}
      onMouseLeave={linkLeave}
    >
      zyvor.dev
    </a>
  );
}

type BrandProps = {
  /** @deprecated Ignored — product name is not shown (redundant in-app). */
  product?: string;
  className?: string;
  style?: React.CSSProperties;
  /** Include © line — default false (use ZyvorFooter for copyright). */
  includeCopyright?: boolean;
};

/** Compact line: zyvor.dev · HyperSDK (optional © when includeCopyright). */
export function ZyvorInline({
  className = '',
  style,
  includeCopyright = false,
}: BrandProps) {
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
      <ZyvorDevLink />
      {sep}
      <span>{ZYVOR_BRAND}</span>
      {includeCopyright ? (
        <>
          {sep}
          <span>{ZYVOR_COPY}</span>
        </>
      ) : null}
    </span>
  );
}

/** Page footer — full line with copyright. */
export function ZyvorFooter({ className = '' }: { className?: string }) {
  return (
    <footer
      className={`zyvor-footer shrink-0 py-2 text-center ${className}`.trim()}
      style={{
        marginTop: 'auto',
        background: 'transparent',
        border: 'none',
      }}
      role="contentinfo"
    ></footer>
  );
}

/** @deprecated Use ZyvorFooter or ZyvorInline. */
export function ZyvorHelpStrip(props: BrandProps) {
  return;
}

/** Header: zyvor.dev link only. */
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
