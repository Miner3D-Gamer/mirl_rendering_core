//! Cache for visual characters
//!
// I despise how much duplicate code is present in the text rendering functions

// use mirl::settings::SettingsMapType;
use mirl_extensions::{IntoPatch, MapLike};
use parking_lot::RwLock;
// /// Caches drawn text
// ///
// /// See [`GlyphCache`] for the composition of this type
// pub static GLYPH_CACHE: GlyphCache = std::sync::LazyLock::new(|| RwLock::new(MapType::new_map()));

/// A trait used by text drawing functions
pub trait GlyphCache {
    /// Clear the entire glyph cache
    fn clear(&mut self);
    /// Remove from glyph cache
    fn remove(&mut self, glyph: Glyph) -> Option<std::sync::Arc<GlyphData>>;
    /// Insert a glyph, replacing the old
    fn insert(&mut self, glyph: Glyph) -> std::sync::Arc<GlyphData>;
    /// Get a glyph from cache or add it if it doesn't exist
    fn get_or_insert(&mut self, glyph: Glyph) -> std::sync::Arc<GlyphData>;
    /// Get a glyph from the cache if it exists
    fn get(&self, glyph: Glyph) -> Option<std::sync::Arc<GlyphData>>;
    /// Get a reference to the internal font used
    fn get_font(&self) -> &fontdue::Font;
}
#[mirl_macros::derive_all]
/// The size of the glyph
///
/// Internally a f32 wrapper that will always be a valid value
pub struct GlyphSize {
    /// The size of the glyph
    pub _size: mirl_narrowed_floats::PositiveFinite<f32>,
}

impl GlyphSize {
    #[must_use]
    #[inline]
    /// Create a new [`GlyphSize`]
    ///
    /// Returns [`None`] if the given f32 is negative, Nan/Inf, or so big it looses notable precision
    pub fn new<T: IntoPatch<f32>>(size: T) -> Option<Self> {
        let size = size.into_value();
        let narrowed = mirl_narrowed_floats::PositiveFinite::new(size)?;

        Some(Self { _size: narrowed })
    }
    #[inline(always)]
    #[must_use]
    /// Get the inner value
    pub const fn inner(self) -> mirl_narrowed_floats::PositiveFinite<f32> {
        self._size
    }
    #[inline(always)]
    #[must_use]
    /// Get the value as T where f32 can turn into T
    pub fn get<T>(self) -> T
    where
        f32: IntoPatch<T>,
    {
        self._size.into_inner().into_value()
    }
}

#[mirl_macros::derive_all]
/// A single glyph
pub struct Glyph {
    /// The internal char
    pub char: char,
    /// The size of the char
    pub size: GlyphSize,
    /// The color of the glyph
    pub color: u32,
}
impl Glyph {
    #[must_use]
    #[inline]
    /// Create a new glyph using a [`char`], it' [`GlyphSize`], and its color packed in a u32 (rgba)
    pub const fn new(char: char, size: GlyphSize, color: u32) -> Self {
        Self { char, size, color }
    }
}

/// The data required to draw a glyph
pub type GlyphData = (fontdue::Metrics, Vec<u8>);

// Copied from
/// Encapsulates all layout information associated with a glyph for a fixed scale.
#[mirl_macros::derive_all]
pub struct GlyphMetrics {
    /// Whole pixel offset of the left-most edge of the bitmap. This may be negative to reflect the
    /// glyph is positioned to the left of the origin.
    pub xmin: i32,
    /// Whole pixel offset of the bottom-most edge of the bitmap. This may be negative to reflect
    /// the glyph is positioned below the baseline.
    pub ymin: i32,
    /// The width of the bitmap in whole pixels.
    pub width: usize,
    /// The height of the bitmap in whole pixels.
    pub height: usize,
    /// Advance width of the glyph in subpixels. Used in horizontal fonts.
    pub advance_width: mirl_narrowed_floats::PositiveFinite<f32>,
    /// Advance height of the glyph in subpixels. Used in vertical fonts.
    pub advance_height: mirl_narrowed_floats::PositiveFinite<f32>,
    /// The bounding box that contains the glyph's outline at the offsets specified by the font.
    /// This is always a smaller box than the bitmap bounds.
    pub bounds: OutlineBounds,
}
/// Defines the bounds for a glyph's outline in subpixels. A glyph's outline is always contained in
/// its bitmap.
#[mirl_macros::derive_all]
pub struct OutlineBounds {
    /// Subpixel offset of the left-most edge of the glyph's outline.
    pub xmin: mirl_narrowed_floats::PositiveFinite<f32>,
    /// Subpixel offset of the bottom-most edge of the glyph's outline.
    pub ymin: mirl_narrowed_floats::PositiveFinite<f32>,
    /// The width of the outline in subpixels.
    pub width: mirl_narrowed_floats::PositiveFinite<f32>,
    /// The height of the outline in subpixels.
    pub height: mirl_narrowed_floats::PositiveFinite<f32>,
}
#[derive(Debug)]
/// The default glyph cache
pub struct DefaultGlyphCache<
    HashMap: MapLike<Glyph, std::sync::Arc<GlyphData>> = std::collections::HashMap<
        Glyph,
        std::sync::Arc<GlyphData>,
    >,
