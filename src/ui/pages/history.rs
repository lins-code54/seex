//! The History page: what the clipboard monitor has seen, newest first.
//!
//! Ported from `SeexApp::history_page` in `src/main.rs`: the same panel, the
//! same `list_height(ui, 0.55, 6, 52.0)` list height against a 900px window, and
//! the same per-row Copy/Delete pair.

use gpui_kit::*;

use crate::ui::app::{SeexUi, copy_to_clipboard, first_line};
use crate::ui::{components, locale, theme};

const HISTORY_LIST_HEIGHT: f32 = 495.0;
const BODY: f32 = 20.0;

impl SeexUi {
    pub(crate) fn history_page(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let history = self
            .state
            .lock()
            .map(|state| state.history.clone())
            .unwrap_or_default();

        let mut body = div()
            .flex()
            .flex_col()
            .w_full()
            .gap(px(10.0))
            .child(components::section_header_inline(
                locale::t("History", "历史"),
                components::count_chip(history.len(), theme::primary()),
            ));

        if history.is_empty() {
            body = body.child(components::empty_state(locale::t(
                "No history yet",
                "暂无历史记录",
            )));
        } else {
            let mut list = div()
                .id("history-list")
                .flex()
                .flex_col()
                .w_full()
                .h(px(HISTORY_LIST_HEIGHT))
                .overflow_y_scroll();

            for (index, (time, content)) in history.iter().enumerate() {
                let preview = first_line(content, 110);
                let mono = self.mono_family.clone();

                list = list.child(
                    div().mb(px(6.0)).child(components::row_frame(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .w_full()
                            .gap(px(12.0))
                            .child(
                                div()
                                    .flex_shrink_0()
                                    .font_family(mono)
                                    .text_size(px(BODY))
                                    .text_color(theme::muted())
                                    .child(time.clone()),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .overflow_hidden()
                                    .text_ellipsis()
                                    .text_size(px(BODY))
                                    .text_color(theme::text_color())
                                    .child(preview),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_shrink_0()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(8.0))
                                    .child(components::small_button(
                                        format!("copy-history-{index}"),
                                        locale::t("Copy", "复制"),
                                        {
                                            let copied = content.clone();
                                            cx.listener(move |this, _event: &ClickEvent, _window, cx| {
                                                copy_to_clipboard(&copied);
                                                this.show_toast_success(locale::t("Copied", "已复制"));
                                                cx.notify();
                                            })
                                        },
                                    ))
                                    .child(components::small_button(
                                        format!("delete-history-{index}"),
                                        locale::t("Delete", "删除"),
                                        cx.listener(move |this, _event: &ClickEvent, _window, cx| {
                                            if let Ok(mut state) = this.state.lock() {
                                                state.delete_history(index);
                                            }
                                            cx.notify();
                                        }),
                                    )),
                            ),
                    )),
                );
            }
            body = body.child(list);
        }

        components::section(body).into_any_element()
    }
}
