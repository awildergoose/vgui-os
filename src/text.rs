//! Used to do font/text rendering.
use crate::{
    color::Color,
    font::FONT,
    framebuffer::{Surface, get_framebuffer_surface_mut, get_framebuffer_width},
};

/// Draws a single character onto the global framebuffer surface.
pub fn draw_char(x: usize, y: usize, c: u8, color: Color) {
    draw_char_ex(get_framebuffer_surface_mut(), x, y, c, color);
}

/// Draws a single character onto a surface.
pub fn draw_char_ex(surface: &mut Surface, x: usize, y: usize, c: u8, color: Color) {
    for (row, &bits) in FONT[c as usize].iter().take(8).enumerate() {
        for col in 0..8 {
            if bits & (1 << (7 - col)) != 0 {
                surface.set_pixel(x + col, y + row, color);
            }
        }
    }
}

/// Draws text that wraps around a surface.
pub fn draw_text_wrapped_ex<S: AsRef<str>>(
    surface: &mut Surface,
    x: usize,
    mut y: usize,
    text: S,
    color: Color,
    max_width: Option<usize>,
) {
    let text = text.as_ref();
    let mut cx = x;

    for ch in text.bytes() {
        if ch == b'\n' {
            y += 8;
            cx = x;
        } else {
            if let Some(mw) = max_width
                && cx + 8 > x + mw
            {
                cx = x;
                y += 8;
            }

            draw_char_ex(surface, cx, y, ch, color);
            cx += 8;
        }
    }
}

/// Draws text onto the global framebuffer surface.
pub fn draw_text<S: AsRef<str>>(x: usize, y: usize, text: S, color: Color) {
    draw_text_wrapped_ex(
        get_framebuffer_surface_mut(),
        x,
        y,
        text.as_ref(),
        color,
        None,
    );
}

/// Draws wrapped text onto the global framebuffer surface.
pub fn draw_text_wrapped<S: AsRef<str>>(x: usize, y: usize, text: S, color: Color) {
    draw_text_wrapped_ex(
        get_framebuffer_surface_mut(),
        x,
        y,
        text.as_ref(),
        color,
        Some(get_framebuffer_width()),
    );
}

/// Returns a color from its' name.
#[must_use]
pub fn color_from_name(name: &str) -> Option<Color> {
    match name.to_ascii_lowercase().as_str() {
        "black" => Some(Color::BLACK),
        "white" => Some(Color::WHITE),
        "red" => Some(Color::RED),
        "green" => Some(Color::GREEN),
        "blue" => Some(Color::BLUE),
        "yellow" => Some(Color::YELLOW),
        "cyan" => Some(Color::CYAN),
        "magenta" => Some(Color::MAGENTA),
        "orange" => Some(Color::ORANGE),
        "purple" => Some(Color::PURPLE),
        "pink" => Some(Color::PINK),
        "brown" => Some(Color::BROWN),
        "gray" => Some(Color::GRAY),
        "light_gray" | "lightgray" => Some(Color::LIGHT_GRAY),
        "dark_gray" | "darkgray" => Some(Color::DARK_GRAY),
        _ => None,
    }
}

/// Draws formatted text onto the global framebuffer surface.
pub fn draw_text_formatted<S: AsRef<str>>(x: usize, y: usize, text: S, default_color: Color) {
    draw_text_formatted_ex(
        get_framebuffer_surface_mut(),
        x,
        y,
        text.as_ref(),
        default_color,
        None,
    );
}

/// Draws formatted wrapped text onto the global framebuffer surface.
pub fn draw_text_formatted_wrapped<S: AsRef<str>>(
    x: usize,
    y: usize,
    text: S,
    default_color: Color,
) {
    let max = get_framebuffer_width();
    draw_text_formatted_ex(
        get_framebuffer_surface_mut(),
        x,
        y,
        text.as_ref(),
        default_color,
        Some(max),
    );
}

/// Draws formatted text onto the surface.
pub fn draw_text_formatted_ex(
    surface: &mut Surface,
    x: usize,
    mut y: usize,
    text: &str,
    default_color: Color,
    max_width: Option<usize>,
) {
    let bytes = text.as_bytes();
    let mut cx = x;
    let mut color = default_color;
    let mut i = 0usize;

    while i < bytes.len() {
        let b = bytes[i];

        if b == b'[' {
            if i + 1 < bytes.len() && bytes[i + 1] == b'[' {
                if let Some(mw) = max_width
                    && cx + 8 > x + mw
                {
                    cx = x;
                    y += 8;
                }
                draw_char(cx, y, b'[', color);
                cx += 8;
                i += 2;
                continue;
            }

            if let Some(rel_pos) = bytes[i + 1..].iter().position(|&c| c == b']') {
                let name_bytes = &bytes[i + 1..i + 1 + rel_pos];
                if let Ok(name_str) = core::str::from_utf8(name_bytes)
                    && let Some(col) = color_from_name(name_str)
                {
                    color = col;
                    i += 1 + rel_pos + 1;
                    continue;
                }
            }
        }

        if b == b'\n' {
            y += 8;
            cx = x;
        } else {
            if let Some(mw) = max_width
                && cx + 8 > x + mw
            {
                cx = x;
                y += 8;
            }
            draw_char_ex(surface, cx, y, b, color);
            cx += 8;
        }
        i += 1;
    }
}

/// Returns the width of the following text.
pub fn get_text_width<S: AsRef<str>>(text: S) -> usize {
    let text = text.as_ref();
    let mut max_width = 0;
    let mut current_width = 0;

    for ch in text.bytes() {
        match ch {
            b'\n' => {
                max_width = max_width.max(current_width);
                current_width = 0;
            }
            _ => {
                current_width += 8;
            }
        }
    }

    max_width.max(current_width)
}

/// Returns the height of the following text.
pub fn get_text_height<S: AsRef<str>>(text: S) -> usize {
    let text = text.as_ref();
    let mut lines = 1;

    for ch in text.bytes() {
        if ch == b'\n' {
            lines += 1;
        }
    }

    lines * 8
}

/// Returns the width of the following formatted text.
pub fn get_formatted_text_width<S: AsRef<str>>(text: S) -> usize {
    let text = text.as_ref();
    let bytes = text.as_bytes();
    let mut max_width = 0usize;
    let mut current_width = 0usize;
    let mut i = 0usize;

    while i < bytes.len() {
        let b = bytes[i];
        if b == b'[' {
            if i + 1 < bytes.len() && bytes[i + 1] == b'[' {
                current_width += 8;
                i += 2;
                continue;
            }
            if let Some(rel_pos) = bytes[i + 1..].iter().position(|&c| c == b']') {
                i += 1 + rel_pos + 1;
                continue;
            }
        }

        if b == b'\n' {
            max_width = core::cmp::max(max_width, current_width);
            current_width = 0;
        } else {
            current_width += 8;
        }
        i += 1;
    }

    core::cmp::max(max_width, current_width)
}

/// Returns the height of the following formatted text.
pub fn get_formatted_text_height<S: AsRef<str>>(text: S) -> usize {
    get_text_height(text)
}
