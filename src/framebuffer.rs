//! This is used to manage and control the framebuffer.
//!
//! Example:
//! ```rs
//! get_framebuffer_surface_mut().clear(Color::GREEN);
//! ```
use core::alloc::Layout;

use limine::framebuffer::Framebuffer;
use spin::{Mutex, MutexGuard};

use crate::color::Color;

/// The global framebuffer surface. (backbuffer)
pub static FRAMEBUFFER_SURFACE: Mutex<Option<Surface>> = Mutex::new(None);
/// The global framebuffer surface. (real buffer)
pub static REAL_FRAMEBUFFER_SURFACE: Mutex<Option<Surface>> = Mutex::new(None);

/// Pointer to the raw framebuffer.
pub type RawFramebufferPointer = *const [u8; 4];
/// Mutable pointer to the raw framebuffer.
pub type RawFramebufferPointerMut = *mut [u8; 4];

/// Returns the pointer of the global framebuffer.
/// The host only calls this function once.
#[unsafe(no_mangle)]
pub extern "C" fn get_framebuffer_ptr() -> RawFramebufferPointer {
    get_framebuffer_surface_ref().rgba.cast_const()
}

/// Returns the mutable pointer of the global framebuffer.
#[must_use]
pub fn get_framebuffer_ptr_mut() -> RawFramebufferPointerMut {
    get_framebuffer_surface_mut().rgba as RawFramebufferPointerMut
}

fn get_framebuffer_surface() -> MutexGuard<'static, Option<Surface>> {
    FRAMEBUFFER_SURFACE.lock()
}

/// Returns the framebuffer surface as a reference.
///
/// # Panics
///
/// Panics if the framebuffer was not initialized.
#[must_use]
#[allow(clippy::significant_drop_tightening)]
pub fn get_framebuffer_surface_ref() -> &'static Surface {
    let guard = get_framebuffer_surface();
    let surface_ref: &Surface = guard.as_ref().expect("surface not initialized");
    unsafe { &*core::ptr::from_ref::<Surface>(surface_ref) }
}

/// Returns the framebuffer surface as a mutable reference.
///
/// # Panics
///
/// Panics if the framebuffer was not initialized.
#[must_use]
#[allow(clippy::significant_drop_tightening)]
pub fn get_framebuffer_surface_mut() -> &'static mut Surface {
    let mut guard = get_framebuffer_surface();
    let surface_mut: &mut Surface = guard.as_mut().expect("surface not initialized");
    unsafe { &mut *core::ptr::from_mut::<Surface>(surface_mut) }
}

/// Initializes the framebuffer surface.
pub fn init_fb(fb: &Framebuffer) {
    // real buffer
    let mut guard = REAL_FRAMEBUFFER_SURFACE.lock();

    let width = fb.width as usize;
    let height = fb.height as usize;
    let ptr = fb.address().cast::<[u8; 4]>();

    *guard = Some(Surface {
        rgba: ptr,
        width,
        height,
    });

    // backbuffer
    let mut bguard = FRAMEBUFFER_SURFACE.lock();
    *bguard = Some(Surface::new_empty(width, height));
}

pub fn swap_buffers() {
    unsafe {
        let mut bb = FRAMEBUFFER_SURFACE.lock();
        let back = bb.as_mut().unwrap();

        core::ptr::copy_nonoverlapping(
            back.rgba.cast::<u8>(),
            REAL_FRAMEBUFFER_SURFACE
                .lock()
                .as_mut()
                .unwrap()
                .rgba
                .cast::<u8>(),
            back.width * back.height * 4,
        );
    }
}

/// Returns an index from a position, using the global framebuffer surface's size, and `None` if out of bounds.
#[must_use]
#[inline]
pub fn get_pixel_index(x: usize, y: usize) -> Option<usize> {
    get_framebuffer_surface_ref().get_pixel_index(x, y)
}

/// Sets a pixel on the global framebuffer surface.
pub fn set_pixel(x: usize, y: usize, color: Color) {
    if let Some(index) = get_pixel_index(x, y) {
        unsafe { color.blit(index) };
    }
}

/// Returns the framebuffer width.
#[must_use]
pub fn get_framebuffer_width() -> usize {
    get_framebuffer_surface_ref().width
}

/// Returns the framebuffer height.
#[must_use]
pub fn get_framebuffer_height() -> usize {
    get_framebuffer_surface_ref().height
}

/// Returns the size of the framebuffer. (w*h*4)
#[must_use]
pub fn get_framebuffer_size() -> usize {
    get_framebuffer_width() * get_framebuffer_height() * 4
}

/// Clears the framebuffer with the following `color`.
pub fn clear_framebuffer(color: Color) {
    get_framebuffer_surface_mut().clear(color);
}

/// Clears a surface with the following `color`.
///
/// # Safety
/// This expects ptr to be a pointer to an RGBA buffer (check Surface's rgba)
pub unsafe fn clear_surface(ptr: RawFramebufferPointerMut, pixel_count: usize, color: Color) {
    let pixel_value = [color.b, color.g, color.r, color.a];
    let slice = unsafe { core::slice::from_raw_parts_mut(ptr, pixel_count) };

    slice.fill(pixel_value);
}

/// A surface to render on.
#[derive(Clone)]
pub struct Surface {
    /// Raw RGBA pixels.
    pub rgba: *mut [u8; 4],
    /// The width of the surface.
    pub width: usize,
    /// The height of the surface.
    pub height: usize,
}

// Sorry not sorry
unsafe impl Send for Surface {}
unsafe impl Sync for Surface {}

impl Surface {
    /// Creates a new surface with the dimensions of `width`x`height`, with `rgba` being in RGBA, and divisible by 4.
    #[must_use]
    pub const fn new(width: usize, height: usize, rgba: *mut [u8; 4]) -> Self {
        Self {
            rgba,
            width,
            height,
        }
    }

    /// Creates a new fully black surface with the dimensions of `width`x`height`.
    #[must_use]
    pub fn new_empty(width: usize, height: usize) -> Self {
        let pixel_count = width * height;
        let layout = Layout::array::<[u8; 4]>(pixel_count).expect("framebuffer layout overflow");
        let rgba = unsafe { alloc::alloc::alloc_zeroed(layout).cast::<[u8; 4]>() };

        Self {
            rgba,
            width,
            height,
        }
    }

    /// Clears the surface with the following `color`.
    pub fn clear(&mut self, color: Color) {
        unsafe {
            clear_surface(
                self.rgba as RawFramebufferPointerMut,
                self.width * self.height,
                color,
            );
        }
    }

    /// Sets a pixel to the following `color`.
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        if let Some(index) = self.get_pixel_index(x, y) {
            unsafe {
                color.blit_ex(self, index);
            }
        }
    }

    /// Returns the index of a position, using `surface`'s size, and `None` if out of bounds.
    #[must_use]
    pub fn get_pixel_index(&self, x: usize, y: usize) -> Option<usize> {
        if x >= self.width || y >= self.height {
            return None;
        }

        let row = y.checked_mul(self.width)?;
        row.checked_add(x)
    }
}
