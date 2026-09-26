//! Reusable pieces of the SeEx chrome, moved out of `main.rs` so both front ends
//! read the same metrics.
//!
//! Everything here mirrors the egui originals: the body/button face is
//! `scaled_font(26.0)` = 19.5px, buttons are 40px tall with 16×8 padding and a
//! 2px radius on `field_bg`, panels are square-cornered `surface` with a 1px
//! `border` and 18×16 padding, and rows are `row_bg` with a `subtle_border`.

use gpui_kit::*;

use crate::ui::theme;

const BODY: f32 = 20.0;
const SMALL: f32 = 14.0;

/// A filled button in the egui style: 40px tall, `field_bg`, 1px border.
pub fn button(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    enabled: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let element = div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .h(px(40.0))
        .px(px(16.0))
        .rounded(px(theme::RADIUS_SM))
        .bg(theme::field_bg())
        .border(px(1.0))
        .border_color(theme::border())
        .text_size(px(BODY))
        .text_color(theme::text_color())
        .cursor_pointer()
        .child(label.into());

    if enabled {
        element
            .hover(|style| style.bg(theme::widget_hover_bg()))
            .on_click(on_click)
    } else {
        element.opacity(0.5)
    }
}

/// A small button for list rows.
pub fn small_button(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .h(px(28.0))
        .px(px(10.0))
        .rounded(px(theme::RADIUS_SM))
        .bg(theme::field_bg())
        .border(px(1.0))
        .border_color(theme::border())
        .text_size(px(SMALL))
        .text_color(theme::text_color())
        .cursor_pointer()
        .hover(|style| style.bg(theme::widget_hover_bg()))
        .child(label.into())
        .on_click(on_click)
}

/// `selectable_label`: the primary colour fills the button while it is chosen.
pub fn selectable(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    selected: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let (background, text) = if selected {
        (theme::primary(), theme::white())
    } else {
        (theme::field_bg(), theme::text_color())
    };
    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .h(px(40.0))
        .px(px(16.0))
        .rounded(px(theme::RADIUS_SM))
        .bg(background)
        .border(px(1.0))
        .border_color(theme::border())
        .text_size(px(BODY))
        .text_color(text)
        .cursor_pointer()
        .hover(move |style| {
            if selected {
                style
            } else {
                style.bg(theme::widget_hover_bg())
            }
        })
        .child(label.into())
        .on_click(on_click)
}

/// A button that fills its share of a row, for the theme-preset grid.
pub fn theme_button(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    selected: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let (background, text) = if selected {
        (theme::primary(), theme::white())
    } else {
        (theme::field_bg(), theme::text_color())
    };
    div()
        .id(id)
        .flex()
        .flex_1()
        .items_center()
        .justify_center()
        .h(px(38.0))
        .rounded(px(theme::RADIUS_SM))
        .bg(background)
        .border(px(1.0))
        .border_color(theme::border())
        .text_size(px(18.0))
        .text_color(text)
        .cursor_pointer()
        .hover(move |style| {
            if selected {
                style
            } else {
                style.bg(theme::widget_hover_bg())
            }
        })
        .child(label.into())
        .on_click(on_click)
}

/// A top-bar button: `width` wide, 44px tall, primary-filled when selected.
pub fn top_bar_button(
    id: impl Into<ElementId>,
    width: f32,
    label: impl Into<SharedString>,
    selected: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let text_color = if selected {
        theme::white()
    } else {
        theme::text_color()
    };
    let background = if selected {
        theme::primary()
    } else {
        theme::top_bar_button_bg()
    };

    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .w(px(width))
        .h(px(44.0))
        .rounded(px(theme::RADIUS_MD))
        .bg(background)
        .hover(move |style| {
            if selected {
                style
            } else {
                style.bg(theme::top_bar_hover())
            }
        })
        .cursor_pointer()
        .text_size(px(theme::scaled_font(24.0)))
        .text_color(text_color)
        .child(label.into())
        .on_click(on_click)
}

/// A panel: `surface` fill, 1px border, 18×16 padding, 10px between children.
pub fn section(body: impl IntoElement) -> impl IntoElement {
    panel(0.0, body)
}

/// The same panel pinned to the height the neighbouring column settles at.
pub fn section_with_min_height(min_height: f32, body: impl IntoElement) -> impl IntoElement {
    panel(min_height, body)
}

