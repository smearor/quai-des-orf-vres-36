//! `ArtAssets` - the generated SVG artwork, cached and drawable onto
//! a cairo context.
//!
//! `resources/art/{family}/{city}/{style}/` holds the font-free SVGs
//! produced by `qdo36-artwork`. Handles are loaded lazily and cached
//! per path; missing files resolve to `None` so callers fall back to
//! the `StyleRenderer` primitives.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use tangible_objects_model::PixelsPerMm;
use tracing::warn;

use crate::ui::art::ArtSpec;
use crate::ui::style::MapStyle;

/// The city the artwork namespace resolves under.
const CITY: &str = "paris";

/// Generated SVG artwork, loaded on demand from `resources/art`.
pub struct ArtAssets {
    /// The `resources/art` root.
    root: PathBuf,
    /// Loaded handles by file path.
    handles: RefCell<HashMap<PathBuf, Rc<rsvg::SvgHandle>>>,
    /// Paths that failed to load - do not retry every frame.
    misses: RefCell<std::collections::HashSet<PathBuf>>,
}

impl ArtAssets {
    /// Locates `resources/art` - first next to the running executable
    /// (installed layout), then relative to the crate (development).
    pub fn detect() -> Rc<Self> {
        let root = Self::find_root();
        Rc::new(Self {
            root,
            handles: RefCell::new(HashMap::new()),
            misses: RefCell::new(std::collections::HashSet::new()),
        })
    }

    fn find_root() -> PathBuf {
        if let Ok(exe) = std::env::current_exe()
            && let Some(parent) = exe.parent()
        {
            let candidate = parent.join("resources").join("art");
            if candidate.is_dir() {
                return candidate;
            }
        }
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/art"))
    }

    /// Draws the spec's asset: `art/{family}/{city}/{style}/{stem}.svg`
    /// scaled so the document is `spec.height` tall, its `anchor`
    /// fraction (document coordinates 0..1) sitting on `pos`.
    /// Returns `false` when the asset does not exist - the caller
    /// then paints its primitive fallback.
    pub fn draw(&self, ctx: &cairo::Context, style: MapStyle, spec: &ArtSpec<'_>, px_per_mm: PixelsPerMm) -> bool {
        let Some(handle) = self.handle(spec.family, style, spec.stem) else {
            return false;
        };
        let height = px_per_mm.scale(spec.height.as_f64());
        let (intrinsic_w, intrinsic_h) = self.intrinsic_size(&handle);
        if intrinsic_h <= 0.0 || intrinsic_w <= 0.0 {
            return false;
        }
        let width = height * intrinsic_w / intrinsic_h;
        let rect = cairo::Rectangle::new(spec.pos.x - spec.anchor.x * width, spec.pos.y - spec.anchor.y * height, width, height);
        if spec.alpha.as_f64() >= 0.999 {
            rsvg::CairoRenderer::new(&handle).render_document(ctx, &rect).is_ok()
        } else {
            ctx.push_group();
            let drawn = rsvg::CairoRenderer::new(&handle).render_document(ctx, &rect).is_ok();
            if drawn {
                ctx.pop_group_to_source().ok();
                ctx.paint_with_alpha(spec.alpha.as_f64()).ok();
            } else {
                ctx.new_path();
            }
            drawn
        }
    }

    /// The cached handle for `art/{family}/{city}/{style}/{stem}.svg`.
    fn handle(&self, family: &str, style: MapStyle, stem: &str) -> Option<Rc<rsvg::SvgHandle>> {
        let path = self.root.join(family).join(CITY).join(style.namespace()).join(format!("{stem}.svg"));
        if let Some(handle) = self.handles.borrow().get(&path) {
            return Some(handle.clone());
        }
        if self.misses.borrow().contains(&path) {
            return None;
        }
        match rsvg::Loader::new().read_path(&path) {
            Ok(handle) => {
                let handle = Rc::new(handle);
                self.handles.borrow_mut().insert(path, handle.clone());
                Some(handle)
            }
            Err(err) => {
                warn!(path = %path.display(), "artwork asset failed to load: {err}");
                self.misses.borrow_mut().insert(path);
                None
            }
        }
    }

    /// The SVG's natural document size (viewBox / width x height).
    /// The SVG's aspect: the `viewBox` where pixels are not absolute
    /// (our generated assets use `width="100%"`).
    fn intrinsic_size(&self, handle: &rsvg::SvgHandle) -> (f64, f64) {
        let renderer = rsvg::CairoRenderer::new(handle);
        if let Some(size) = renderer.intrinsic_size_in_pixels() {
            return size;
        }
        renderer
            .intrinsic_dimensions()
            .vbox
            .map(|vbox| (vbox.width(), vbox.height()))
            .unwrap_or((0.0, 0.0))
    }
}
