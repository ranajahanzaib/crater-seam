import type { Coordinate, View } from "./types";
export const number = (v: number) => new Intl.NumberFormat("en-US").format(v);
export const degrees = (v: number | null | undefined) =>
  v == null ? "—" : `${v.toFixed(3)}°`;
export function clampView(v: View): View {
  const zoom = Math.min(12, Math.max(1, v.zoom));
  return {
    zoom,
    x: Math.max(0, Math.min(1 - 1 / zoom, v.x)),
    y: Math.max(0, Math.min(1 - 1 / zoom, v.y)),
  };
}
export function project(c: Coordinate, width: number, height: number, v: View) {
  return {
    x: ((c.longitude + 180) / 360 - v.x) * width * v.zoom,
    y: ((90 - c.latitude) / 180 - v.y) * height * v.zoom,
  };
}
export function unproject(x: number, y: number, v: View): Coordinate {
  return {
    longitude: (v.x + x / v.zoom) * 360 - 180,
    latitude: 90 - (v.y + y / v.zoom) * 180,
  };
}
export async function api<T>(path: string, signal?: AbortSignal): Promise<T> {
  const response = await fetch(path, { signal });
  if (!response.ok) throw new Error((await response.text()).slice(0, 300));
  return response.json() as Promise<T>;
}
