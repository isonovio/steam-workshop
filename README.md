# steam-workshop

CLI for interacting with the Steam Workshop.

An adaptation of [KP-Steam](https://github.com/KillahPotatoes/KP-Steam)

## Install

macOS ARM64 (Steam must be running):

```sh
mise use github:isonovio/steam-workshop@latest
steam-workshop probe-steam
```

## Tech Stack

- `steamworks::sys`: native bindings and Valve's library
