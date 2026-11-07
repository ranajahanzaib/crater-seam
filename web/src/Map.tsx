import { useEffect, useRef, useState } from "react";
import { clampView, project, unproject } from "./lib";
import type { Bin, Coordinate, Point, View } from "./types";
type Props = {
  title: string;
  subtitle: string;
  side: "naive" | "validated";
  bins: Bin[];
  points: Point[];
  affected: Bin[];
  view: View;
  setView: (v: View) => void;
  selected: Coordinate | null;
  onPick: (c: Coordinate, side: string, hover: boolean) => void;
  count: number;
  hover: string;
};
export function LunarMap({
  title,
  subtitle,
  side,
  bins,
  points,
  affected,
  view,
  setView,
  selected,
  onPick,
  count,
  hover,
}: Props) {
  const canvas = useRef<HTMLCanvasElement>(null);
  const image = useRef<HTMLImageElement | null>(null);
  const [ready, setReady] = useState(false);
  const [failed, setFailed] = useState(false);
  const [size, setSize] = useState({ width: 600, height: 300 });
  const drag = useRef<{
    x: number;
    y: number;
    view: View;
    moved: boolean;
  } | null>(null);
  const lastHover = useRef(0);
  useEffect(() => {
    const img = new Image();
    img.onload = () => {
      image.current = img;
      setReady(true);
    };
    img.onerror = () => setFailed(true);
    img.src = "/lroc_color_2k.jpg";
    const observer = new ResizeObserver((entries) => {
      const r = entries[0].contentRect;
      setSize({ width: r.width, height: r.height });
    });
    if (canvas.current) observer.observe(canvas.current);
    return () => observer.disconnect();
  }, []);
  useEffect(() => {
    const c = canvas.current;
    if (!c) return;
    const dpr = window.devicePixelRatio || 1;
    c.width = Math.round(size.width * dpr);
    c.height = Math.round(size.height * dpr);
    const ctx = c.getContext("2d");
    if (!ctx) return;
    ctx.scale(dpr, dpr);
    const { width: w, height: h } = size;
    ctx.fillStyle = "#131c21";
    ctx.fillRect(0, 0, w, h);
    ctx.save();
    ctx.translate(-view.x * w * view.zoom, -view.y * h * view.zoom);
    ctx.scale(view.zoom, view.zoom);
    if (ready && image.current) {
      ctx.globalAlpha = 0.53;
      ctx.filter = "grayscale(1)";
      ctx.drawImage(image.current, 0, 0, w, h);
      ctx.filter = "none";
      ctx.globalAlpha = 1;
    }
    const draw = (data: Bin[], color: string) => {
      for (const [x, y, n] of data) {
        ctx.globalAlpha = Math.min(0.88, 0.13 + Math.log2(n + 1) * 0.1);
        ctx.fillStyle = color;
        ctx.fillRect(
          (x * w) / 180,
          (y * h) / 90,
          Math.max(0.6, w / 180),
          Math.max(0.6, h / 90),
        );
      }
      ctx.globalAlpha = 1;
    };
    draw(bins, side === "naive" ? "#bdc6c3" : "#75bdbd");
    if (side === "validated") draw(affected, "#e5b467");
    ctx.restore();
    for (const [longitude, latitude, , isAffected] of points) {
      const p = project({ longitude, latitude }, w, h, view);
      ctx.beginPath();
      ctx.arc(p.x, p.y, 1.8, 0, 2 * Math.PI);
      ctx.fillStyle = isAffected ? "#edbc79" : "#b7e6e7";
      ctx.globalAlpha = 0.85;
      ctx.fill();
    }
    ctx.globalAlpha = 1;
  }, [bins, points, affected, view, size, ready, side]);
  const position = (e: React.PointerEvent<HTMLCanvasElement>) => {
    const r = e.currentTarget.getBoundingClientRect();
    return {
      x: (e.clientX - r.left) / r.width,
      y: (e.clientY - r.top) / r.height,
    };
  };
  const marker = selected ? project(selected, 1000, 500, view) : null;
  return (
    <section className="map-panel" aria-label={title}>
      <div className="map-heading">
        <div>
          <span className={`map-tag ${side}`}>
            {side === "naive" ? "A / SIMULATION" : "B / SOURCE PROFILE"}
          </span>
          <h2>{title}</h2>
          <p>{subtitle}</p>
        </div>
        <div className="map-total">
          {count.toLocaleString()}
          <small>placeable records</small>
        </div>
      </div>
      <div className="map-frame">
        <canvas
          ref={canvas}
          role="img"
          aria-label={`${title}. Density map. Use the audit table to select records with the keyboard.`}
          onPointerDown={(e) => {
            e.currentTarget.setPointerCapture(e.pointerId);
            drag.current = { ...position(e), view, moved: false };
          }}
          onPointerMove={(e) => {
            const pos = position(e);
            const d = drag.current;
            if (d) {
              if (Math.abs(pos.x - d.x) + Math.abs(pos.y - d.y) > 0.005)
                d.moved = true;
              if (d.moved)
                setView(
                  clampView({
                    ...d.view,
                    x: d.view.x - (pos.x - d.x) / d.view.zoom,
                    y: d.view.y - (pos.y - d.y) / d.view.zoom,
                  }),
                );
            } else if (Date.now() - lastHover.current > 250) {
              lastHover.current = Date.now();
              onPick(unproject(pos.x, pos.y, view), side, true);
            }
          }}
          onPointerUp={(e) => {
            if (drag.current && !drag.current.moved) {
              const pos = position(e);
              onPick(unproject(pos.x, pos.y, view), side, false);
            }
            drag.current = null;
          }}
          onPointerCancel={() => {
            drag.current = null;
          }}
          onWheel={(e) => {
            const rect = e.currentTarget.getBoundingClientRect();
            const x = (e.clientX - rect.left) / rect.width,
              y = (e.clientY - rect.top) / rect.height;
            const zoom = Math.max(
              1,
              Math.min(12, view.zoom * (e.deltaY < 0 ? 1.15 : 1 / 1.15)),
            );
            setView(
              clampView({
                zoom,
                x: view.x + x / view.zoom - x / zoom,
                y: view.y + y / view.zoom - y / zoom,
              }),
            );
          }}
        />
        <svg viewBox="0 0 1000 500" aria-hidden="true" className="map-overlay">
          <defs>
            <clipPath id={`clip-${side}`}>
              <rect width="1000" height="500" />
            </clipPath>
          </defs>
          <g clipPath={`url(#clip-${side})`}>
            {[-180, -120, -60, 0, 60, 120, 180].map((lon) => {
              const x = project(
                { longitude: lon, latitude: 0 },
                1000,
                500,
                view,
              ).x;
              return (
                <g key={lon}>
                  <line
                    x1={x}
                    x2={x}
                    y1="0"
                    y2="500"
                    className={
                      lon === 0 ? "prime" : Math.abs(lon) === 180 ? "seam" : ""
                    }
                  />
                  <text
                    style={{ fontSize: 10000 / size.width }}
                    textAnchor={x > 900 ? "end" : "start"}
                    x={Math.max(24, Math.min(970, x + 6))}
                    y="487"
                  >
                    {lon}°
                  </text>
                </g>
              );
            })}
            {[-90, -60, -30, 0, 30, 60, 90].map((lat) => {
              const y = project(
                { longitude: 0, latitude: lat },
                1000,
                500,
                view,
              ).y;
              return (
                <g key={lat}>
                  <line x1="0" x2="1000" y1={y} y2={y} />
                  <text
                    style={{ fontSize: 10000 / size.width }}
                    x="8"
                    y={Math.max(14000 / size.width, Math.min(467, y - 7))}
                  >
                    {lat}°
                  </text>
                </g>
              );
            })}
            {marker && (
              <g
                className="target"
                transform={`translate(${marker.x},${marker.y})`}
              >
                <circle r="12" />
                <circle r="3" />
                <path d="M-20 0h12M8 0h12M0-20v12M0 8v12" />
              </g>
            )}
          </g>
        </svg>
        {failed && (
          <span className="texture-fallback">
            Texture unavailable · coordinate graticule active
          </span>
        )}
      </div>
      <div className="map-footer">
        <span>
          {side === "naive"
            ? "○ Strict domain rejection"
            : "○ Canonical positive east"}
        </span>
        <span>
          {hover || "Drag to pan · scroll to zoom · click to inspect"}
        </span>
      </div>
    </section>
  );
}
