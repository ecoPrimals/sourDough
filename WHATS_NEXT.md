# What's Next for sourDough

**Version**: 0.3.0
**Date**: May 14, 2026

---

## Shipped (v0.3.0 — May 14, 2026)

- [x] `sourdough sign` — Ed25519 detached signatures (`.sig` sidecar)
- [x] `sourdough verify` — signature verification against public key
- [x] `sourdough validate ecobin <binary>` — static linking, stripped, size budget checks
- [x] `sourdough scaffold systemd` — hardened `.service` unit generation
- [x] `sourdough layout` — triple-first binary layout validation
- [x] `sourdough validate composition` — composition binary presence (tower/node/nest/nucleus/meta/full + niche)
- [x] `sourdough genomebin sign` — wired to real Ed25519 signing
- [x] Scaffold README includes deployment section
- [x] Niche compositions from `ports.env` (hotspring, neuralspring, wetspring, groundspring, healthspring)

## Shipped (v0.2.0 — May 11, 2026)

- [x] Scaffold generates `{name}-server` crate (JSON-RPC 2.0 + capability wire standard)
- [x] Scaffold generates `.github/workflows/ci.yml` + `notify-plasmidbin.yml` + `release.yml`
- [x] Scaffold generates `deny.toml` (ecoBin v3.0 + explicit `ring` ban)
- [x] Scaffold generates `method_gate.rs` (JH-0/JH-2 pre-dispatch capability gate)
- [x] Scaffold generates `btsp.negotiate` handler (NULL cipher fallback)
- [x] `PeekedStream` transport in sourdough-core (ecosystem convergence)
- [x] Socket path resolution (`$XDG_RUNTIME_DIR/biomeos/{name}-{family_id}.sock`)
- [x] First-byte peek (JSON-RPC vs BTSP auto-detection)
- [x] Ed25519 signing module in sourdough-genomebin (SD-03 resolved)
- [x] musl cross-compilation in release.yml (SD-02 resolved)

---

## Next (v0.4.0 — Harvest + Release)

Per deployment internalization contract
(primalSpring/docs/SOURDOUGH_DEPLOYMENT_INTERNALIZATION.md):

- [ ] `sourdough harvest --all` — cross-compile all primals per `sources.toml`
- [ ] `sourdough harvest --release` — checksum, stage, tag, push to GitHub Releases
- [ ] Asset carry-forward logic (currently in `auto-harvest.yml`) into Rust
- [ ] `sourdough validate composition` gains Phase 3 live health probes

---

## Medium Term (v0.5.0 — Package)

- [ ] `sourdough package` — self-extracting genomeBin archives
- [ ] Embed manifest, checksums, signature in archive header
- [ ] Support offline deployment to air-gapped gates

---

## Longer Term

### v0.6.0 — Deploy

- [ ] `sourdough deploy --target membrane` — full deploy+verify cycle
- [ ] Multi-target support: membrane, gate, nest topologies

### Ephemeral Primal Scaffolding

See `specs/EPHEMERAL_PRIMAL_SCAFFOLDING.md` for the full specification.

- [ ] `EphemeralOwner<T>` utility in sourdough-core
- [ ] Scoped capability namespacing (`session.{id}.*`)

---

## Quality Targets

| Metric | Current | Target |
|--------|---------|--------|
| Test coverage | 95%+ | >90% maintained |
| Tests passing | 281 | All passing |
| Clippy | zero warnings | zero warnings |
| Unsafe code | zero (forbid) | zero |
| C dependencies | zero | zero |
| Max file size | 750 lines | <1000 lines |

---

## How to Contribute

1. Pick an item from this list
2. Check `specs/` for related specifications
3. Follow `CONVENTIONS.md` for coding standards
4. Run the full verification suite before submitting:
   ```bash
   cargo test --workspace
   cargo clippy --workspace --all-targets -- -D warnings
   cargo fmt --all -- --check
   cargo deny check
   cargo doc --workspace --no-deps
   ```

---

**Date**: May 14, 2026
**Status**: v0.3.0 deployment internalization complete, harvest and release next
