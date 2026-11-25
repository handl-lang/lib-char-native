# lib-char-native

`handl-plugin-lib-char-native` — the Rust `std::char` backend for the
`char` contract (Unicode-semantics `char.*` ops).

## Layout

| path | role |
| --- | --- |
| `handl-packages/char-native/` | the `.hnl` package surface |
| `src/lib.rs`, `src/ops.rs`, `src/tests.rs` | the Rust plugin crate |
| `atoms.yaml`, `build.rs`, `Cargo.toml`, `xtask/` | atom registry + build + gate |
| `pkg.hnl`, `scripts/hnl-gate.sh` | manifest + `.hnl` gate |
| `workflows/gate.yaml` | CI — shared `handl-gates` campaign |

## Contract

- Sibling-repo deps are git dependencies (`branch = "main"`); `[patch]`
  tables resolve them from sibling checkouts.
- Implements the axioms declared by `lib-char` (`deps: { char: ... }`).

## Verification

```bash
HNL=../host-cli/target/debug/hnl scripts/hnl-gate.sh
cargo xtask gate
```

<!-- myco:agent-awareness -->

## Agent awareness

- Session start / any work discussion: run `myco status --prompt`
  before proposing next steps — know the board first.
- User describes new work → offer to capture it (`myco todo add`,
  `issue new`, `proposal new`). Don't wait to be asked.
- Picking up a task: `myco resume <ID>` is the briefing.
- Wrapping up: `myco handoff <ID>` anything resumable; keep statuses
  honest (claimed → in-progress → blocked/done).

<!-- /myco:agent-awareness -->
