import antfu from "@antfu/eslint-config";

export default antfu(
  {
    ignores: ["tauri-plugin-better-posthog/permissions/"],

    stylistic: {
      quotes: "double",
      semi: true,
    },

    pnpm: false,
    toml: false,
  },
  {
    files: ["**/*.json", "**/*.jsonc"],
    rules: {
      "jsonc/sort-keys": "off",
    },
  },
);
