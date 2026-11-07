import { useEffect, useRef, useState } from "react";
import { LunarMap } from "./Map";
import { api, clampView, degrees, number } from "./lib";
import type {
  Convention,
  Coordinate,
  Detail,
  MapData,
  Metadata,
  Row,
  Simulation,
  Summary,
  View,
} from "./types";
const names: Record<Convention, string> = {
  signed_east: "Signed · positive east",
  unsigned_east: "Unsigned · positive east",
  signed_west: "Signed · positive west",
  unsigned_west: "Unsigned · positive west",
};
function useData<T>(url: string) {
  const [data, setData] = useState<T>();
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(true);
  useEffect(() => {
    const controller = new AbortController();
    setLoading(true);
    setError("");
    api<T>(url, controller.signal)
      .then(setData)
      .catch((e) => {
        if (e.name !== "AbortError") setError(String(e));
      })
      .finally(() => {
        if (!controller.signal.aborted) setLoading(false);
      });
    return () => controller.abort();
  }, [url]);
  return { data, error, loading };
}
function Histogram({
  values,
  label,
  edges = false,
}: {
  values: number[];
  label: string;
  edges?: boolean;
}) {
  const max = Math.max(1, ...values);
  return (
    <div className="histogram">
      <div className="small-heading">{label}</div>
      <svg
        viewBox="0 0 280 48"
        role="img"
        aria-label={`${label}: ${values.map((n, i) => `bin ${i + 1}: ${n}`).join(", ")}`}
      >
        {values.map((n, i) => (
          <rect
            key={i}
            x={(i * 280) / values.length}
            y={46 - (n / max) * 42}
            width={280 / values.length - 1}
            height={Math.max(1, (n / max) * 42)}
            fill={
              edges && (i === 0 || i === values.length - 1)
                ? "#db8b80"
                : "#6d9298"
            }
          />
        ))}
      </svg>
      <div className="hist-labels">
        <span>{edges ? "below −90°" : "below −180°"}</span>
        <span>0°</span>
        <span>{edges ? "above +90°" : "above +180°"}</span>
      </div>
    </div>
  );
}
export function App() {
  const [source, setSource] = useState<Convention>("signed_east");
  const [importer, setImporter] = useState<Convention>("unsigned_east");
  const [view, setView] = useState<View>({ zoom: 1, x: 0, y: 0 });
  const [search, setSearch] = useState("");
  const [rule, setRule] = useState("");
  const [severity, setSeverity] = useState("");
  const [disposition, setDisposition] = useState("");
  const [onlyAffected, setOnlyAffected] = useState(false);
  const [page, setPage] = useState(0);
  const [selectedRow, setSelectedRow] = useState<number>();
  const [detail, setDetail] = useState<Detail | null>(null);
  const [hover, setHover] = useState("");
  const [metadataOpen, setMetadataOpen] = useState(false);
  const hoverSeq = useRef(0);
  const base = new URLSearchParams({ source, importer });
  const filters = new URLSearchParams({
    ...Object.fromEntries(base),
    search,
    rule,
    severity,
    disposition,
    affected: String(onlyAffected),
  });
  const core = base.toString(),
    query = filters.toString();
  const meta = useData<Metadata>(`/api/dataset?${core}`);
  const stats = useData<{ summary: Summary; simulation: Simulation }>(
    `/api/summary?${core}`,
  );
  const map = useData<MapData>(
    `/api/map?${query}&zoom=${view.zoom}&x=${view.x}&y=${view.y}`,
  );
  const table = useData<{
    total: number;
    page: number;
    limit: number;
    rows: Row[];
  }>(`/api/issues?${query}&page=${page}`);
  const [interactionError, setInteractionError] = useState("");
  useEffect(() => {
    setPage(0);
  }, [source, importer, search, rule, severity, disposition, onlyAffected]);
  useEffect(() => {
    if (selectedRow === undefined) return;
    const controller = new AbortController();
    api<Detail>(`/api/records/row:${selectedRow}?${core}`, controller.signal)
      .then(setDetail)
      .catch((e) => {
        if (e.name !== "AbortError") setInteractionError(String(e));
      });
    return () => controller.abort();
  }, [selectedRow, core]);
  const pick = async (c: Coordinate, side: string, isHover: boolean) => {
    const seq = ++hoverSeq.current;
    try {
      const result = await api<Detail | null>(
        `/api/nearest?${query}&lat=${c.latitude}&lon=${c.longitude}&side=${side}&tolerance=${4 / view.zoom}`,
      );
      if (isHover) {
        if (seq === hoverSeq.current)
          setHover(
            result
              ? `${result.record.raw.id} · ${degrees(result.record.parsed.latitude as number)} / ${degrees(result.record.parsed.longitude as number)}`
              : "No nearby crater",
          );
      } else if (result) {
        setSelectedRow(result.record.raw.source_row);
        setDetail(result);
      }
    } catch (e) {
      if (!isHover) setInteractionError(String(e));
    }
  };
  const summary = stats.data?.summary,
    sim = stats.data?.simulation;
  const error =
    meta.error || stats.error || map.error || table.error || interactionError;
  const loading = stats.loading || map.loading || table.loading;
  return (
    <>
      <header className="topbar">
        <a className="brand" href="/" aria-label="CraterSeam home">
          <span className="brand-mark" aria-hidden="true">
            ⊕
          </span>
          <span>
            CraterSeam<small>PLANETARY COORDINATE INSPECTOR</small>
          </span>
        </a>
        <div className="dataset-badge">
          <span className="status-dot" /> ROBBINS-DERIVED{" "}
          <span className="subtle">/ LUNAR</span>
        </div>
        <nav>
          <a
            href="https://github.com/ranajahanzaib/crater-seam#readme"
            target="_blank"
            rel="noreferrer"
          >
            Documentation ↗
          </a>
          <details className="export-menu">
            <summary>Export audit ↓</summary>
            <div>
              {["json", "flagged", "normalized"].map((format) => (
                <a
                  key={format}
                  href={`/api/report?${core}&format=${format}`}
                  download
                >
                  {format === "json"
                    ? "Audit JSON"
                    : format === "flagged"
                      ? "Flagged records CSV"
                      : "Normalized CSV"}
                </a>
              ))}
            </div>
          </details>
        </nav>
      </header>
      <main>
        <div className="intro">
          <div>
            <div className="eyebrow">
              COORDINATE INTEROPERABILITY / LOCAL WORKSPACE
            </div>
            <h1>Same Moon. Different assumptions.</h1>
            <p>
              Compare a strict importer with a validated source profile. Trace
              every missing point to its evidence.
            </p>
          </div>
          <button
            className="quiet"
            onClick={() => setMetadataOpen(!metadataOpen)}
            aria-expanded={metadataOpen}
          >
            Source & schema {metadataOpen ? "−" : "+"}
          </button>
        </div>
        {error && (
          <div role="alert" className="error">
            Unable to load audit: {error}. Start the Rust API with{" "}
            <code>cargo run -- serve --input data/lunar_craters.parquet</code>.
          </div>
        )}
        {!stats.data && !error && (
          <p role="status">Loading the complete source audit…</p>
        )}
        <section className="metrics" aria-label="Validated audit totals">
          {[
            ["Source records", summary?.total],
            ["Accepted", summary?.accepted],
            ["Normalized", summary?.normalized],
            [
              "Manual review",
              summary ? summary.review + summary.excluded : undefined,
            ],
            ["Excluded", summary?.excluded],
          ].map(([label, value]) => (
            <div
              className={label === "Excluded" ? "metric invalid" : "metric"}
              key={label}
            >
              <span>{label}</span>
              <strong>
                {value === undefined ? "—" : number(value as number)}
              </strong>
            </div>
          ))}
          <div className="metric convention">
            <span>Source convention</span>
            <strong>
              {source === "signed_east" ? "Signed east" : names[source]}
            </strong>
            <small>
              {source === "signed_east"
                ? "Scientific reference profile"
                : "Explicit user assumption"}
            </small>
          </div>
        </section>
        <section
          className="comparison-controls"
          aria-label="Coordinate conventions"
        >
          <label>
            <span>A / Simulated importer</span>
            <select
              value={importer}
              onChange={(e) => setImporter(e.target.value as Convention)}
            >
              {Object.entries(names).map(([key, label]) => (
                <option key={key} value={key}>
                  {label}
                  {key === "unsigned_east" ? " · trust README" : ""}
                </option>
              ))}
            </select>
          </label>
          <span className="comparison-arrow" aria-hidden="true">
            →
          </span>
          <label>
            <span>B / Validated source</span>
            <select
              value={source}
              onChange={(e) => setSource(e.target.value as Convention)}
            >
              {Object.entries(names).map(([key, label]) => (
                <option key={key} value={key}>
                  {label}
                  {key === "signed_east" ? " · Robbins reference" : ""}
                </option>
              ))}
            </select>
          </label>
          <div className="view-controls">
            <span>Linked view / {view.zoom.toFixed(1)}×</span>
            <button
              aria-label="Zoom in both maps"
              onClick={() =>
                setView(clampView({ ...view, zoom: view.zoom * 1.5 }))
              }
            >
              +
            </button>
            <button
              aria-label="Zoom out both maps"
              onClick={() =>
                setView(clampView({ ...view, zoom: view.zoom / 1.5 }))
              }
            >
              −
            </button>
            <button onClick={() => setView({ zoom: 1, x: 0, y: 0 })}>
              Reset view
            </button>
          </div>
        </section>
        <div
          className={`maps ${loading ? "updating" : ""}`}
          aria-busy={loading}
        >
          <LunarMap
            title="Naive interpretation"
            subtitle="Strictly apply the chosen importer contract"
            side="naive"
            bins={map.data?.naive ?? []}
            points={map.data?.naive_points ?? []}
            affected={[]}
            view={view}
            setView={setView}
            selected={detail?.simulation ?? null}
            onPick={pick}
            count={map.data?.naive_count ?? 0}
            hover={hover}
          />
          <LunarMap
            title="Validated coordinates"
            subtitle="Validate first. Convert only permitted values."
            side="validated"
            bins={map.data?.validated ?? []}
            points={map.data?.validated_points ?? []}
            affected={map.data?.affected ?? []}
            view={view}
            setView={setView}
            selected={detail?.record.normalized ?? null}
            onPick={pick}
            count={map.data?.validated_count ?? 0}
            hover={hover}
          />
        </div>
        <div className="map-notes">
          <span>
            <i className="swatch cyan" />
            Full-data density · 2° cells; zoom above 2×: sampled crater markers{" "}
            <i className="swatch amber" />
            Affected under simulation
          </span>
          <span>0° central meridian · ±180° seam · north up · NASA / LROC</span>
        </div>
        <div className="evidence-strip">
          <div>
            <span className="eyebrow amber-text">SIMULATED CONSEQUENCE</span>
            <strong>
              {number(sim?.incorrectly_excluded ?? 0)} valid records rejected
            </strong>
            <p>
              {number(sim?.located_differently ?? 0)} located differently ·{" "}
              {number(sim?.normalized ?? 0)} importer conversions. These counts
              describe the selected assumptions.
            </p>
          </div>
          <div>
            <span className="eyebrow red-text">OBSERVED SOURCE FINDINGS</span>
            <strong>
              {number(summary?.excluded ?? 0)} records cannot be placed
            </strong>
            <p>
              Excluded under the source profile. Invalid latitude is never
              clamped or assigned a fabricated map position.
            </p>
          </div>
          <button
            onClick={() => {
              setOnlyAffected(true);
              setRule("");
              setSeverity("");
              setDisposition("");
              setSearch("");
            }}
          >
            Inspect affected records ↗
          </button>
        </div>
        {metadataOpen && meta.data && (
          <section className="metadata">
            <div>
              <h2>Evidence, not inference</h2>
              <dl>
                <dt>Published dataset card</dt>
                <dd>
                  0–360° east longitude · crater_id int64 · planetocentric
                  latitude
                </dd>
                <dt>Observed file ranges</dt>
                <dd className="mono">
                  Longitude {degrees(summary?.longitude.min)} to{" "}
                  {degrees(summary?.longitude.max)}
                  <br />
                  Latitude {degrees(summary?.latitude.min)} to{" "}
                  {degrees(summary?.latitude.max)}
                </dd>
                <dt>Scientific reference</dt>
                <dd>
                  <a href="https://doi.org/10.1029/2018JE005592">
                    Robbins (2019), Table 1 ↗
                  </a>{" "}
                  · signed positive east
                </dd>
                <dt>Active profile evidence</dt>
                <dd>{meta.data.profile.evidence}</dd>
              </dl>
              {meta.data.schema_findings.map((f, i) => (
                <div className="finding" key={i}>
                  <strong>{f.title}</strong>
                  <p>{f.explanation}</p>
                </div>
              ))}
            </div>
            <div>
              <h3>Physical schema</h3>
              <table>
                <thead>
                  <tr>
                    <th>Column</th>
                    <th>Physical type</th>
                    <th>Logical type</th>
                    <th>Nulls</th>
                  </tr>
                </thead>
                <tbody>
                  {meta.data.provenance.schema.map((s) => (
                    <tr key={s.name}>
                      <td>{s.name}</td>
                      <td>{s.physical_type}</td>
                      <td>{s.logical_type}</td>
                      <td>{number(s.null_count)}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
              <dl>
                <dt>SHA-256</dt>
                <dd className="hash">{meta.data.provenance.sha256}</dd>
                <dt>Source revision</dt>
                <dd className="hash">
                  {meta.data.provenance.source_revision ??
                    "Not recorded; run craterseam fetch"}
                </dd>
                <dt>Acquired / engine</dt>
                <dd>
                  {meta.data.provenance.acquired_at ?? "Not recorded"} /{" "}
                  {meta.data.engine_version}
                </dd>
              </dl>
            </div>
          </section>
        )}
        <section className="explorer">
          <div className="section-heading">
            <div>
              <span className="eyebrow">RECORD-LEVEL EVIDENCE</span>
              <h2>
                Audit explorer <span>{number(table.data?.total ?? 0)}</span>
              </h2>
            </div>
            <button
              className="quiet"
              onClick={() => {
                setSearch("01-1-00003");
                setRule("");
                setSeverity("");
                setDisposition("");
                setOnlyAffected(false);
              }}
            >
              Walkthrough: first crater ↗
            </button>
          </div>
          <div className="filters">
            <label>
              Record ID
              <input
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                placeholder="Search crater ID…"
                type="search"
              />
            </label>
            <label>
              Issue
              <select
                aria-label="Issue"
                value={rule}
                onChange={(e) => setRule(e.target.value)}
              >
                <option value="">All rules</option>
                {Object.keys(summary?.rule_counts ?? {}).map((r) => (
                  <option key={r}>{r}</option>
                ))}
              </select>
            </label>
            <label>
              Severity
              <select
                aria-label="Severity"
                value={severity}
                onChange={(e) => setSeverity(e.target.value)}
              >
                <option value="">All severities</option>
                {["critical", "warning", "informational"].map((v) => (
                  <option key={v}>{v}</option>
                ))}
              </select>
            </label>
            <label>
              Disposition
              <select
                aria-label="Disposition"
                value={disposition}
                onChange={(e) => setDisposition(e.target.value)}
              >
                <option value="">All dispositions</option>
                {["accepted", "normalized", "review", "excluded"].map((v) => (
                  <option key={v}>{v}</option>
                ))}
              </select>
            </label>
            <label className="checkbox">
              <input
                type="checkbox"
                checked={onlyAffected}
                onChange={(e) => setOnlyAffected(e.target.checked)}
              />
              Simulation affected only
            </label>
          </div>
          <div className="explorer-grid">
            <div className="table-area">
              <div className="table-scroll">
                <table>
                  <thead>
                    <tr>
                      <th>Record ID</th>
                      <th>Raw lat / lon</th>
                      <th>Canonical lat / lon</th>
                      <th>Finding / severity</th>
                      <th>Disposition</th>
                    </tr>
                  </thead>
                  <tbody>
                    {table.data?.rows.map((r) => (
                      <tr
                        key={r.source_row}
                        className={
                          selectedRow === r.source_row ? "selected" : ""
                        }
                      >
                        <td>
                          <button
                            className="record-link"
                            onClick={() => setSelectedRow(r.source_row)}
                          >
                            {r.id}
                          </button>
                          <small>row {r.source_row}</small>
                        </td>
                        <td className="mono">
                          {degrees(r.parsed.latitude)}
                          <br />
                          {degrees(r.parsed.longitude)}
                        </td>
                        <td className="mono">
                          {degrees(r.normalized?.latitude)}
                          <br />
                          {degrees(r.normalized?.longitude)}
                        </td>
                        <td>
                          {r.issues.length ? (
                            r.issues.map((i, j) => (
                              <span
                                className={`issue-label ${i.severity}`}
                                key={j}
                              >
                                {i.rule_id.toLowerCase().replaceAll("_", " ")}
                                <small>{i.severity}</small>
                              </span>
                            ))
                          ) : (
                            <span className="subtle">
                              {r.affected
                                ? "Simulated interpretation difference"
                                : "No coordinate findings"}
                            </span>
                          )}
                        </td>
                        <td>
                          <span className={`disposition ${r.disposition}`}>
                            {r.disposition}
                          </span>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
                {table.data?.total === 0 && (
                  <p className="empty">
                    No records match these filters. Dataset-level metadata
                    findings appear in Source & schema.
                  </p>
                )}
              </div>
              <div className="pagination">
                <span>
                  {number(table.data?.total ?? 0)} matching · page {page + 1} of{" "}
                  {Math.max(1, Math.ceil((table.data?.total ?? 0) / 25))}
                </span>
                <div>
                  <button
                    disabled={page === 0}
                    onClick={() => setPage(page - 1)}
                  >
                    Previous
                  </button>
                  <button
                    disabled={
                      !table.data || (page + 1) * 25 >= table.data.total
                    }
                    onClick={() => setPage(page + 1)}
                  >
                    Next
                  </button>
                </div>
              </div>
            </div>
            <aside
              className="inspector"
              aria-label="Selected record"
              aria-live="polite"
            >
              {detail ? (
                <>
                  <div className="eyebrow">
                    SELECTED RECORD / ROW {detail.record.raw.source_row}
                  </div>
                  <h3>{detail.record.raw.id}</h3>
                  <span className={`disposition ${detail.record.disposition}`}>
                    {detail.record.disposition}
                  </span>
                  <h4>Why did this point change?</h4>
                  <p>{detail.why}</p>
                  <dl>
                    <dt>Source latitude / longitude</dt>
                    <dd>
                      {degrees(detail.record.parsed.latitude as number)} /{" "}
                      {degrees(detail.record.parsed.longitude as number)}
                    </dd>
                    <dt>Canonical latitude / longitude</dt>
                    <dd>
                      {degrees(detail.record.normalized?.latitude)} /{" "}
                      {degrees(detail.record.normalized?.longitude)}
                    </dd>
                  </dl>
                  {!detail.simulation && (
                    <p className="amber-text">
                      Absent from naive map: coordinate violates this importer’s
                      contract.
                    </p>
                  )}
                  {!detail.record.normalized && (
                    <p className="red-text">
                      No validated map position. The source values fail
                      validation; see findings.
                    </p>
                  )}
                  {detail.calculation && (
                    <details>
                      <summary>Normalization calculation</summary>
                      <code className="calculation">{detail.calculation}</code>
                      <p>
                        Applied only after validating the declared source
                        domain. Latitude is preserved.
                      </p>
                    </details>
                  )}
                  {detail.record.issues.map((f, i) => (
                    <div className="finding" key={i}>
                      <strong>{f.title}</strong>
                      <small>
                        {f.severity} · observed under source profile
                      </small>
                      <p>{f.explanation}</p>
                      <p>{f.suggested_action}</p>
                    </div>
                  ))}
                  <details>
                    <summary>Original source record</summary>
                    <pre>
                      {JSON.stringify(detail.record.raw.fields, null, 2)}
                    </pre>
                  </details>
                  <details>
                    <summary>Parsed fields & source profile</summary>
                    <pre>
                      {JSON.stringify(
                        {
                          parsed: detail.record.parsed,
                          profile: detail.profile,
                        },
                        null,
                        2,
                      )}
                    </pre>
                  </details>
                </>
              ) : (
                <div className="inspector-empty">
                  <span className="crosshair" aria-hidden="true">
                    ⊕
                  </span>
                  <h3>Inspect a crater</h3>
                  <p>
                    Select a record or click either map to trace its raw
                    coordinates, validation, and interpretation.
                  </p>
                  <p>
                    Invalid coordinates remain in the table. They never receive
                    invented map locations.
                  </p>
                </div>
              )}
            </aside>
          </div>
        </section>
        {summary && (
          <section className="distributions" aria-label="Source distributions">
            <Histogram
              values={summary.longitude_histogram}
              label="SOURCE LONGITUDE / 10° BINS"
            />
            <Histogram
              values={summary.latitude_histogram}
              label="SOURCE LATITUDE / 5° BINS"
              edges
            />
            <div>
              <div className="small-heading">
                ISSUE BREAKDOWN / ALL SOURCE ROWS
              </div>
              {Object.entries(summary.rule_counts).map(([k, v]) => (
                <div className="rule-count" key={k}>
                  <span>{k.toLowerCase().replaceAll("_", " ")}</span>
                  <strong>{number(v)}</strong>
                </div>
              ))}
              <small>
                Multiple findings may belong to one row. Excluded rows also
                require manual review.
              </small>
            </div>
          </section>
        )}
        <footer>
          <span>
            CraterSeam / audit engine {meta.data?.engine_version ?? "0.1.0"} ·
            local, reproducible, source-preserving
          </span>
          <span>
            <a href="https://huggingface.co/datasets/juliensimon/lunar-craters-robbins">
              Dataset: Julien Simon / Robbins · CC BY 4.0
            </a>{" "}
            ·{" "}
            <a href="https://svs.gsfc.nasa.gov/4720/">
              Texture: NASA SVS / LROC
            </a>
          </span>
        </footer>
      </main>
    </>
  );
}
