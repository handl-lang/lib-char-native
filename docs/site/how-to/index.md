# How-to guides

Task recipes. Each assumes sibling checkouts of this repo and of the platform
repos it patches.

## Swap character semantics

Replace this backend with an ASCII-only one and `str-handl` behaviour changes with it — no string algorithm needs editing.

## Add a native char op

Extend the contract first (`lib-char`), then implement it here; the contract is what other backends must also satisfy.
