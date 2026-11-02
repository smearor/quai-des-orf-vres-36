# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Workspace scaffold: `qdo36-core`, `qdo36-tangibles`, `qdo36-icons`,
  `qdo36-mcp` and the `qdo36` GTK4 application crate
- Project documentation: roadmap, work packages WP01–WP09, artwork and
  style-book under `concepts/planned/`
- `AppShell`: fullscreen GTK4/libadwaita window, daemon setup
  screen with auto-advance on anchor detection, `MapScene` rendering
  the Paris map under `MapTransform` (live pan/rotation from the
  Eiffel tower tangible), tap hit-testing via `nearest_station`,
  "tower missing" overlay, Fluent L10n (`i18n/{en,de}`), persisted
  `AppSettings` with CLI overrides
- `MapStyle`/`StyleRenderer` seam with all three style renderers:
  Comic (default), Belle Époque (parchment, sepia engraving strokes,
  gold accents, Cormorant Garamond labels) and Neon; swappable via
  `settings.map_style`. Semantic `StylePalette` tokens shared across
  styles
- `qdo36-artwork` generator crate: parametrized SVG masters organized
  as self-contained `masters/{city}/{style}/` packs (masters +
  `variants.toml`), deterministic output into
  `resources/art/{family}/{city}/{style}/` (208 assets: ligne, pawn,
  station, ticket, landmarks, corbeau, map, splash)
- Font pipeline: bundled OFL TTFs per style (`resources/fonts/`),
  style-specific font stacks, and an optional `outline` feature that
  converts SVG `<text>` to glyph paths at generation time
  (fontdb + rustybuzz) - shipped artwork carries no font dependency
- Generated artwork wired into the scene: `ArtAssets` (rsvg handle
  cache, lazy load, primitive fallback) renders landmark pins, the
  Eiffel anchor marker and a map-rotating compass; typed seam via
  `ArtSpec`/`AssetStem`/`AnchorFrac`/`ArtFamily`/`DocumentSize`
- Bundled fonts registered with fontconfig at app start
  (`fonts::install`, `FcConfigAppFontAddDir`) so live Pango UI text
  resolves the bundled families
- `heading_offset_deg` calibration (settings + `--heading-offset`
  flag, `HeadingOffsetDeg` newtype) feeding the existing
  `AnchorConfig.heading_offset` map-north correspondence
- `qdo36-mcp` server: Streamable HTTP endpoint (`/mcp`) on a dedicated
  Tokio thread behind `--mcp-listen`, host allowlisting for concrete
  binds, typed `StateSnapshot` over a `watch` channel, and the
  `anchor_state`/`map_info` introspection tools
- WP06 rules engine (`qdo36-session`): `GameSession::deal` samples
  curated start stations via injected RNG (deterministic deals for
  tests and the balancing simulator); `begin` opens the turn cycle.
  `request_move` runs the ordered pipeline phase → turn → ticket →
  route → occupancy → apply with per-reason `MoveError`s (incl.
  `TicketShapeMismatch`, `BatobusRequiresBlack`); side effects:
  position/stock updates, spent detective tickets feeding Fugitive's
  pool, `GameLog` records, stuck-detective turn skips, and the three
  win conditions (capture, round cap, all-stuck). `legal_destinations`
  powers UI hints and simulations; chess-style `undo`/`redo`
  (`allow_undo` + `InProgress` gated) returns a `RestorationPlan` that
  keeps Fugitive moves physically invisible; `SavedGame` snapshots the
  full session to TOML (`ordered_map` serde keeps typed map keys) and
  re-validates on restore. Configurable `detective_stock`/
  `fugitive_stock` on `GameConfig`/`MapMeta` for city balancing.
  Engine split `session/{mod,validate,apply,undo,tests}.rs`;
  `SeatCount`/`MoveShape`/`TicketCounts::{UNLIMITED,DETECTIVE_START,
  FUGITIVE_START,gain}`/`Transport::for_ticket`/`DetectiveCount::pawns`
