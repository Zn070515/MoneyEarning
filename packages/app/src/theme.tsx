import React, { createContext, useContext, useState, useEffect, useCallback } from "react";

export interface AccessibilitySettings {
  largeFont: boolean;
  highContrast: boolean;
}

interface ThemeContextValue {
  settings: AccessibilitySettings;
  toggleLargeFont: () => void;
  toggleHighContrast: () => void;
  reset: () => void;
}

const STORAGE_KEY = "me_accessibility";

const defaults: AccessibilitySettings = {
  largeFont: false,
  highContrast: false,
};

function load(): AccessibilitySettings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw);
      return { ...defaults, ...parsed };
    }
  } catch {
    // corrupted — use defaults
  }
  return { ...defaults };
}

function save(s: AccessibilitySettings) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(s));
  } catch {
    // quota exceeded — ignore
  }
}

function applyToDOM(s: AccessibilitySettings) {
  const root = document.documentElement;
  if (s.largeFont) {
    root.setAttribute("data-large-font", "");
  } else {
    root.removeAttribute("data-large-font");
  }
  if (s.highContrast) {
    root.setAttribute("data-high-contrast", "");
  } else {
    root.removeAttribute("data-high-contrast");
  }
}

const ThemeContext = createContext<ThemeContextValue | null>(null);

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  const [settings, setSettings] = useState<AccessibilitySettings>(load);

  useEffect(() => {
    applyToDOM(settings);
  }, [settings]);

  const toggleLargeFont = useCallback(() => {
    setSettings((prev) => {
      const next = { ...prev, largeFont: !prev.largeFont };
      save(next);
      return next;
    });
  }, []);

  const toggleHighContrast = useCallback(() => {
    setSettings((prev) => {
      const next = { ...prev, highContrast: !prev.highContrast };
      save(next);
      return next;
    });
  }, []);

  const reset = useCallback(() => {
    save(defaults);
    setSettings({ ...defaults });
  }, []);

  return (
    <ThemeContext.Provider value={{ settings, toggleLargeFont, toggleHighContrast, reset }}>
      {children}
    </ThemeContext.Provider>
  );
}

export function useTheme(): ThemeContextValue {
  const ctx = useContext(ThemeContext);
  if (!ctx) {
    // If no provider, return no-op defaults (graceful degradation)
    return {
      settings: { ...defaults },
      toggleLargeFont: () => {},
      toggleHighContrast: () => {},
      reset: () => {},
    };
  }
  return ctx;
}
