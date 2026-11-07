import "@testing-library/jest-dom/vitest";
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";
vi.mock("./Map", () => ({
  LunarMap: ({ title }: { title: string }) => <section>{title}</section>,
}));
const summary = {
  total: 2,
  accepted: 1,
  normalized: 0,
  review: 0,
  excluded: 1,
  valid_coordinates: 1,
  affected_records: 1,
  duplicate_records: 0,
  rule_counts: { LATITUDE_OUT_OF_RANGE: 1 },
  latitude: { min: -105.383, max: 79.257 },
  longitude: { min: -148.095, max: 88.993 },
  latitude_histogram: [1, 1],
  longitude_histogram: [1, 1],
};
describe("inspector interactions", () => {
  beforeEach(() => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async (url: string) => {
        let data: unknown = {};
        if (url.includes("/api/summary"))
          data = {
            summary,
            simulation: {
              included: 0,
              excluded: 2,
              normalized: 0,
              located_differently: 0,
              incorrectly_excluded: 1,
              affected: 1,
            },
          };
        else if (url.includes("/api/dataset"))
          data = {
            provenance: { schema: [], sha256: "abc" },
            profile: { evidence: "explicit" },
            schema_findings: [],
            engine_version: "test",
          };
        else if (url.includes("/api/map"))
          data = {
            naive: [],
            validated: [],
            affected: [],
            naive_points: [],
            validated_points: [],
          };
        else if (url.includes("/api/issues"))
          data = {
            total: 1,
            rows: [
              {
                id: "test-crater",
                source_row: 1,
                parsed: { latitude: 79.257, longitude: -148.095 },
                normalized: { latitude: 79.257, longitude: -148.095 },
                issues: [],
                disposition: "accepted",
                affected: true,
              },
            ],
          };
        else if (url.includes("/api/records"))
          data = {
            record: {
              raw: { id: "test-crater", source_row: 1, fields: {} },
              parsed: { latitude: 79.257, longitude: -148.095 },
              normalized: { latitude: 79.257, longitude: -148.095 },
              issues: [],
              disposition: "accepted",
            },
            simulation: null,
            why: "The point has not moved.",
            profile: {},
            calculation: "canonical",
          };
        return { ok: true, json: async () => data };
      }),
    );
  });
  afterEach(() => {
    cleanup();
    vi.unstubAllGlobals();
  });
  it("selects records and sends explicit filter and convention choices", async () => {
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "test-crater" }));
    await expect(
      screen.findByText("The point has not moved."),
    ).resolves.toBeVisible();
    fireEvent.change(screen.getByLabelText("Issue", { exact: true }), {
      target: { value: "LATITUDE_OUT_OF_RANGE" },
    });
    fireEvent.change(screen.getByLabelText("A / Simulated importer"), {
      target: { value: "signed_west" },
    });
    await waitFor(() =>
      expect(
        vi
          .mocked(fetch)
          .mock.calls.some(
            ([url]) =>
              String(url).includes("importer=signed_west") &&
              String(url).includes("rule=LATITUDE_OUT_OF_RANGE"),
          ),
      ).toBe(true),
    );
  });
  it("opens metadata and affected-only controls", async () => {
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: /Source & schema/ }));
    await screen.findByRole("heading", { name: "Evidence, not inference" });
    fireEvent.click(
      screen.getByRole("button", { name: /Inspect affected records/ }),
    );
    expect(screen.getByLabelText("Simulation affected only")).toBeChecked();
  });
});
