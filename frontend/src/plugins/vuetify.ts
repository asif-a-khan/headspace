import "vuetify/styles";
import "@mdi/font/css/materialdesignicons.css";
import { createVuetify } from "vuetify";

export default createVuetify({
  theme: {
    defaultTheme: localStorage.getItem("headspace-theme") || "light",
    themes: {
      light: {
        colors: {
          primary: "#0E90D9",
          "on-primary": "#FFFFFF",
          secondary: "#6B7280",
          "on-secondary": "#FFFFFF",
          error: "#EF4444",
          warning: "#F59E0B",
          info: "#3B82F6",
          success: "#10B981",
          background: "#F3F4F6",
          surface: "#FFFFFF",
          "on-background": "#1F2937",
          "on-surface": "#1F2937",
          "surface-variant": "#F9FAFB",
        },
        variables: {
          "border-color": "#E5E7EB",
          "border-opacity": 0.2,
        },
      },
      dark: {
        colors: {
          primary: "#0E90D9",
          "on-primary": "#FFFFFF",
          secondary: "#9CA3AF",
          "on-secondary": "#FFFFFF",
          error: "#EF4444",
          warning: "#F59E0B",
          info: "#3B82F6",
          success: "#10B981",
          background: "#030712",
          surface: "#111827",
          "on-background": "#FFFFFF",
          "on-surface": "#FFFFFF",
          "surface-variant": "#1F2937",
        },
        variables: {
          "border-color": "#1F2937",
          "border-opacity": 0.2,
        },
      },
    },
  },
  defaults: {
    VCard: {
      rounded: "lg",
      elevation: 0,
    },
    VDataTable: {
      density: "comfortable",
    },
    VTextField: {
      variant: "outlined",
      density: "comfortable",
    },
    VSelect: {
      variant: "outlined",
      density: "comfortable",
    },
    VTextarea: {
      variant: "outlined",
      density: "comfortable",
    },
    VSwitch: {
      color: "primary",
    },
    VBtn: {
      variant: "flat",
      rounded: "md",
    },
    VChip: {
      rounded: "md",
    },
    VNavigationDrawer: {
      elevation: 0,
    },
    VAppBar: {
      elevation: 0,
    },
  },
});
