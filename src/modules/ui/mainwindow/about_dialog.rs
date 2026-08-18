//! AboutDialog: the About Silo overlay dialog.
//!
//! Shown when the silo logo in the action area is pressed. A centered dialog
//! box sits on a dimmed full-window backdrop. Pressing the backdrop or the
//! CLOSE button closes the dialog.

use iced::mouse;
use iced::widget::{Column, MouseArea, Stack, container, svg, text};
use iced::{Border, Color, Element, Length, Padding, Shadow, Vector};

use crate::modules::ui::scaling::sp;
use crate::modules::ui::theme::{BACK, DETAIL, GREY, TEAL};

use super::Message;

/// The embedded banner image, compiled into the binary at build time.
/// Rendered as an SVG so it is available on the first frame.
const BANNER_BYTES: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/logo/banner.svg"));

/// The embedded GitHub logo, compiled into the binary at build time.
const GITHUB_BYTES: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/logo/GitHub_Invertocat_White_Clearspace.svg"));

/// The project URL opened when the GitHub logo is pressed.
pub(super) const GITHUB_URL: &str = "https://github.com/DOCKPORT/Silo";

/// The width of the dialog box, in reference pixels.
const DIALOG_WIDTH: f32 = 620.0;

/// The width of the banner image in the dialog, in reference pixels.
/// The height follows the image aspect ratio.
const BANNER_WIDTH: f32 = 540.0;

/// The rendered size of the square GitHub logo, in reference pixels.
const GITHUB_SIZE: f32 = 40.0;

/// The font size of the version text, in reference pixels.
const VERSION_SIZE: f32 = 26.0;

/// The font size of the body text, in reference pixels.
const TEXT_SIZE: f32 = 22.0;

/// The internal padding of the dialog box, in reference pixels.
const DIALOG_PAD: f32 = 40.0;

/// The height of the CLOSE button, in reference pixels.
const CLOSE_HEIGHT: f32 = 44.0;

/// The horizontal padding of the CLOSE button, in reference pixels.
const CLOSE_PAD_H: f32 = 28.0;

/// The width of the dialog border, in reference pixels.
const BORDER_WIDTH: f32 = 2.0;

/// The alpha of the dimmed backdrop behind the dialog box.
const BACKDROP_ALPHA: f32 = 0.85;

/// Builds the About dialog overlay.
///
/// Returns a full-window overlay: a dimmed backdrop that closes the dialog on
/// press, and a centered dialog box with the banner, version, and a
/// CLOSE button.
pub fn view() -> Element<'static, Message> {
    // The dimmed backdrop. Pressing it closes the dialog.
    let backdrop = MouseArea::new(
        container(text(""))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color { a: BACKDROP_ALPHA, ..BACK }.into()),
                ..container::Style::default()
            }),
    )
    .on_press(Message::CloseAboutDialog);

    // The dialog box content.
    let content = Column::new()
        .spacing(sp(16.0))
        .align_x(iced::alignment::Horizontal::Center)
        .push(
            svg::Svg::new(svg::Handle::from_memory(BANNER_BYTES))
                .width(Length::Fixed(sp(BANNER_WIDTH))),
        )
        .push(
            text(format!("VERSION {}", env!("CARGO_PKG_VERSION")))
                .size(sp(VERSION_SIZE))
                .color(TEAL),
        )
        .push(
            MouseArea::new(
                svg::Svg::new(svg::Handle::from_memory(GITHUB_BYTES))
                    .width(Length::Fixed(sp(GITHUB_SIZE)))
                    .height(Length::Fixed(sp(GITHUB_SIZE))),
            )
            .on_press(Message::OpenGithub)
            .interaction(mouse::Interaction::Pointer),
        )
        .push(text("Silo is an rsync GUI application. It lets you define a body of data — a \"silo\" — by selecting & excluding folders from source, then mirror that silo to a destination with rsync.").size(sp(TEXT_SIZE)).color(GREY))
        .push(close_button());

    let dialog_box = container(content)
        .width(Length::Fixed(sp(DIALOG_WIDTH)))
        .padding(sp(DIALOG_PAD))
        .style(|_| container::Style {
            background: Some(BACK.into()),
            border: Border {
                color: DETAIL,
                width: sp(BORDER_WIDTH),
                radius: 0.0.into(),
            },
            shadow: Shadow {
                color: Color { a: 0.5, ..DETAIL },
                offset: Vector::ZERO,
                blur_radius: sp(12.0),
            },
            ..container::Style::default()
        });

    // Absorbs clicks inside the box so they do not reach the backdrop. Only
    // the backdrop and the CLOSE button close the dialog.
    let box_area = MouseArea::new(dialog_box).on_press(Message::NoOp);

    Stack::new()
        .push(backdrop)
        .push(
            container(box_area)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
        )
        .into()
}

/// Builds the CLOSE button that closes the dialog.
fn close_button() -> Element<'static, Message> {
    let label = container(text("CLOSE").size(sp(TEXT_SIZE)).color(TEAL))
        .height(Length::Fixed(sp(CLOSE_HEIGHT)))
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .padding(Padding {
            left: sp(CLOSE_PAD_H),
            right: sp(CLOSE_PAD_H),
            top: 0.0,
            bottom: 0.0,
        })
        .style(|_| container::Style {
            background: None,
            border: Border {
                color: DETAIL,
                width: sp(BORDER_WIDTH),
                radius: 0.0.into(),
            },
            ..container::Style::default()
        });

    MouseArea::new(label)
        .on_press(Message::CloseAboutDialog)
        .interaction(mouse::Interaction::Pointer)
        .into()
}