fn panel(min_height: f32, body: impl IntoElement) -> impl IntoElement {
    let element = div()
        .flex()
        .flex_col()
        .w_full()
        .px(px(18.0))
        .py(px(16.0))
        .gap(px(10.0))
        .bg(theme::surface())
        .border(px(1.0))
        .border_color(theme::border())
        .rounded(px(theme::RADIUS_LG))
        .child(body);

    if min_height > 0.0 {
        // egui reserved an exact size here, so children that grow have a
        // definite box to grow into.
        element.h(px(min_height)).min_h(px(min_height))
    } else {
        element
    }
}

/// A list row: `row_bg`, 1px `subtle_border`, 12×8 padding.
pub fn row_frame(body: impl IntoElement) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .px(px(12.0))
        .py(px(8.0))
        .bg(theme::row_bg())
        .border(px(1.0))
        .border_color(theme::subtle_border())
        .rounded(px(theme::RADIUS_SM))
        .child(body)
}

/// A panel title with its one-line description underneath.
pub fn section_header(
    title: impl Into<SharedString>,
    desc: impl Into<SharedString>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(4.0))
        .child(
            div()
                .text_size(px(BODY))
                .text_color(theme::text_color())
                .child(title.into()),
        )
        .child(
            div()
                .text_size(px(SMALL))
                .text_color(theme::muted())
                .child(desc.into()),
        )
}

/// A title with room for a control on the right-hand edge.
pub fn section_header_inline(
    title: impl Into<SharedString>,
    right: impl IntoElement,
) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .child(
            div()
                .text_size(px(BODY))
                .text_color(theme::text_color())
                .child(title.into()),
        )
        .child(right)
}

/// The 180×30 label column that lines up the fields in a form.
pub fn fixed_label(label: impl Into<SharedString>) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .w(px(180.0))
        .h(px(30.0))
        .text_size(px(BODY))
        .text_color(theme::muted())
        .child(label.into())
}

pub fn empty_state(label: impl Into<SharedString>) -> impl IntoElement {
    div()
        .mt(px(8.0))
        .text_size(px(BODY))
        .text_color(theme::muted())
        .italic()
        .child(label.into())
}

/// A rounded counter, sized from its text like the painted egui chip.
pub fn chip(label: impl Into<SharedString>, color: Rgba) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .px(px(9.0))
        .py(px(4.0))
        .rounded(px(12.0))
        .bg(theme::chip_bg(color))
        .text_size(px(theme::scaled_font(20.0)))
        .text_color(color)
        .child(label.into())
}

pub fn count_chip(value: usize, color: Rgba) -> impl IntoElement {
    chip(value.to_string(), color)
}

/// One half of the Font page: the slot's name, the family in use, and the
/// button that opens its picker.
pub fn font_slot_row(
    id: &'static str,
    label: impl Into<SharedString>,
    family: impl Into<SharedString>,
    open: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let marker = if open { "\u{25B2}" } else { "\u{25BC}" };
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .gap(px(12.0))
        .child(
            div()
                .w(px(64.0))
                .text_size(px(BODY))
                .text_color(theme::muted())
                .child(label.into()),
        )
        .child(
            div()
                .flex_1()
                .overflow_hidden()
                .text_ellipsis()
                .text_size(px(BODY))
                .text_color(theme::text_color())
                .child(family.into()),
        )
        .child(selectable(
            format!("{id}-toggle"),
            marker,
            open,
            on_click,
        ))
}

/// Attaches a hover hint. Rather than a floating tooltip — which landed on top
/// of the buttons around it — the text is parked on the view and shown in the
/// status bar, where it cannot cover anything.
pub fn hint(
    element: Stateful<Div>,
    text: impl Into<SharedString>,
    area: crate::ui::app::HintArea,
    cx: &mut Context<crate::ui::app::SeexUi>,
) -> Stateful<Div> {
    let text: SharedString = text.into();
    element.on_hover(cx.listener(move |this, hovered: &bool, _window, cx| {
        if *hovered {
            this.hover_hint = Some((area, text.clone()));
        } else if matches!(&this.hover_hint, Some((_, current)) if current == &text) {
            // Only clear our own hint: the pointer may already be on the next
            // button, which has set its own by now.
            this.hover_hint = None;
        }
        cx.notify();
    }))
}

