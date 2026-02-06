import "vuetify/styles";
import "@mdi/font/css/materialdesignicons.css";
import { createVuetify } from "vuetify";

export default createVuetify({
  theme: {
    defaultTheme: "light",
    themes: {
      light: {
        colors: {
          primary: "#7C3AED",
          secondary: "#6366F1",
          accent: "#8B5CF6",
          error: "#EF4444",
          warning: "#F59E0B",
          info: "#3B82F6",
          success: "#10B981",
        },
      },
      dark: {
        colors: {
          primary: "#A78BFA",
          secondary: "#818CF8",
          accent: "#C4B5FD",
        },
      },
    },
  },
  defaults: {
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
    },
  },
});
