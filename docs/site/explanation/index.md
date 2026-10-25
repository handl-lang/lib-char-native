# Explanation

## Why this crate exists

The platform needs to run the same string library with different notions of
«letter»: a host may want ASCII for speed or Unicode for correctness. Keeping the
Rust `std::char` reading in a plugin makes that a composition decision instead of
a fork of the library.

## Where it sits

This crate belongs to the `engine-*` family. Plugins, kits and hosts reach it
through the kit's operation slot, so the engine can be swapped without touching a
language definition.

See the [reference](/lib-char-native/reference/) for the API and the
[how-to guides](/lib-char-native/how-to/) for tasks.
