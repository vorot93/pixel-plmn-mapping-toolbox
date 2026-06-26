# pixel-plmn-mapping-toolbox

Self-contained Rust CLI for inspecting and editing Google Pixel's
`ap_plmn_mapping.binarypb` — the firmware file that maps mobile networks
(PLMNs) to carrier profiles. It decodes to editable TOML and re-encodes
**byte-for-byte**, with no `protoc`.

## What this edits

`ap_plmn_mapping.binarypb` is a serialized **proto2** message Pixel firmware
uses as a lookup table: given the network a SIM is on (its PLMN), it resolves a
`carrier_id` / `identifier` and applies carrier-specific behaviour. Schema
(`definition.proto`):

```proto
syntax = "proto2";
message CarrierMap  { repeated CarrierEntry entry = 1; }
message CarrierEntry {
  repeated int32  plmns      = 1;   // one or more networks
  optional int32  carrier_id = 2;   // internal index
  optional string identifier = 3;   // human label, e.g. "VZW", "EU_COMMON"
}
```

The stock file holds **80 entries**. `carrier_id` is an incremental index
(`1..84`, with gaps at 31/32/48/49 left by carriers removed over revisions);
every entry has a unique `identifier`, which is the real key you edit against.

## PLMN encoding (3GPP TS 24.008, TBCD)

A PLMN is `MCC` (3 digits) + `MNC` (2 or 3 digits) packed into a 24-bit int.
As six hex nibbles (big-endian) the layout is:

```
nibble:  0    1    2    3    4    5
value:  M2   M1   N3   M3   N2   N1     (N3 = F marks a 2-digit MNC)
```

This tool shows each PLMN as **`MCC-MNC`, where every character is the raw hex
nibble** — so it reads naturally for ordinary networks and is a faithful,
lossless representation for the filler/wildcard nibble `F`:

| value (int) | string | meaning |
|---|---|---|
| 197154 | `302-220` | TELUS (3-digit MNC) |
| 5435408 | `250-01` | an MCC-250 (Russia) network, 2-digit MNC |
| 2291967 | `228-ff` | **wildcard**: any MNC under MCC 228 |
| 10090905 | `999-99` | the test/placeholder PLMN (unused slots) |

Parsing is case-insensitive. Of the 443 PLMNs in the stock file, 42 are
`MCC-ff` wildcards used by the regional `*_COMMON` / `EU` / `IN_GEN` entries.

## Build

```bash
cargo build --release
# binary: target/release/pixel-plmn-mapping-toolbox
```

Requires a Rust toolchain new enough for **edition 2024** (Rust ≥ 1.85).
Protobuf code is generated at build time by `protobuf-codegen` in pure-Rust
mode — no `protoc` binary needed.

## Commands

Every command reads `--in`/`-i` (default stdin) and writes `--out`/`-o`
(default stdout), mirroring a `protoc … < in > out` workflow.

| Command | In → Out | Does |
|---|---|---|
| `decode` | binarypb → TOML | dump the map as editable TOML |
| `encode` | TOML → binarypb | rebuild the binary; validates ids/names unique |
| `inject-plmn <plmn> <mapping>` | binarypb → binarypb | append one PLMN to an existing mapping (by `name`); idempotent; errors if the name is absent |

## TOML format

```toml
[[mapping]]
id = 8
name = "TELUS"
plmns = ["302-220", "302-221"]

[[mapping]]
id = 85
name = "RU_COMMON"
plmns = ["250-01", "250-02", "250-99"]
```

- `id` ↔ `carrier_id`, `name` ↔ `identifier`, `plmns` ↔ the decoded networks.
- `id` and `name` are **required and must be unique**. `encode` takes `id`
  verbatim — it never renumbers — so existing entries keep their exact id, and
  you pick a free one for a new mapping (`85`, since `84` is the current max).
- `decode` errors if any entry is missing `carrier_id` or `identifier` (the
  stock file always has both).

## Examples

Add Russian carriers (MCC 250) as a new mapping:

```bash
pixel-plmn-mapping-toolbox decode < ap_plmn_mapping.binarypb > mapping.toml
# add an [[mapping]] block: id = 85, name = "RU_COMMON", plmns = ["250-01", ...]
pixel-plmn-mapping-toolbox encode < mapping.toml > ap_plmn_mapping_fix.binarypb
```

One-off: drop a single PLMN into an existing mapping without a full round-trip:

```bash
pixel-plmn-mapping-toolbox inject-plmn 250-01 EU_COMMON \
  --in ap_plmn_mapping.binarypb --out ap_plmn_mapping_fix.binarypb
```

## Guarantees

- **Byte-identical round-trips.** `decode` then `encode` with no edits
  reproduces the input *byte-for-byte*. This holds because the encoder matches
  the stock wire format exactly: `plmns` is non-packed (each value is its own
  field), fields are emitted in field-number order, there are no unknown fields,
  and the codec is a lossless bijection over all 24-bit values. Only your actual
  edits change bytes.
- **Order and duplicates are preserved verbatim — never sorted or deduped.**
  The stock `plmns` lists are unordered (they grow by append over firmware
  revisions — e.g. `IN_GEN` is several sorted runs concatenated) and can even
  repeat a value (`APAC_COMMON` lists `450-05` twice). The tool keeps them as-is
  in both directions; you may list them in any order when editing.

## Notes

- `decode`/`encode` is the bulk path (full TOML round-trip); `inject-plmn`
  operates on the binary directly and so preserves untouched entries exactly.
- The generated protobuf types depend on the `protobuf` 3.x output shape, and
  `Cargo.lock` is gitignored — pin `protobuf` / `protobuf-codegen` to an exact
  version if you need the build reproducible across dependency updates.
