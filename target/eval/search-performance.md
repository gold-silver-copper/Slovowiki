# Search cold-load performance (issue #71 harness: tools/search-perf.mjs)

## monolithic search.json (pre-#71 baseline)

Site: `site` · queries: 11 · gzip level 6 (proxy for Pages compression). Byte counts, fetch counts and hits are deterministic; timings are Node-measured medians (informational).

**Cold worst-case query** (“voda”): 1 fetch(es), 44.06 MB raw / 10.80 MB gzipped → ~18.1s @5 Mbps, ~2.27s @40 Mbps (download alone), + 1925 ms parse+score measured.

| query | fetches | raw | gz | ~s @5 Mbps | cold ms | hits | top hits |
|---|--:|--:|--:|--:|--:|--:|---|
| `voda` | 1 | 44.06 MB | 10.80 MB | 18.13 | 1925 | 96 | 7092:voda, 34754:vođa, 29586:okropiti |
| `rěka` | 1 | 44.06 MB | 10.80 MB | 18.13 | 1897 | 94 | 5383:rěka, 5405:rěka//*rěka, 4794:potok |
| `medžu` | 1 | 44.06 MB | 10.80 MB | 18.13 | 1934 | 78 | 3110:među, 26244:iz-srěd, 30748:posrěd |
| `zem` | 1 | 44.06 MB | 10.80 MB | 18.13 | 1937 | 460 | 7812:zem, 1285:glina, 7810:zemja |
| `пластинка` | 1 | 44.06 MB | 10.80 MB | 18.13 | 1922 | 1 | 90980:plastinka |
| `winyl` | 1 | 44.06 MB | 10.80 MB | 18.13 | 1924 | 2 | 178123:vinyl, 178124:vinylovy |
| `water` | 1 | 44.06 MB | 10.80 MB | 18.13 | 1897 | 866 | 19220:klozet, 3548:napustiti, 4640:pojiti |
| `baksheesh` | 1 | 44.06 MB | 10.80 MB | 18.13 | 1886 | 0 |  |
| `s` | 1 | 44.06 MB | 10.80 MB | 18.13 | 1499 | 5001 | 6323:s, 6325:s, 6142:s polȯm |
| `gramplastinka` | 1 | 44.06 MB | 10.80 MB | 18.13 | 1869 | 1 | 122815:gramplastinka |
| `qqzz` | 1 | 44.06 MB | 10.80 MB | 18.13 | 1889 | 0 |  |

Warm session (all 11 queries, shared cache): 1 fetches, 44.06 MB raw / 10.80 MB gzipped, 5726 ms total.

Files fetched per query:
- `voda`: search.json
- `rěka`: search.json
- `medžu`: search.json
- `zem`: search.json
- `пластинка`: search.json
- `winyl`: search.json
- `water`: search.json
- `baksheesh`: search.json
- `s`: search.json
- `gramplastinka`: search.json
- `qqzz`: search.json

## first-letter shards (#71)

Site: `site` · queries: 11 · gzip level 6 (proxy for Pages compression). Byte counts, fetch counts and hits are deterministic; timings are Node-measured medians (informational).

**Cold worst-case query** (“baksheesh”): 3 fetch(es), 5.56 MB raw / 1.39 MB gzipped → ~2.3s @5 Mbps, ~0.29s @40 Mbps (download alone), + 288 ms parse+score measured.

| query | fetches | raw | gz | ~s @5 Mbps | cold ms | hits | top hits |
|---|--:|--:|--:|--:|--:|--:|---|
| `voda` | 2 | 2.02 MB | 528.8 KB | 0.87 | 156 | 62 | 7092:voda, 34754:vođa, 29586:okropiti |
| `rěka` | 2 | 2.50 MB | 661.7 KB | 1.08 | 178 | 33 | 5383:rěka, 5405:rěka//*rěka, 4794:potok |
| `medžu` | 2 | 1.52 MB | 407.6 KB | 0.67 | 110 | 77 | 3110:među, 26244:iz-srěd, 30748:posrěd |
| `zem` | 2 | 921.6 KB | 240.7 KB | 0.39 | 68 | 220 | 7812:zem, 1285:glina, 7810:zemja |
| `пластинка` | 3 | 4.30 MB | 1.05 MB | 1.76 | 199 | 1 | 90980:plastinka |
| `winyl` | 3 | 4.88 MB | 1.21 MB | 2.03 | 238 | 2 | 178123:vinyl, 178124:vinylovy |
| `water` | 2 | 830.3 KB | 221.1 KB | 0.36 | 57 | 317 | 19220:klozet, 3548:napustiti, 4640:pojiti |
| `baksheesh` | 3 | 5.56 MB | 1.39 MB | 2.33 | 288 | 0 |  |
| `s` | 2 | 52.7 KB | 13.9 KB | 0.02 | 2 | 62 | 6323:s, 6325:s, 32590:s-, sȯ- |
| `gramplastinka` | 3 | 5.04 MB | 1.25 MB | 2.10 | 249 | 1 | 122815:gramplastinka |
| `qqzz` | 3 | 4.20 MB | 1.02 MB | 1.72 | 190 | 0 |  |

Warm session (all 11 queries, shared cache): 13 fetches, 15.84 MB raw / 4.09 MB gzipped, 1214 ms total.

Files fetched per query:
- `voda`: search/manifest.json + search/vo.json
- `rěka`: search/manifest.json + search/re.json
- `medžu`: search/manifest.json + search/me.json
- `zem`: search/manifest.json + search/ze.json
- `пластинка`: search/manifest.json + search/u043fu043b.json + search/browse.json
- `winyl`: search/manifest.json + search/wi.json + search/browse.json
- `water`: search/manifest.json + search/wa.json
- `baksheesh`: search/manifest.json + search/ba.json + search/browse.json
- `s`: search/manifest.json + search/s_.json
- `gramplastinka`: search/manifest.json + search/gr.json + search/browse.json
- `qqzz`: search/manifest.json + search/q.json + search/browse.json
