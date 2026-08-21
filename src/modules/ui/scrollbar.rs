//! Scrollbar: the single source of truth for scrollbar design in Silo.
//!
//! Every scrollable area in the app uses this module's scrollbar so the look
//! stays consistent. The module provides the scrollbar configuration ([`bar`]),
//! the appearance function ([`style`]), and a wrapper that turns any content
//! into a vertical scrollable with the app scrollbar applied ([`vertical`]).

use iced::widget::container;
use iced::widget::scrollable::{self, Scrollable, Status};
use iced::{Border, Color, Element, Length, Shadow, Theme};

use crate::modules::ui::scaling::sp;
use crate::modules::ui::theme::{BACK, DETAIL, ORANGE, TEAL};

/// The width of the scrollbar track, in reference pixels.
const SCROLLBAR_WIDTH: f32 = 8.0;

/// The width of the draggable scroller, in reference pixels.
const SCROLLER_WIDTH: f32 = 8.0;

/// The margin around the scrollbar, in reference pixels.
const SCROLLBAR_MARGIN: f32 = 2.0;

/// The gap between the content and the scrollbar, in reference pixels. This
/// also makes the scrollable reserve the space, so the content never passes
/// under the scrollbar.
const SCROLLBAR_SPACING: f32 = 8.0;

/// The app-wide scrollbar configuration.
///
/// With a spacing set, the scrollable shrinks its content on the right by the
/// track width plus both margins plus the spacing whenever the scrollbar is
/// visible, so the content ends before the scrollbar.
pub fn bar() -> scrollable::Scrollbar {
    scrollable::Scrollbar::new()
        .width(sp(SCROLLBAR_WIDTH))
        .scroller_width(sp(SCROLLER_WIDTH))
        .margin(sp(SCROLLBAR_MARGIN))
        .spacing(sp(SCROLLBAR_SPACING))
}

/// The app-wide scrollbar appearance.
///
/// The track is invisible. The scroller is DETAIL by default, TEAL while
/// hovered, and ORANGE while dragged.
pub fn style(_theme: &Theme, status: Status) -> scrollable::Style {
    let scroller_color = match status {
        Status::Dragged { .. } => ORANGE,
        Status::Hovered { .. } => TEAL,
        Status::Active { .. } => DETAIL,
    };

    let rail = scrollable::Rail {
        background: None,
        border: Border::default(),
        scroller: scrollable::Scroller {
            background: scroller_color.into(),
            border: Border::default(),
        },
    };

    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: rail,
        horizontal_rail: rail,
        gap: None,
        auto_scroll: scrollable::AutoScroll {
            background: Color { a: 0.60, ..BACK }.into(),
            border: Border {
                color: DETAIL,
                width: 1.0,
                radius: 0.0.into(),
            },
            shadow: Shadow::default(),
            icon: TEAL,
        },
    }
}

/// Wraps content in a full-size vertical scrollable with the app scrollbar.
pub fn vertical<'a, Message>(content: impl Into<Element<'a, Message>>) -> Scrollable<'a, Message> {
    Scrollable::new(content)
        .direction(scrollable::Direction::Vertical(bar()))
        .style(style)
        .width(Length::Fill)
        .height(Length::Fill)
}
