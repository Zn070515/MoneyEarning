import { create } from "zustand";
import { persist } from "zustand/middleware";
import { invoke } from "@tauri-apps/api/core";

export interface LicenseStatus {
  valid: boolean;
  tier: string;
  expiry: string | null;
  trial_days_left: number | null;
}

export interface AppState {
  // License — synced from backend, NOT persisted to localStorage
  licenseValid: boolean;
  licenseExpiry: string | null;
  licenseTier: string;
  trialDaysLeft: number | null;
  refreshLicense: () => Promise<LicenseStatus>;

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
      licenseTier: "free",
      trialDaysLeft: null,
      refreshLicense: async () => {
        try {
          const s = await invoke<LicenseStatus>("check_license");
          set({
            licenseValid: s.valid,
            licenseExpiry: s.expiry,
            licenseTier: s.tier,
            trialDaysLeft: s.trial_days_left,
          });
          return s;
        } catch {
          set({
            licenseValid: false,
            licenseExpiry: null,
            licenseTier: "free",
            trialDaysLeft: null,
          });
          return {
            valid: false,
            tier: "free",
            expiry: null,
            trial_days_left: null,
          } as LicenseStatus;
        }
      },

      largeFont: false,
      highContrast: false,
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
        // License fields intentionally excluded — backend is the sole source of truth
      }),
    },
  ),
);
