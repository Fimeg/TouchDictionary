# TouchDictionary

**WARNING: ALPHA SOFTWARE - NOT READY FOR PRODUCTION**

**This is experimental software in active development. Features may be broken, bugs are expected, and breaking changes happen frequently. Use at your own risk, preferably on test systems only. Seriously, don't put this in production yet.**

## What It Does

- 3-finger tap on any word to get instant definitions
- Combines dictionary definitions with Wikipedia summaries  
- Frosted glass popup that looks... acceptable
- Written in Rust + Tauri (because why not)

## Quick Start

```bash
cd touchdictionary/gui
cargo run -- your-word-here
```

## Known Issues

- UI is janky
- Sometimes it crashes for no reason
- Wikipedia formatting is minimal at best
- Probably other things I haven't found yet

## Requirements

- Rust + Cargo
- Tauri dependencies
- libinput-gestures (for 3-finger tap)

## License

MIT (do whatever you want with this buggy code)
