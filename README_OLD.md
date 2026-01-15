# CHARON

CHARON is the terminal user interface for managing SHADOW.

## Overview
**Role:** terminal user interface  
**Platforms:** Linux  
**Idea:** Makes it possible to manage SHADOW 

## Capabilities:
- Customize, generate (possibly in the future...) and deploy implants
- Manage SHADOW

## How to run

### Development use

Run the app by executing:
```bash
cargo run
```

### Production use

> TODO

## TODO

- [x] how should SHADOW be deployed to enable this sort of orchestration? docker, service, etc? -> a multi-stage Docker file (first build, then lightweight runtime)
- [x] create a moodboard of designs (preferably ASCII art, it's really pretty)
- [x] basic menu panel
- [x] SHADOW integration
- [x] live implant panel (live/dead GHOSTs)
- [ ] [EXTRA] builder for GHOST payloads -> improve this to compile GHOSTs and expose them on the `/file` endpoint of SHADOW
- [ ] [EXTRA] local cache (preferably a database; would be nice if it also cached requests)
- [x] design a better structure, as cramming everything into a single file will quickly become highly unmaintainable (I think xD)
- [ ] [EXTRA] add a help section with guide on how to use the currently active window
- [ ] ship the readme from current work in progress to a polished, user friendly one (will probably also keep the TODO thing, but it a separate README, or in a roadmap type section [or track them on the kanban, also a cool option!!])
- [ ] when GHOST exceeds it's death timer by 10x (or some other magic number), remove from list
- [ ] if GHOST comes back alive, show special symbol (or smth) in STATUS column to differentiate from normally alive GHOSTs
- [ ] add directory navigation to the terminal panel (check if GHOSTs can even do that? I don't think paths are cached rn, so each command defaults to `~`; think if this "feature" is needed for the operator)
- [ ] GHOST remove functionality to dashboard (`x` menu action)

## Legal

> **Disclaimer:** This software is for educational purposes and authorized red team engagements only. The authors are not responsible for misuse.