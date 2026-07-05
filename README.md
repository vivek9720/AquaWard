# AquaWard

AquaWard is a Rust library for decoding offline municipal water-safety
incident bundles. The crate models packets collected by treatment plants,
hydrant pressure monitors, laboratory samplers, and field crews during a
network outage. A decoded bundle can contain a binary envelope, dictionary
records, topology maps, plume raster tiles, sensor calibration frames,
stateful work-order ledgers, operator journals, and small response scripts.

The repository is intentionally self-contained: it has no third-party Rust
dependencies, includes local cargo-fuzz-compatible harness glue, ships seed
corpora, and has a ClusterFuzzLite build script that builds every harness
with `--offline`.
