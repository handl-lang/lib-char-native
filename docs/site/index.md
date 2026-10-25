# lib-char-native

The Rust `std::char` backend for the `char` contract.

## What it is

`char-native` implements the `char` contract over Rust's `std::char`, i.e. it
is the **Unicode-capable** backend: `is_whitespace`, `is_alpha` and friends are
read per the Unicode Character Database rather than per ASCII.

## Modules
- `handl-packages/` — the Handl package that exposes the native ops to `.hnl` code
- `src/` — the Rust implementations behind those ops


## Start here

- **Reference** — [public API](/lib-char-native/reference/)
- **How-to** — [task recipes](/lib-char-native/how-to/)
- **Explanation** — [design and rationale](/lib-char-native/explanation/)

No tutorials: this repo is a building block used from Rust. End-to-end
walkthroughs live with the product that composes it — see the
[Handl platform](/handl/).
