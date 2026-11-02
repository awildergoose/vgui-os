//! This is used as a generic struct to hold colors and blit
//! them onto surfaces.
//!
//! ```rs
//! let red = Color::RED;
//! unsafe { red.blit(0); }
//! ```
use crate::framebuffer::{Surface, get_framebuffer_surface_mut};

/// RGBA color, from 0-255.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Color {
    /// Red.
    pub r: u8,
    /// Green.
    pub g: u8,
    /// Blue.
    pub b: u8,
    /// Alpha.
    pub a: u8,
}

/// Converts an HSV color to an RGB color tuple
#[must_use]
#[allow(clippy::many_single_char_names)]
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    #[inline]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn to_byte(x: f32) -> u8 {
        let y = core::f32::math::mul_add(x, 255.0, 0.5);

        if y <= 0.0 {
            0
        } else if y >= 255.0 {
            255
        } else {
            (y as i32) as u8
        }
    }

    let mut h = h - core::f32::math::floor(h);
    h *= 6.0;

    #[allow(clippy::cast_possible_truncation)]
    let i = h as i32;
    let f = core::f32::math::fract(h);

    let p = v * (1.0 - s);
    let q = v * core::f32::math::mul_add(s, -f, 1.0);
    let t = v * core::f32::math::mul_add(s, -(1.0 - f), 1.0);

    let (r, g, b) = match i {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };

    (to_byte(r), to_byte(g), to_byte(b))
}

impl Color {
    /// Black.
    pub const BLACK: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    /// Blue.
    pub const BLUE: Self = Self {
        r: 0,
        g: 0,
        b: 255,
        a: 255,
    };
    /// Brown.
    pub const BROWN: Self = Self {
        r: 165,
        g: 42,
        b: 42,
        a: 255,
    };
    /// Cyan.
    pub const CYAN: Self = Self {
        r: 0,
        g: 255,
        b: 255,
        a: 255,
    };
    /// Dark gray.
    pub const DARK_GRAY: Self = Self {
        r: 64,
        g: 64,
        b: 64,
        a: 255,
    };
    /// Gray.
    pub const GRAY: Self = Self {
        r: 128,
        g: 128,
        b: 128,
        a: 255,
    };
    /// Green.
    pub const GREEN: Self = Self {
        r: 0,
        g: 255,
        b: 0,
        a: 255,
    };
    /// Light gray.
    pub const LIGHT_GRAY: Self = Self {
        r: 211,
        g: 211,
        b: 211,
        a: 255,
    };
    /// Magenta.
    pub const MAGENTA: Self = Self {
        r: 255,
        g: 0,
        b: 255,
        a: 255,
    };
    /// Orange.
    pub const ORANGE: Self = Self {
        r: 255,
        g: 165,
        b: 0,
        a: 255,
    };
    /// Pink.
    pub const PINK: Self = Self {
        r: 255,
        g: 192,
        b: 203,
        a: 255,
    };
    /// Purple.
    pub const PURPLE: Self = Self {
        r: 128,
        g: 0,
        b: 128,
        a: 255,
    };
    /// Red.
    pub const RED: Self = Self {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };
    /// Transparent.
    pub const TRANSPARENT: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };
    /// White.
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    /// Yellow.
    pub const YELLOW: Self = Self {
        r: 255,
        g: 255,
        b: 0,
        a: 255,
    };

    /// Returns a new RGBA color.
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Returns a new RGB color.
    #[must_use]
    pub const fn new_opaque(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Blits the color onto the global surface.
    ///
    /// # Safety
    /// This directly sets pixels in the framebuffer without checking the index
    #[inline]
    pub unsafe fn blit(self, index: usize) {
        unsafe {
            self.blit_ex(get_framebuffer_surface_mut(), index);
        }
    }

    /// Blits the color onto `surface`.
    ///
    /// # Safety
    /// This directly sets pixels in the framebuffer without checking the index
    #[inline]
    pub unsafe fn blit_ex(self, surface: &mut Surface, index: usize) {
        unsafe {
            let pixel_ptr = surface.rgba.add(index);
            *pixel_ptr = [self.b, self.g, self.r, self.a];
        }
    }
}
