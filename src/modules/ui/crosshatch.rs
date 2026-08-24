//! Crosshatch: the diagonal line pattern behind the dialog boxes.
//!
//! This module draws evenly spaced straight lines across the full surface.
//! Every line runs at -45 degrees. The dialog views place this overlay between
//! the dimmed backdrop and the dialog box. The pattern shows between the main
//! window and the dialog. The overlay is transparent outside the lines and
//! non-interactive, so it never intercepts input.

use iced::mouse;
use iced::widget::canvas::{self, Path, Program, Stroke};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Theme};

use crate::modules::ui::scaling::sp;
use crate::modules::ui::theme::GREY;

/// The perpendicular distance between parallel hatch lines, in reference px.
const HATCH_SPACING: f32 = 10.0;

/// The thickness of a hatch line, in reference pixels.
const HATCH_THICKNESS: f32 = 1.5;

/// The opacity of the hatch lines, kept low for a subtle effect.
const HATCH_ALPHA: f32 = 0.20;

/// The crosshatch overlay widget.
struct Crosshatch;

/// Builds the full-window crosshatch overlay element.
pub fn overlay<Message: 'static>() -> Element<'static, Message> {
    canvas::Canvas::new(Crosshatch)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

impl<Message> Program<Message> for Crosshatch {
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

        let spacing = sp(HATCH_SPACING);
        let thickness = sp(HATCH_THICKNESS);

        // The step between parallel lines along the intercept axis. A
        // perpendicular distance of `spacing` becomes `spacing * sqrt(2)` at a
        // 45-degree slope.
        let step = spacing * 2.0_f32.sqrt();

        // Half-length of every drawn line, long enough to cross the full
        // surface from any intercept. The canvas clips the excess.
        let reach = bounds.width + bounds.height;

        let stroke = Stroke {
            width: thickness,
            style: canvas::Style::Solid(Color {
                a: HATCH_ALPHA,
                ..GREY
            }),
            ..Stroke::default()
        };

        // The -45 degree family: direction (1, -1), lines satisfy x + y = c.
        let mut c = 0.0_f32;
        while c <= reach {
            let path = Path::line(Point::new(c - reach, reach), Point::new(c + reach, -reach));
            frame.stroke(&path, stroke.clone());
            c += step;
        }

        vec![frame.into_geometry()]
    }
}
