//! Scanlines: a subtle CRT-scanline overlay for the whole window.
//!
//! This module draws thin, evenly spaced horizontal lines across the entire
//! surface to give the window a retro screen look. The lines are near-black at
//! low alpha, so they stay subtle over the dark background but show clearly
//! over brighter content, just like real CRT scanlines. The overlay is
//! non-interactive, so it never intercepts input from widgets beneath it.

use iced::mouse;
use iced::widget::canvas::{self, Program};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Size, Theme};

use crate::modules::ui::scaling::sp;

/// The vertical distance between scanlines, in pixels.
const SCANLINE_SPACING: f32 = 3.0;

/// The thickness of a scanline, in pixels.
const SCANLINE_THICKNESS: f32 = 1.0;

/// The opacity of the scanlines, kept low for a subtle effect.
const SCANLINE_ALPHA: f32 = 0.22;

/// The scanlines overlay widget.
struct Scanlines;

/// Builds the full-window scanline overlay element.
pub fn overlay<Message: 'static>() -> Element<'static, Message> {
    canvas::Canvas::new(Scanlines)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

impl<Message> Program<Message> for Scanlines {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::with_bounds(renderer, bounds);

        let spacing = sp(SCANLINE_SPACING);
        let thickness = sp(SCANLINE_THICKNESS);

        let fill = canvas::Fill {
            style: canvas::Style::Solid(Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: SCANLINE_ALPHA,
            }),
            ..canvas::Fill::default()
        };

        let mut y = 0.0_f32;
        while y < bounds.height {
            frame.fill_rectangle(Point::new(0.0, y), Size::new(bounds.width, thickness), fill);
            y += spacing;
        }

        vec![frame.into_geometry()]
    }
}
