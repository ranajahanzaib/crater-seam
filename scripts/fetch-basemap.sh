#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
mkdir -p web/public
url=https://svs.gsfc.nasa.gov/vis/a000000/a004700/a004720/lroc_color_2k.jpg
curl --fail --location --retry 3 "$url" -o web/public/lroc_color_2k.jpg.download
mv web/public/lroc_color_2k.jpg.download web/public/lroc_color_2k.jpg
shasum -a 256 web/public/lroc_color_2k.jpg
printf 'Credit: NASA SVS / Ernie Wright / LROC. Source: https://svs.gsfc.nasa.gov/4720/\n'
node - <<'NODE'
const fs=require('fs'),crypto=require('crypto');const b=fs.readFileSync('web/public/lroc_color_2k.jpg');fs.writeFileSync('web/public/basemap-provenance.json',JSON.stringify({source_url:'https://svs.gsfc.nasa.gov/vis/a000000/a004700/a004720/lroc_color_2k.jpg',resource:'https://svs.gsfc.nasa.gov/4720/',credit:'NASA SVS / Ernie Wright / NASA Goddard / LROC',sha256:crypto.createHash('sha256').update(b).digest('hex'),width:2048,height:1024,central_meridian:0,orientation:'north up, east right',recorded_at:new Date().toISOString()},null,2)+'\n');
NODE
