# Clink
A simple C++ build system generator. Simplify your Visual Studio project linking.

## Getting Started
1. Download clink.exe from github releases.
2. Place it somewhere in your PATH system variable.
3. Create a *Clink.toml* file for every project you want.
4. Run `clink` in your command shell in the executable you want to generate
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

## License
Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
