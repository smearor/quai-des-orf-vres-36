# Quai des Orfèvres 36

A Scotland Yard clone for the multitouch table — written in Rust with
GTK4. The Paris map is anchored to a physical **Eiffel tower tangible**
(4 conductive pins): wherever you place and rotate the tower, the map
follows. Detectives move physical pawns or tap stations; the fugitive
**Monsieur Corbeau** enters hidden moves behind a physical sight shield.

Built on top of:

- [tangible-objects-rs](https://github.com/smearor/tangible-objects-rs) —
  multitouch tangible detection daemon (GraphQL interface)
- [fonts-rs](https://github.com/smearor/fonts-rs) — icon sets for GTK4

## Requirements

- Rust latest stable − 1 (currently **1.98.0**), edition 2024
- GTK4 + libadwaita + gtk4-layer-shell development libraries
  (`libgtk-4-dev libadwaita-1-dev libgtk4-layer-shell-dev`)
- For tangible play: the `tangible-objects` daemon running with
  `--filter --device /dev/input/eventX --graphql-listen 127.0.0.1:8080`

## Build & run

```bash
cargo build --workspace --release
./target/release/qdo36 --daemon-url http://127.0.0.1:8080
```

Options: `--daemon-url`, `--locale en|de`, `--mcp-listen 127.0.0.1:8091`.
Set `QDO36_NO_FULLSCREEN` to run windowed.

## Layout

| Path | Content |
|------|---------|
| `crates/core` | `qdo36-core` — Paris map graph + game domain |
| `crates/tangibles` | `qdo36-tangibles` — daemon adapter, anchor transform |
| `crates/icons` | `qdo36-icons` — fonts-rs icon set |
| `crates/mcp` | `qdo36-mcp` — game-domain MCP server |
| `crates/app` | `qdo36` — GTK4/libadwaita application |
| `concepts/planned` | work packages, roadmap, artwork/style-book |
| `i18n/{en,de}` | Fluent localization |

## License

MIT — see [LICENSE](LICENSE). Icon upstreams are credited in
[NOTICE](NOTICE).
