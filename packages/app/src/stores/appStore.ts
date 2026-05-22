import { create } from "zustand";
import { persist } from "zustand/middleware";

export interface AppState {
  // License
  licenseValid: boolean;
  licenseExpiry: string | null;
  setLicense: (valid: boolean, expiry: string | null) => void;

  // License tier
  licenseTier: string;
  setLicenseTier: (tier: string) => void;

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
      licenseTier: "free",
      toggleLargeFont: () =>
        set((s) => {
          const next = !s.largeFont;
          if (next) {
            document.documentElement.setAttribute("data-large-font", "true");
          } else {
            document.documentElement.removeAttribute("data-large-font");
          }
          return { largeFont: next };
        }),
      toggleHighContrast: () =>
        set((s) => {
          const next = !s.highContrast;
          if (next) {
            document.documentElement.setAttribute("data-high-contrast", "true");
          } else {
            document.documentElement.removeAttribute("data-high-contrast");
          }
          return { highContrast: next };
        }),
      setLicenseTier: (tier) => set({ licenseTier: tier }),

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
        licenseTier: state.licenseTier,
        licenseValid: state.licenseValid,
        licenseExpiry: state.licenseExpiry,
      }),
    },
  ),
);