/// A single family in the picker list.
pub fn font_row(
    id: impl Into<ElementId>,
    name: impl Into<SharedString>,
    selected: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let name: SharedString = name.into();
    let text = if selected {
        theme::white()
    } else {
        theme::text_color()
    };
    div()
        .id(id)
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px(px(12.0))
        .py(px(6.0))
        .bg(if selected {
            theme::primary()
        } else {
            theme::row_bg()
        })
        .border(px(1.0))
        .border_color(theme::subtle_border())
        .rounded(px(theme::RADIUS_SM))
        .cursor_pointer()
        .hover(move |style| {
            if selected {
                style
            } else {
                style.bg(theme::widget_hover_bg())
            }
        })
        .child(
            div()
                .flex_1()
                .overflow_hidden()
                .text_ellipsis()
                .text_size(px(BODY))
                .text_color(text)
                .child(name.clone()),
        )
        .child(
            div()
                .flex_shrink_0()
                .font_family(name)
                .text_size(px(BODY))
                .text_color(text)
                .child("Aa 汉字"),
        )
        .on_click(on_click)
}

/// The single style for everything the status bar has to say — a running notice
/// or a hover hint — so the two read as the same kind of thing: a filled box in
/// the accent colour, with white text.
pub fn status_box(text: impl Into<SharedString>, accent: Rgba) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .h(px(34.0))
        .px(px(12.0))
        .rounded(px(6.0))
        .bg(accent)
        .border(px(1.0))
        .border_color(accent)
        .overflow_hidden()
        .text_ellipsis()
        .text_size(px(theme::scaled_font(23.0)))
        .text_color(theme::white())
        .child(text.into())
}

/// A wide, count-bearing pill that heads an action row.
pub fn metric_pill(label: impl Into<SharedString>, value: usize, color: Rgba) -> impl IntoElement {
    let label: SharedString = label.into();
    div()
        .flex()
        .items_center()
        .px(px(10.0))
        .py(px(5.0))
        .rounded(px(12.0))
        .bg(theme::chip_bg(color))
        .text_size(px(theme::scaled_font(25.0)))
        .text_color(color)
        .child(format!("{label} {value}"))
}

/// The same pill with its text already assembled, for the export metrics.
pub fn pill_text(text: impl Into<SharedString>, color: Rgba) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .px(px(10.0))
        .py(px(5.0))
        .rounded(px(12.0))
        .bg(theme::chip_bg(color))
        .text_size(px(theme::scaled_font(25.0)))
        .text_color(color)
        .child(text.into())
}

/// A progress bar in the theme's primary colour, with its message written
/// inside the track the way egui drew it.
pub fn progress_bar(value: f32, message: impl Into<SharedString>) -> impl IntoElement {
    let value = value.clamp(0.0, 1.0);
    let message: SharedString = message.into();
    div()
        .relative()
        .flex_1()
        .min_w(px(200.0))
        .h(px(34.0))
        .rounded(px(theme::RADIUS_SM))
        .bg(theme::field_bg())
        .border(px(1.0))
        .border_color(theme::border())
        .overflow_hidden()
        .child(
            div()
                .absolute()
                .left_0()
                .top_0()
                .h_full()
                .w(relative(value))
                .bg(theme::progress_fill()),
        )
        .child(
            div()
                .absolute()
                .left_0()
                .top_0()
                .w_full()
                .h_full()
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(theme::scaled_font(22.0)))
                .text_color(theme::text_color())
                .child(message),
        )
}

/// `color_setting`: the 180px label column, a round swatch, and the `#RRGGBB`
/// box that keeps the swatch in step with what is typed.
pub fn color_setting(
    label: impl Into<SharedString>,
    color: [u8; 3],
    input: &Entity<gpui_kit::component::input::InputState>,
) -> impl IntoElement {
    let swatch = gpui_kit::rgb(
        ((color[0] as u32) << 16) | ((color[1] as u32) << 8) | (color[2] as u32),
    );
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(12.0))
        .child(fixed_label(label))
        .child(
            div()
                .w(px(26.0))
                .h(px(26.0))
                .rounded_full()
                .bg(swatch)
                .border(px(1.0))
                .border_color(theme::border()),
        )
        .child(
            div()
                .w(px(100.0))
                .child(gpui_kit::component::input::Input::new(input).h(px(28.0))),
        )
}
