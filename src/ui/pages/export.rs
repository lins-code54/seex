//! The Export page: the export run at the top, the settings that feed it
//! underneath.
//!
//! Ported from `SeexApp::export_page` in `src/main.rs`. The two panels, the
//! 180px label column, the scope buttons, and the toggle row are the same; the
//! failed-ID list, which used to pop up over the page, is now shown inside the
//! export panel where it cannot cover anything.

use gpui_kit::component::input::Input;
use gpui_kit::*;

use crate::ui::app::{FileDialogKind, InputSlot, SeexUi};
use crate::ui::locale::Lang;
use crate::ui::{components, locale, theme};

const BODY: f32 = 20.0;

/// The five numbers `render_export_metrics` scraped out of an export result.
fn metric_values(result: &str) -> Vec<(&'static str, String)> {
    let mut metrics: Vec<(&'static str, String)> = Vec::new();
    for line in result.lines().filter(|line| !line.trim().is_empty()) {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("Elapsed:") {
            metrics.push(("Elapsed", value.trim().to_string()));
        } else if trimmed.contains("Total:") {
            for key in ["Total", "Success", "Skipped", "Failed"] {
                if let Some((_, value)) = trimmed.split_once(&format!("{key}:")) {
                    metrics.push((
                        key,
                        value.split('|').next().unwrap_or(value).trim().to_string(),
                    ));
                }
            }
        }
    }
    metrics
}

fn metric_labels() -> [&'static str; 5] {
    match locale::lang() {
        Lang::En => ["Elapsed", "Total", "Success", "Skipped", "Failed"],
        Lang::Zh => ["耗时", "总数", "成功", "跳过", "失败"],
    }
}

fn failed_ids(result: &str) -> Vec<String> {
    result
        .lines()
        .find(|line| line.trim().to_ascii_lowercase().starts_with("failed ids:"))
        .map(|line| crate::extract::extract_lcsc_ids(line))
        .unwrap_or_default()
}

impl SeexUi {
    pub(crate) fn export_page(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .flex()
            .flex_col()
            .w_full()
            .child(self.export_run_panel(cx))
            .child(div().h(px(10.0)))
            .child(self.export_settings_panel(window, cx))
            .into_any_element()
    }

    fn export_run_panel(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let (matched_count, running, result, progress) = self
            .state
            .lock()
            .map(|state| {
                (
                    state.matched.len(),
                    state.export_running,
                    state.export_last_result.clone(),
                    state
                        .export_progress
                        .as_ref()
                        .map(|progress| (progress.current, progress.total, progress.message.clone())),
                )
            })
            .unwrap_or((0, false, None, None));

        let mut body = div()
            .flex()
            .flex_col()
            .w_full()
            .gap(px(10.0))
            .child(components::section_header_inline(
                locale::t("EDA Library Export", "EDA 库导出"),
                components::metric_pill(
                    locale::t("Ready", "待导出"),
                    matched_count,
                    theme::primary(),
                ),
            ))
            .child(
                div()
                    .text_size(px(BODY))
                    .text_color(theme::muted())
                    .child(locale::t(
                        "Altium and KiCad export through export",
                        "通过 export 导出 Altium 和 KiCad 库",
                    )),
            );

        let mut row = div()
            .flex()
            .flex_row()
            .flex_wrap()
            .items_center()
            .w_full()
            .gap(px(12.0));

        if let Some(result) = &result {
            let metrics = metric_values(result);
            for (index, label) in metric_labels().iter().enumerate() {
                let key = ["Elapsed", "Total", "Success", "Skipped", "Failed"][index];
                let value = metrics
                    .iter()
                    .find(|(name, _)| *name == key)
                    .map(|(_, value)| value.clone())
                    .unwrap_or_else(|| "0".to_string());
                let color = if key == "Elapsed" || key == "Success" {
                    theme::success()
                } else if key == "Failed" && value != "0" {
                    theme::danger()
                } else {
                    theme::text_color()
                };
                let clickable = key == "Failed" && value != "0";
                let text = format!("{label} {value}");
                if clickable {
                    row = row.child(
                        div()
                            .id("failed-ids-pill")
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _event: &ClickEvent, _window, cx| {
                                this.show_failed_ids = !this.show_failed_ids;
                                cx.notify();
                            }))
                            .child(components::pill_text(text, color)),
                    );
                } else {
                    row = row.child(components::pill_text(text, color));
                }
            }

            if let Some((current, total, message)) = &progress {
                let value = if *total > 0 {
                    *current as f32 / *total as f32
                } else {
                    0.0
                };
                row = row.child(components::progress_bar(value, message.clone()));
            }

            row = row.child(div().flex_1()).child(components::button(
                "export-library",
                if running {
                    locale::t("Running...", "运行中...")
                } else {
                    locale::t("Export Library", "导出库")
                },
                matched_count > 0 && !running,
                cx.listener(|this, _event: &ClickEvent, _window, cx| {
                    this.start_export();
                    cx.notify();
                }),
            ));
        } else {
            if let Some((current, total, message)) = &progress {
                let value = if *total > 0 {
                    *current as f32 / *total as f32
                } else {
                    0.0
                };
                row = row.child(components::progress_bar(value, message.clone()));
            }
            row = row.child(div().flex_1()).child(components::button(
                "export-library",
                if running {
                    locale::t("Running...", "运行中...")
                } else {
                    locale::t("Export Library", "导出库")
                },
                matched_count > 0 && !running,
                cx.listener(|this, _event: &ClickEvent, _window, cx| {
                    this.start_export();
                    cx.notify();
                }),
            ));
        }

        body = body.child(row);

        if self.show_failed_ids {
            if let Some(result) = &result {
                let ids = failed_ids(result);
                if !ids.is_empty() {
                    body = body.child(
                        div()
                            .w_full()
                            .p(px(10.0))
                            .rounded(px(4.0))
                            .bg(theme::chip_bg(theme::danger()))
                            .text_size(px(BODY))
                            .font_family(self.mono_family.clone())
                            .text_color(theme::danger())
                            .child(ids.join(", ")),
                    );
                }
            }
        }

        components::section(body).into_any_element()
    }

    /// Builds the request the egui page built inline and hands it to the same
    /// exporter task.
    fn start_export(&mut self) {
        let request = {
            let Ok(state) = self.state.lock() else {
                return;
            };
            crate::exporter::ExportRequest {
                ids: state.get_unique_ids(),
                target: state.export_target.clone(),
                output_path: state.export_output_path.clone(),
                mode: if state.export_target == "kicad" {
                    state.export_kicad_mode.clone()
                } else {
                    state.export_mode.clone()
                },
                merge: state.export_merge,
                append: state.export_append,
                library_name: state.export_library_name.clone(),
                parallel: state.export_parallel,
                continue_on_error: state.export_continue_on_error,
                lcsc_english: state.export_lcsc_english,
                force: state.export_force,
                schematic_stroke_color: self.schematic_stroke_color,
                schematic_fill_color: self.schematic_fill_color,
            }
        };
        crate::exporter::spawn_export(std::sync::Arc::clone(&self.state), request);
    }

    fn export_settings_panel(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let (target, mode, kicad_mode, merge, append, continue_on_error, force, english) = self
            .state
            .lock()
            .map(|state| {
                (
                    state.export_target.clone(),
                    state.export_mode.clone(),
                    state.export_kicad_mode.clone(),
                    state.export_merge,
                    state.export_append,
                    state.export_continue_on_error,
                    state.export_force,
                    state.export_lcsc_english,
                )
            })
            .unwrap_or_default();

        let path_input = self.input(InputSlot::ExportPath, window, cx);
        let library_input = self.input(InputSlot::LibraryName, window, cx);
        let parallel_input = self.input(InputSlot::Parallel, window, cx);
        let stroke_input = self.input(InputSlot::StrokeHex, window, cx);
        let fill_input = self.input(InputSlot::FillHex, window, cx);

        let target_row = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(12.0))
            .child(components::fixed_label(locale::t("EDA target:", "EDA 目标:")))
            .child(components::selectable(
                "target-altium",
                "Altium",
                target != "kicad",
                cx.listener(|this, _event: &ClickEvent, _window, cx| {
                    if let Ok(mut state) = this.state.lock() {
                        state.set_export_target("altium".to_string());
                    }
                    this.active_color_picker = None;
                    this.mark_config_dirty();
                    cx.notify();
                }),
            ))
            .child(components::selectable(
                "target-kicad",
                "KiCad",
                target == "kicad",
                cx.listener(|this, _event: &ClickEvent, _window, cx| {
                    if let Ok(mut state) = this.state.lock() {
                        state.set_export_target("kicad".to_string());
                    }
                    this.active_color_picker = None;
                    this.mark_config_dirty();
                    cx.notify();
                }),
            ));

        let mut scope_row = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(12.0))
            .child(components::fixed_label(locale::t(
                "Export scope:",
                "导出范围:",
            )));

        let modes: [(&'static str, &'static str); 4] = if target == "kicad" {
            [
                ("full", locale::t("Full", "完整")),
                ("symbol", locale::t("Symbol", "符号")),
                ("footprint", locale::t("Footprint", "封装")),
                ("3d", "3D"),
            ]
        } else {
            [
                ("full", locale::t("Full", "完整")),
                ("schlib", "SchLib"),
                ("pcblib", "PcbLib"),
                ("3d", "3D"),
            ]
        };
        let current_mode = if target == "kicad" {
            kicad_mode.clone()
        } else {
            mode.clone()
        };
        for (value, label) in modes {
            let target = target.clone();
            scope_row = scope_row.child(components::selectable(
                format!("scope-{value}"),
                label,
                current_mode == value,
                cx.listener(move |this, _event: &ClickEvent, _window, cx| {
                    if let Ok(mut state) = this.state.lock() {
                        if target == "kicad" {
                            state.set_export_kicad_mode(value.to_string());
                        } else {
                            state.set_export_mode(value.to_string());
                        }
                    }
                    this.mark_config_dirty();
                    cx.notify();
                }),
            ));
        }

        let mut toggles = div()
            .flex()
            .flex_row()
            .flex_wrap()
            .items_center()
            .justify_center()
            .w_full()
            .gap(px(12.0));

        if target != "kicad" && mode != "3d" {
            toggles = toggles
                .child(components::hint(
                    components::selectable(
                        "toggle-merge",
                        locale::t("Merge", "合并"),
                        merge,
                        cx.listener(|this, _event: &ClickEvent, _window, cx| {
                            if let Ok(mut state) = this.state.lock() {
                                let next = !state.export_merge;
                                state.set_export_merge(next);
                            }
                            this.mark_config_dirty();
                            cx.notify();
                        }),
                    ),
                    locale::t(
                        "Merge: export all components into a single library file",
                        "合并：将所有元件输出到同一个库文件中",
                    ),
                ))
                .child(components::hint(
                    components::selectable(
                        "toggle-append",
                        locale::t("Append", "追加"),
                        append,
                        cx.listener(|this, _event: &ClickEvent, _window, cx| {
                            if let Ok(mut state) = this.state.lock() {
                                let next = !state.export_append;
                                state.set_export_append(next);
                            }
                            this.mark_config_dirty();
                            cx.notify();
                        }),
                    ),
                    locale::t(
                        "Append: add components to an existing library instead of overwriting it",
                        "追加：向已有库文件追加元件，而不是覆盖",
                    ),
                ));
        }

        toggles = toggles
            .child(components::hint(
                components::selectable(
                    "toggle-continue",
                    locale::t("Continue On Error", "出错继续"),
                    continue_on_error,
                    cx.listener(|this, _event: &ClickEvent, _window, cx| {
                        if let Ok(mut state) = this.state.lock() {
                            let next = !state.export_continue_on_error;
                            state.set_export_continue_on_error(next);
                        }
                        this.mark_config_dirty();
                        cx.notify();
                    }),
                ),
                locale::t(
                    "Continue On Error: skip failed components and keep exporting instead of stopping on first error",
                    "出错继续：单个元件导出失败时跳过并继续，不中断整个导出",
                ),
            ))
            .child(components::hint(
                components::selectable(
                    "toggle-force",
                    locale::t("Force", "强制"),
                    force,
                    cx.listener(|this, _event: &ClickEvent, _window, cx| {
                        if let Ok(mut state) = this.state.lock() {
                            let next = !state.export_force;
                            state.set_export_force(next);
                        }
                        this.mark_config_dirty();
                        cx.notify();
                    }),
                ),
                locale::t(
                    "Force: re-export and overwrite even if the component already exists in the library",
                    "强制：即使元件已存在也强制重新导出，覆盖现有数据",
                ),
            ))
            .child(components::hint(
                components::selectable(
                    "toggle-english",
                    locale::t("English metadata", "英文元数据"),
                    english,
                    cx.listener(|this, _event: &ClickEvent, _window, cx| {
                        if let Ok(mut state) = this.state.lock() {
                            let english = !state.export_lcsc_english;
                            state.set_export_lcsc_english(english);
                            state.bom_language =
                                if english { "english" } else { "chinese" }.to_string();
                        }
                        this.mark_config_dirty();
                        cx.notify();
                    }),
                ),
                locale::t(
                    "English metadata: fill component property fields (description, package, etc.) in English",
                    "英文元数据：使用英文填写元件属性字段（如描述、封装等）",
                ),
            ));

        let mut body = div()
            .flex()
            .flex_col()
            .w_full()
            .gap(px(10.0))
            .child(components::section_header(
                locale::t("Export Settings", "导出设置"),
                locale::t("Defaults are saved automatically.", "设置会自动保存。"),
            ))
            .child(target_row);

        if target != "kicad" {
            body = body.child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .items_center()
                    .gap(px(12.0))
                    .child(components::color_setting(
                        locale::t("SCH stroke color:", "原理图描边颜色:"),
                        self.schematic_stroke_color,
                        &stroke_input,
                    ))
                    .child(components::color_setting(
                        locale::t("SCH fill color:", "原理图填充颜色:"),
                        self.schematic_fill_color,
                        &fill_input,
                    )),
            );
        }

        body = body
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(12.0))
                    .child(components::fixed_label(locale::t(
                        "Export directory:",
                        "导出目录:",
                    )))
                    .child(div().flex_1().child(Input::new(&path_input).h(px(30.0))))
                    .child(components::button(
                        "browse-export-dir",
                        locale::t("Browse", "浏览"),
                        true,
                        cx.listener(|this, _event: &ClickEvent, _window, cx| {
                            this.start_file_dialog(FileDialogKind::PickFolder);
                            cx.notify();
                        }),
                    )),
            )
            .child(scope_row)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(12.0))
                    .child(components::fixed_label(locale::t(
                        "Library name:",
                        "库名称:",
                    )))
                    .child(
                        div()
                            .w(px(320.0))
                            .child(Input::new(&library_input).h(px(30.0))),
                    )
                    .child(
                        div()
                            .text_size(px(BODY))
                            .text_color(theme::muted())
                            .child(locale::t("Parallel jobs:", "并行任务数:")),
                    )
                    .child(
                        div()
                            .w(px(80.0))
                            .child(Input::new(&parallel_input).h(px(30.0))),
                    ),
            )
            .child(toggles);

        components::section(body).into_any_element()
    }
}
