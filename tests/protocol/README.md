# Protocol compatibility fixtures

Real blocks used by `tests/protocol_compat.rs`, as lowercase hex of the block with its
era wrapper. Each is named for its era and the major protocol version in its header.

| file | header version | source |
|---|---|---|
| `babbage-v7.block` | 7 | Pallas `test_data/babbage1.block` |
| `babbage-v8.block` | 8 | Pallas `test_data/babbage7.block` |
| `conway-v9.block` | 9 | Pallas `test_data/conway1.block` |
| `conway-v10.block` | 10 | Pallas `test_data/conway4.block` |
| `conway-v11.block` | 11 | Mainnet slot 202315107, block `d03a37835e9f1ffb70e8a476334c69183ab5491c911faa7ea0d6ba285dd7c8b4`, fetched from `backbone.cardano.iog.io:3001` with block-fetch |
| `conway-v12.block` | 12 | Pallas `test_data/conway5.block`, Musashi testnet, last block before the fork |
| `dijkstra-v12.block` | 12 | Pallas `test_data/dijkstra3.block`, Musashi testnet |

Pallas documents the Musashi blocks in `test_data/dijkstra-fixtures.md`.

The Dijkstra block needs a Pallas build that can read the era, so its tests only compile
with `--cfg pallas_next`. The `Protocol compatibility (next Pallas)` CI job runs them.
