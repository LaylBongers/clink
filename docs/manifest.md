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
[package]
name = "SomeLibrary"
type = "library" # <-
```

## Compile and Include Paths
You can define custom paths for CLink to search for files to compile and to add
to the include path. The default value for `compile` is `["src"]`, the default
value for `include` is `["include"]`.

```
[package]
name = "SomeLibrary"
type = "library"
compile = ["../somelibrary/src", "./src_ext"] # <-
include = ["../somelibrary/include", "./include_ext"] # <-
```
