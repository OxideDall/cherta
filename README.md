# cherta

Wayland GPU-accelerated screen annotator with trail fade effect.

**cherta** - a minimalist drawing tool for Wayland with GPU acceleration and fade-out effects for strokes.

## Build

```bash
cargo build --release
```

## Run

```bash
./target/release/cherta
```

## Configuration

On first run, creates `~/.config/cherta/default.toml`:

```toml
thickness = 3.0
color = [1.0, 0.0, 0.0]     # RGB (0.0-1.0)
opacity = 0.9               # initial stroke opacity
ttl = 2.0                   # stroke lifetime in seconds
fade_start = 1.5            # when fade-out begins
smooth_lines = true         # remove close points for smoothness
min_point_distance = 2.0    # minimum distance between points (px)
line_feather = 0.0          # (not yet implemented, requires quad-rendering)
scroll_cooldown = 500       # pause (ms) after scroll before polling
polling_interval = 50       # polling interval (ms) for LMB in PASSTHROUGH
```

## Controls

**FSM with two states:**

### CAPTURING mode (on startup)

- **LMB pressed** → draw
- **LMB released** → stop drawing, stay in CAPTURING
- **Scroll (up/down)** → switch to PASSTHROUGH mode

### PASSTHROUGH mode

- All input passes through to other windows
- Strokes continue to fade (rendering stays active)
- **Smart polling:** After `scroll_cooldown` ms of silence, check LMB every `polling_interval` ms
  - During active scrolling → polling disabled, smooth scrolling
  - Finished scrolling → after `scroll_cooldown` ms start checking LMB
- **LMB pressed** → return to CAPTURING mode

**Trade-off:** First scroll exits capture mode. After that, scrolling works perfectly.

Edit config and restart `cherta` to apply changes.

## Features

- GPU rendering via GLES2
- Trail fade-out in shader with configurable timing
- **Smooth lines:** decimation of close points to reduce jaggedness
- **GL_LINE_SMOOTH:** hardware edge smoothing (if supported)
- Layer shell overlay (transparent above all windows)
- FSM for input capture management (scroll-escape)
- Left mouse button drawing
- Configurable color, thickness, opacity
- Front fade-out effect
- Smart polling with pause after scroll
- 60 FPS rendering

## Roadmap

- [ ] Quad-based rendering for true feathering (soft edges)
- [ ] Pressure sensitivity for graphics tablets
- [ ] Export annotations to SVG/PNG

## Requirements

- Wayland compositor with wlr-layer-shell support (Hyprland, Sway, etc.)
- EGL + GLESv2

## License

MIT License - see LICENSE file for details
