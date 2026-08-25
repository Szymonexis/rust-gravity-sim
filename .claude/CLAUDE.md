# Project instructions

## Commits
- Do not create any git commits unless explicitly told to in the request.

## Comments
- Do not write comments. This covers `//` explanations, `///` doc comments and `//!` module docs, in Rust and in WGSL alike. Name things well instead and let the code carry the explanation.
- One exception, because it is not commentary: `///` on a config type that derives `JsonSchema`, and on its fields and variants. Schemars turns that text into the `description` entries of `app-config.schema.json`, which is what documents the settings file in the user's editor. Deleting it silently strips the schema.
- When editing existing code, drop any comments you find in the parts you touch.
