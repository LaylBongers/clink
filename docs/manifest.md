# Clink.toml Manifest
This file tells CLink how to interpret your project.

## Project Types
This describes what type of project you want CLink to generate files for.
Project types are listed under the `[package]` section as the `type` value.
Currently available types are:
- "application" An executable application
- "library" A linkable static library
- "external" A custom external dependency (not yet implemented)

```toml
[project]
name = "SomeLibrary"
type = "library" # <-
```
