import React, { createContext, useContext, useEffect, useCallback } from "react";
import { useAppStore } from "./stores/appStore";

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

function applyToDOM(s: AccessibilitySettings) {
  const root = document.documentElement;
  if (s.largeFont) {
    root.setAttribute("data-large-font", "true");
  } else {
    root.removeAttribute("data-large-font");
  }
  if (s.highContrast) {
    root.setAttribute("data-high-contrast", "true");
  } else {
    root.removeAttribute("data-high-contrast");
  }
}

const ThemeContext = createContext<ThemeContextValue | null>(null);

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  const largeFont = useAppStore((s) => s.largeFont);
  const highContrast = useAppStore((s) => s.highContrast);

  // Apply on mount and on every change
  useEffect(() => {
    applyToDOM({ largeFont, highContrast });
  }, [largeFont, highContrast]);

  const toggleLargeFont = useCallback(() => {
    useAppStore.getState().toggleLargeFont();
  }, []);

  const toggleHighContrast = useCallback(() => {
    useAppStore.getState().toggleHighContrast();
  }, []);

  const reset = useCallback(() => {
    const store = useAppStore.getState();
    store.toggleLargeFont();
    if (store.largeFont) store.toggleLargeFont(); // toggle back if was on
    if (store.highContrast) store.toggleHighContrast();
  }, []);

  return (
    <ThemeContext.Provider value={{ settings: { largeFont, highContrast }, toggleLargeFont, toggleHighContrast, reset }}>
      {children}
    </ThemeContext.Provider>
  );
}

export function useTheme(): ThemeContextValue {
  const ctx = useContext(ThemeContext);
  if (!ctx) {
    return {
      settings: { largeFont: false, highContrast: false },
      toggleLargeFont: () => {},
      toggleHighContrast: () => {},
      reset: () => {},
    };
  }
  return ctx;
}