> {
    /// The cache containing
    pub _cache: parking_lot::RwLock<HashMap>,
    /// The font
    pub _font: fontdue::Font,
}
impl<HashMap: MapLike<Glyph, std::sync::Arc<GlyphData>> + PartialEq> PartialEq
    for DefaultGlyphCache<HashMap>
{
    fn eq(&self, other: &Self) -> bool {
        self._cache.read().eq(&self._cache.read())
            && are_fontdue_fonts_eq(&self._font, &other._font)
    }
}
#[must_use]
/// Checks if 2 fontdue font files are the same
///
/// Does not check `horizontal_kern`
pub fn are_fontdue_fonts_eq(f1: &fontdue::Font, f2: &fontdue::Font) -> bool {
    f1.chars().eq(f2.chars())
        && f1.name() == f2.name()
        && f1.units_per_em() == f2.units_per_em()
        && f1.glyph_count() == f2.glyph_count()
        && f1.horizontal_line_metrics(1.0) == f2.horizontal_line_metrics(1.0)
        && f1.vertical_line_metrics(1.0) == f2.vertical_line_metrics(1.0)
        && format!("{f1:?}") == format!("{f2:?}")
}

impl<HashMap: MapLike<Glyph, std::sync::Arc<GlyphData>> + Clone> Clone
    for DefaultGlyphCache<HashMap>
{
    fn clone(&self) -> Self {
        Self {
            _cache: RwLock::new(self._cache.read().clone()),
            _font: self._font.clone(),
        }
    }
}
impl<HashMap: MapLike<Glyph, std::sync::Arc<GlyphData>> + Default> DefaultGlyphCache<HashMap> {
    #[must_use]
    /// Create a new empty glyph cache
    pub fn new(font: fontdue::Font) -> Self {
        Self {
            _cache: RwLock::new(HashMap::default()),
            _font: font,
        }
    }
}
impl<HashMap: MapLike<Glyph, std::sync::Arc<GlyphData>> + Clone> GlyphCache
    for DefaultGlyphCache<HashMap>
{
    fn clear(&mut self) {
        let lock = self._cache.get_mut();
        for (key, _values) in lock.clone().iter() {
            lock.remove(key);
        }
    }

    fn remove(&mut self, glyph: Glyph) -> Option<std::sync::Arc<GlyphData>> {
        let lock = self._cache.get_mut();
        lock.remove(&glyph)
    }

    fn insert(&mut self, glyph: Glyph) -> std::sync::Arc<GlyphData> {
        let lock = self._cache.get_mut();
        let rasterized = self._font.rasterize(glyph.char, glyph.size._size.into_inner());
        let arc = std::sync::Arc::new(rasterized);
        lock.insert(glyph, arc.clone());
        arc
    }

    fn get_or_insert<'a>(&mut self, glyph: Glyph) -> std::sync::Arc<GlyphData> {
        self.get(glyph).unwrap_or_else(|| self.insert(glyph))
    }

    fn get<'a>(&self, glyph: Glyph) -> Option<std::sync::Arc<GlyphData>> {
        let lock = self._cache.read();
        lock.get(&glyph).map(std::clone::Clone::clone)
    }
    fn get_font(&self) -> &fontdue::Font {
        &self._font
    }
}

/// Get the length of a string in a font if it was rendered out
#[must_use]
pub fn get_text_width<T: GlyphCache>(
    string: &str,
    size: GlyphSize,
    cache: &mut T,
) -> mirl_narrowed_floats::PositiveFinite<f32> {
    let mut total_width = 0.0;

    for ch in string.chars() {
        let metrics = cache.get_or_insert(Glyph::new(ch, size, 0)).0;

        total_width += metrics.advance_width;
    }

    unsafe { mirl_narrowed_floats::PositiveFinite::new(total_width).unwrap_unchecked() }
}
/// Get the height of a string in a font if it was rendered out
#[must_use]
pub fn get_text_height<T: GlyphCache>(
    string: &str,
    size: GlyphSize,
    cache: &mut T,
) -> mirl_narrowed_floats::PositiveFinite<f32> {
    let mut max_height = size.inner().into_inner();
    let mut min_height = 0.0;

    for ch in string.chars() {
        let metrics = cache.get_or_insert(Glyph::new(ch, size, 0)).0;

        if metrics.height as f32 > max_height {
            max_height = metrics.height as f32;
        }
        if (metrics.ymin as f32) < min_height {
            min_height = metrics.ymin as f32;
        }
    }
    unsafe {
        mirl_narrowed_floats::PositiveFinite::new(max_height - min_height).unwrap_unchecked()
    }
}
