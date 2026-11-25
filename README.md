# lib-char-native

`handl-plugin-lib-char-native` — Native `char` module — the Rust `std::char` backend for the `char` contract.

Sibling-repo deps are git dependencies (`branch = "main"`); `[patch]` tables
resolve them from sibling checkouts.

## Layout

```text
handl-packages/char-native/   the .hnl package
src/                          the Rust plugin crate (lib.rs, ops.rs, tests.rs)
scripts/hnl-gate.sh           the gate
workflows/gate.yaml                         bupha entry (handl-gates)
```

## Gate

```bash
HNL=../host-cli/target/debug/hnl scripts/hnl-gate.sh
```

## License

MIT.

## Documentation

<https://handl-lang.github.io/docs/lib-char-native/>
