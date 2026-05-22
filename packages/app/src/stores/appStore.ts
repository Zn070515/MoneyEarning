import { create } from "zustand";
import { persist } from "zustand/middleware";

export interface AppState {
  // License
  licenseValid: boolean;
  licenseExpiry: string | null;
  setLicense: (valid: boolean, expiry: string | null) => void;

  // Accessibility
  largeFont: boolean;
  highContrast: boolean;
  toggleLargeFont: () => void;
  toggleHighContrast: () => void;

  // Navigation
  currentPage: string;
  navigate: (page: string) => void;

  // Selected stock
  selectedStockId: number | null;
  selectedStockCode: string | null;
  selectedStockName: string | null;
  selectStock: (id: number, code: string, name: string) => void;
}

export const useAppStore = create<AppState>()(
  persist(
    (set) => ({
      licenseValid: false,
      licenseExpiry: null,
      setLicense: (valid, expiry) => set({ licenseValid: valid, licenseExpiry: expiry }),

      largeFont: false,
      highContrast: false,
      toggleLargeFont: () =>
        set((s) => {
          const next = !s.largeFont;
          document.documentElement.setAttribute("data-large-font", next ? "true" : "false");
          return { largeFont: next };
        }),
      toggleHighContrast: () =>
        set((s) => {
          const next = !s.highContrast;
          document.documentElement.setAttribute("data-high-contrast", next ? "true" : "false");
          return { highContrast: next };
        }),

      currentPage: "/",
      navigate: (page) => set({ currentPage: page }),

      selectedStockId: null,
      selectedStockCode: null,
      selectedStockName: null,
      selectStock: (id, code, name) =>
        set({ selectedStockId: id, selectedStockCode: code, selectedStockName: name }),
    }),
    {
      name: "me-app-state",
      partialize: (state) => ({
        largeFont: state.largeFont,
        highContrast: state.highContrast,
        licenseValid: state.licenseValid,
        licenseExpiry: state.licenseExpiry,
      }),
    },
  ),
);
