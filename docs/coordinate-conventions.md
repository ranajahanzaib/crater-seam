# Coordinate contract

The Rust module `coordinates` owns all coordinate interpretation. React only projects canonical positions supplied by the server.

| Profile       | Permitted raw longitude | East-positive conversion |
| ------------- | ----------------------- | ------------------------ |
| Signed east   | −180 through +180       | raw                      |
| Unsigned east | 0 through 360           | raw                      |
| Signed west   | −180 through +180       | −raw                     |
| Unsigned west | 0 through 360           | −raw                     |

After validating the raw domain, wrap east-positive longitude to `[-180,180)`. Values already inside that canonical interval are preserved exactly to avoid floating-point cancellation near zero. Otherwise use `(east + 180).rem_euclid(360) - 180`. Normalize negative zero to positive zero. Domain checks have no hidden epsilon: a value just outside the declared domain is rejected. Tolerances are used only for geographic comparisons and tests.

Idempotence applies when canonical output is interpreted under the canonical **signed-east** profile. Reapplying an unsigned or west-positive source profile to already-canonical output would change the input contract and is not a valid idempotence test.

Latitude must be finite and within inclusive ±90°. Never clamp or modulo-wrap it. Poles are permitted; differing longitude values there are not considered geographic displacement. Seam equivalents are compared modulo 360 rather than by numeric equality. Converting positive-west 90° to canonical −90° describes conversion of the same stated source position; changing an assumption about east versus west is a hypothetical interpretation difference.

The default profile uses signed east from [Robbins Table 1](https://doi.org/10.1029/2018JE005592), and planetocentric latitude from the derived dataset card. This does not independently establish every transformation in the derived pipeline. `unknown` direction excludes placement with a dataset-level critical finding. `unconfirmed` latitude preserves the values, permits conditional placement, and raises a warning. `planetographic` is rejected because no ellipsoid/body model is supplied.

The NASA image is centered on 0°; west is left, east is right, north is up. The global extent is `[−180,+180] × [+90,−90]`. Projection is `x=(lon+180)/360*width`, `y=(90-lat)/180*height`. The source profile changes how coordinates become canonical; it does not relabel or rotate the canonical display texture. The prime-meridian line, signed edge labels, lunar maria around the image center, and Tycho near 43°S, 11°W were checked in the rendered view. Browser geometry tests check the north/prime/seam mappings and viewport round trips. No pixel-level geodetic registration of the texture is claimed.

See [NASA CGI Moon Kit](https://svs.gsfc.nasa.gov/4720/), [USGS coordinate systems](https://planetarynames.wr.usgs.gov/TargetCoordinates), and [NASA PDS definitions](https://pds.nasa.gov/datastandards/documents/dd/all/current/ch37s22.html).
