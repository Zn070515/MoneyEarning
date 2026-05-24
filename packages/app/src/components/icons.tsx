import React from "react";

interface IconProps {
  size?: number;
  className?: string;
}

function icon(path: React.ReactNode, viewBox = "0 0 24 24") {
  return ({ size = 20, className }: IconProps) => (
    <svg
      width={size}
      height={size}
      viewBox={viewBox}
      fill="none"
      stroke="currentColor"
      strokeWidth={1.8}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
      aria-hidden="true"
    >
      {path}
    </svg>
  );
}

/* ── Navigation ── */
export const IconDashboard = icon(
  <>
    <rect x="3" y="3" width="7" height="7" rx="1" />
    <rect x="14" y="3" width="7" height="7" rx="1" />
    <rect x="3" y="14" width="7" height="7" rx="1" />
    <rect x="14" y="14" width="7" height="7" rx="1" />
  </>
);

export const IconChart = icon(
  <>
    <rect x="3" y="3" width="18" height="18" rx="2" />
    <path d="M3 16l5-5 3 3 5-7 5 6" />
  </>
);

export const IconBacktest = icon(
  <>
    <polygon points="5,3 19,12 5,21" />
  </>
);

export const IconScanner = icon(
  <>
    <circle cx="10" cy="10" r="7" />
    <line x1="15" y1="15" x2="21" y2="21" />
  </>
);

export const IconPortfolio = icon(
  <>
    <rect x="2" y="3" width="20" height="18" rx="2" />
    <path d="M2 10h20" />
    <path d="M12 10v11" />
  </>
);

export const IconReview = icon(
  <>
    <rect x="3" y="3" width="18" height="18" rx="2" />
    <line x1="7" y1="8" x2="17" y2="8" />
    <line x1="7" y1="12" x2="17" y2="12" />
    <line x1="7" y1="16" x2="12" y2="16" />
  </>
);

export const IconEditor = icon(
  <>
    <polyline points="16,18 22,12 16,6" />
    <polyline points="8,6 2,12 8,18" />
    <line x1="12" y1="3" x2="12" y2="21" />
  </>
);

export const IconSettings = icon(
  <>
    <circle cx="12" cy="12" r="4" />
    <path d="M12 2v3" />
    <path d="M12 19v3" />
    <path d="M2 12h3" />
    <path d="M19 12h3" />
    <path d="M5.64 5.64l1.41 1.41" />
    <path d="M16.95 16.95l1.41 1.41" />
    <path d="M18.36 5.64l-1.41 1.41" />
    <path d="M7.05 16.95l-1.41 1.41" />
  </>
);

/* ── Window Controls ── */
export const IconMinimize = icon(
  <line x1="6" y1="12" x2="18" y2="12" />
);

export const IconMaximize = icon(
  <rect x="5" y="5" width="14" height="14" rx="1" />
);

export const IconRestore = icon(
  <>
    <rect x="7" y="5" width="12" height="12" rx="1" />
    <rect x="5" y="9" width="12" height="12" rx="1" />
  </>
);

export const IconClose = icon(
  <>
    <line x1="6" y1="6" x2="18" y2="18" />
    <line x1="6" y1="18" x2="18" y2="6" />
  </>
);

/* ── Actions ── */
export const IconPlay = icon(
  <polygon points="6,4 20,12 6,20" />
);

export const IconDownload = icon(
  <>
    <path d="M12 3v14" />
    <polyline points="7,12 12,17 17,12" />
    <path d="M3 21h18" />
  </>
);

export const IconUpload = icon(
  <>
    <path d="M12 21V7" />
    <polyline points="5,12 12,5 19,12" />
    <path d="M3 21h18" />
  </>
);

export const IconRefresh = icon(
  <>
    <path d="M3 12a9 9 0 0 1 9-9 9 9 0 0 1 6 2.5" />
    <path d="M21 12a9 9 0 0 1-9 9 9 9 0 0 1-6-2.5" />
    <polyline points="18,2 18,6 22,6" />
    <polyline points="6,22 6,18 2,18" />
  </>
);

export const IconPlus = icon(
  <>
    <line x1="12" y1="5" x2="12" y2="19" />
    <line x1="5" y1="12" x2="19" y2="12" />
  </>
);

export const IconTrash = icon(
  <>
    <polyline points="4,7 20,7" />
    <path d="M8 7V5a1 1 0 0 1 1-1h6a1 1 0 0 1 1 1v2" />
    <path d="M6 7l1 12h10l1-12" />
  </>
);

export const IconEdit = icon(
  <>
    <path d="M12 4h4a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2v-4" />
    <path d="M20 4L12 12l-3 1 1-3L20 4z" />
  </>
);

export const IconCheck = icon(
  <polyline points="4,12 9,17 20,6" />
);

export const IconX = icon(
  <>
    <line x1="6" y1="6" x2="18" y2="18" />
    <line x1="6" y1="18" x2="18" y2="6" />
  </>
);

/* ── Data ── */
export const IconTrendUp = icon(
  <>
    <polyline points="22,7 14,15 10,11 2,19" />
    <polyline points="16,7 22,7 22,13" />
  </>
);

export const IconTrendDown = icon(
  <>
    <polyline points="2,7 10,15 14,11 22,19" />
    <polyline points="16,19 22,19 22,13" />
  </>
);

export const IconDatabase = icon(
  <>
    <ellipse cx="12" cy="5" rx="9" ry="3" />
    <path d="M3 5v14c0 1.66 4.03 3 9 3s9-1.34 9-3V5" />
    <path d="M3 12c0 1.66 4.03 3 9 3s9-1.34 9-3" />
  </>
);

export const IconShield = icon(
  <>
    <path d="M12 2L3 7v5c0 5.25 3.83 10.15 9 12 5.17-1.85 9-6.75 9-12V7l-9-5z" />
    <polyline points="8,12 11,15 16,9" />
  </>
);

export const IconAlert = icon(
  <>
    <path d="M12 2L2 22h20L12 2z" />
    <line x1="12" y1="10" x2="12" y2="16" />
    <circle cx="12" cy="19" r="0.5" fill="currentColor" stroke="none" />
  </>
);

export const IconInfo = icon(
  <>
    <circle cx="12" cy="12" r="10" />
    <line x1="12" y1="10" x2="12" y2="16" />
    <circle cx="12" cy="7.5" r="0.5" fill="currentColor" stroke="none" />
  </>
);

export const IconCommand = icon(
  <>
    <path d="M8 6a3 3 0 1 0 0 6h8a3 3 0 1 0 0-6H8z" />
    <path d="M8 18a3 3 0 1 0 3-3H8v3z" />
    <path d="M16 6a3 3 0 1 0 3 3V6h-3z" />
    <path d="M16 18a3 3 0 1 1-3-3h3v3z" />
  </>
);

export const IconSearch = IconScanner; // alias

/* ── Finance ── */
export const IconDollar = icon(
  <>
    <line x1="12" y1="3" x2="12" y2="21" />
    <path d="M16 6H9.5A2.5 2.5 0 0 0 7 8.5v0A2.5 2.5 0 0 0 9.5 11h5a2.5 2.5 0 0 1 0 5H8" />
  </>
);

export const IconPercent = icon(
  <>
    <line x1="19" y1="5" x2="5" y2="19" />
    <circle cx="7" cy="7" r="2" />
    <circle cx="17" cy="17" r="2" />
  </>
);

export const IconTarget = icon(
  <>
    <circle cx="12" cy="12" r="10" />
    <circle cx="12" cy="12" r="5" />
    <circle cx="12" cy="12" r="1.5" fill="currentColor" stroke="none" />
  </>
);
