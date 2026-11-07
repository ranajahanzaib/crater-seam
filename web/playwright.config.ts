import { defineConfig } from "@playwright/test";
export default defineConfig({
  testDir: "./e2e",
  use: {
    baseURL: process.env.CRATERSEAM_URL ?? "http://127.0.0.1:8787",
    headless: true,
  },
  workers: 1,
  timeout: 30000,
});
