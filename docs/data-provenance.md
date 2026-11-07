# Acquisition and observed evidence

The Rust fetch command acquired the full public Parquet file on November 2, 2026, at 12:24:00 UTC. Acquisition pins the Hugging Face Git revision, records a SHA-256 and saves the dataset card alongside the source. Subsequent audits only attach the sidecar's source/revision metadata when its hash matches the loaded file.

- Dataset: Julien Simon, Lunar Crater Database (Robbins 2019), CC BY 4.0.
- Dataset page: https://huggingface.co/datasets/juliensimon/lunar-craters-robbins
- Revision: `a6d08bfde1273a5e92fa34dbd584eafb6d667e8b`
- File: https://huggingface.co/datasets/juliensimon/lunar-craters-robbins/resolve/a6d08bfde1273a5e92fa34dbd584eafb6d667e8b/data/lunar_craters.parquet
- SHA-256: `0b7365cdd8547915c2d79b0261e14b6e66efaa3d6ec12be25aabb352d40acd74`
- Original input size: 4,322,672 bytes; 384,278 rows.

The observed physical schema contains exactly six columns. The card also mentions `floor_elevation_km` and `depth_rim_sd_km`, which are absent; these optional measurements are not required coordinate columns.

| Field         | Physical / logical type |
| ------------- | ----------------------- |
| crater_id     | BYTE_ARRAY / String     |
| latitude_deg  | DOUBLE / none           |
| longitude_deg | DOUBLE / none           |
| diameter_km   | DOUBLE / none           |
| depth_km      | DOUBLE / none           |
| size_class    | BYTE_ARRAY / String     |

The ingester records null counts for every column. Coordinate null/malformed/nonfinite counts are all zero; optional depth has missing measurements without producing coordinate findings. See [the generated summary](observed-audit.json) for exact column null counts and numeric coordinate ranges.

Measured latitude range: −105.383° to 85.576°. Measured longitude range: −179.997° to 179.998°. Exactly one invalid latitude exists: `12-1-09489`, source row 284,838, latitude −105.383°, longitude 88.993°. No source IDs are duplicated. Under the default scientific profile, 384,277 records are accepted unchanged; none require longitude conversion.

The card says `crater_id` is int64 and longitude is 0–360° east. Both conflict with the file evidence. Robbins (2019), Table 1, explicitly describes signed positive-east longitude. The card explicitly calls latitude planetocentric, which the active profile records as a source metadata claim. We did not independently trace the derived publishing pipeline or compare all coordinates with the original catalog. The impossible latitude must not be attributed to Robbins's original data without that additional work.

The strict README-based simulation includes 210,340 rows and excludes 173,938. Of the excluded rows, 173,937 are valid under the scientific signed-east profile; one has the independently observed invalid latitude. Thus **simulated omissions are not additional source defects**. Crater `01-1-00003` exists, latitude 79.257°, longitude −148.095°; the signed-east canonical numbers are identical to source, yet the unsigned importer rejects it.

## Basemap

Local file: `web/public/lroc_color_2k.jpg`, downloaded from https://svs.gsfc.nasa.gov/vis/a000000/a004700/a004720/lroc_color_2k.jpg, 2048×1024. Credit NASA SVS / Ernie Wright / NASA Goddard / LROC. The NASA resource describes the 0°-centered color map, adapted from the LROC Hapke-normalized WAC mosaic. The application displays it north-up on a signed-east equirectangular extent and applies grayscale/opacity plus overlays. See `web/public/basemap-provenance.json` for the acquired image checksum.

Use [NASA media guidance](https://www.nasa.gov/nasa-brand-center/images-and-media/) for imagery reuse and avoid implying NASA endorsement. The code's MIT license does not relicense the dataset or image.
