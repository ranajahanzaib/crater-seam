import { describe, expect, it } from "vitest";
import { clampView, project, unproject } from "./lib";
describe("map geometry", () => {
  it("maps prime meridian and north correctly", () => {
    expect(
      project({ longitude: 0, latitude: 90 }, 1000, 500, {
        zoom: 1,
        x: 0,
        y: 0,
      }),
    ).toEqual({ x: 500, y: 0 });
    expect(
      project({ longitude: -180, latitude: -90 }, 1000, 500, {
        zoom: 1,
        x: 0,
        y: 0,
      }),
    ).toEqual({ x: 0, y: 500 });
  });
  it("round trips linked viewport", () => {
    const v = { zoom: 4, x: 0.2, y: 0.3 };
    const c = { longitude: 20, latitude: 10 };
    const p = project(c, 1000, 500, v);
    const back = unproject(p.x / 1000, p.y / 500, v);
    expect(back.latitude).toBeCloseTo(c.latitude);
    expect(back.longitude).toBeCloseTo(c.longitude);
  });
  it("bounds zoom and pan", () => {
    expect(clampView({ zoom: 20, x: 10, y: -1 })).toEqual({
      zoom: 12,
      x: 1 - 1 / 12,
      y: 0,
    });
    expect(clampView({ zoom: 0.2, x: 1, y: 1 })).toEqual({
      zoom: 1,
      x: 0,
      y: 0,
    });
  });
});
