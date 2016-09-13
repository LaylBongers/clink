# Clink
A simple C++ build system generator. Simplify your Visual Studio project linking.

## Getting Started
1. Download clink.exe from github releases.
2. Place it somewhere in your PATH system variable.
3. Create a *Clink.toml* file for every project you want.
4. Run `clink` in your command shell in the project folder you want to generate
    Visual Studio projects for.

Clink expects your *.hpp* files to be in the *include* directory.

### Example *Clink.toml* files
```toml
[package]
name = "MyGame"
type = "binary"

[dependencies]
AmazingEngine = "../../Engine/Libraries/AmazingEngine"
```

```toml
[package]
name = "AmazingEngine"
type = "library"

[dependencies]
```

## Filter-only Usage
You can also use clink to only generate a *.vcxproj.filters* file. Keep in mind
that when doing this, clink will not update your *.vcxproj* with new or moved
files, it will only re-order them in filters.

1. Follow steps 1-3 from **Getting Started**. You can omit dependencies if you
    plan to only use clink for filters.
2. *Make sure you have a Clink.toml file with a name matching the project you
    want to generate for.*
3. Run `clink filters` in your command shell in the project folder you want to
    generate Visual Studio filters for.
4. Reload your project in Visual Studio. (This is unfortunately needed because
    Visual Studio does not detect changes to the filters file)
5. (Optional) Create a .cmd file or build event to automatically generate
    filters.

## License
Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
