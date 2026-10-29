# Assets made for this build

Files here are the user's own art, made for this build (not taken from the game disc, and not
derived from it). The game loads them at run time and falls back to its built-in stand-ins
when they are missing.

- `models/cloud_beast.obj`: the Cloud Beast (the user's "Tempest Golem", made with Meshy; the
  disc has no model for this boss, SPEC-0010). Loaded by `crates/neopets-app/src/placeholder.rs`
  from NEOPETS_CLOUD_MODEL, then `assets/models/` beside the executable or in the working
  folder, then `private/models/`.
