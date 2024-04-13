# Muse Package Manager (MPM)
[![Release](https://github.com/nightcycle/muse-package-manager/actions/workflows/release.yml/badge.svg)](https://github.com/nightcycle/muse-package-manager/actions/workflows/release.yml)

Say hello to the bare minimum for a package manager! This will compile open source muse packages into a single source, allowing for you to mass update your myth's dependencies with a single command.

## Running

### All Myths in Map
To run, in vscodium just call
```sh
path-to-mpm.exe install
```

### Specific Myth
To run, in vscodium just call
```sh
path-to-mpm.exe install --myth MythNameHere
```


## Config Format
This is used at a myth level to determine which packages to download. It needs to be named `muse-package.toml`, otherwise it wont' be detected.

Example:
```toml
deprecated = false # unimplemented, will eventually send a warning to anyone that installs it for them to update

[dependencies]
Signal="https://github.com/nightcycle/muse-packages/releases/tag/v0.2.0/src/signal"
Option="https://github.com/nightcycle/muse-packages/releases/tag/v0.2.0/src/option"
```

A dependency contains multiple parts
- a name, used for the script name + namespace
- a github release url with verison tag
- a local path guiding it to the specific directory containing the scripts 
- it will download this directory, and assemble a script out of the containing scripts

The assembly logic is not bulletproof, double check your stuff compiles errorlessly with 
```sh 
path-to-mpm.exe build --input dir-path-here --output file-path-here.cs
```
