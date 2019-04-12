# stripng

[![Build status](https://api.travis-ci.org/FauxFaux/stripng.png)](https://travis-ci.org/FauxFaux/stripng)
[![](https://img.shields.io/crates/v/stripng.svg)](https://crates.io/crates/stripng)

Remove all non-critical chunks from a `.png` (Portable Network Graphics) file.

This reduces the size, and increases the reproducibility. Use `optipng` (too!)
to squeeze the actual data down.

Overwrites the file in place.
