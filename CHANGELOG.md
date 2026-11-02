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
