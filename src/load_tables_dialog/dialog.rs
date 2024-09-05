/*
 * Copyright 2024, WiltonDB Software
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::time;

use super::*;

#[derive(Default)]
pub struct LoadTablesDialog {
    pub(super) c: LoadTablesDialogControls,

    args: LoadTablesDialogArgs,
    load_join_handle: ui::PopupJoinHandle<LoadTablesResult>,
    dialog_result: LoadTablesDialogResult,

    progress_pending: Vec<String>,
    progress_last_updated: u128,
}

impl LoadTablesDialog {
    pub(super) fn on_progress(&mut self, _: nwg::EventData) {
        let msg = self.c.progress_notice.receive();
        self.progress_pending.push(msg);
        let now = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis();
        if now - self.progress_last_updated > 100 {
            let joined = self.progress_pending.join("\r\n");
            self.progress_pending.clear();
            self.progress_last_updated = now;
            self.c.details_box.appendln(&joined);
        }
    }

    pub(super) fn on_complete(&mut self, _: nwg::EventData) {
        self.c.complete_notice.receive();
        let res = self.load_join_handle.join();
        let success = res.error.is_empty();
        self.stop_progress_bar(success.clone());
        if !success {
            self.dialog_result = LoadTablesDialogResult::failure();
            self.c.label.set_text("Load tables failed");
            self.progress_pending.push(res.error);
            self.c.copy_clipboard_button.set_enabled(true);
            self.c.close_button.set_enabled(true);
            if self.progress_pending.len() > 0 {
                let joined = self.progress_pending.join("\r\n");
                self.c.details_box.appendln(&joined);
                self.progress_pending.clear();
            }
        } else {
            self.dialog_result = LoadTablesDialogResult::success(res.tables);
            self.close(nwg::EventData::NoData)
        }
    }

    pub(super) fn copy_to_clipboard(&mut self, _: nwg::EventData) {
        let text = self.c.details_box.text();
        let _ = set_clipboard(formats::Unicode, &text);
    }

    fn stop_progress_bar(&self, success: bool) {
        self.c.progress_bar.set_marquee(false, 0);
        self.c.progress_bar.remove_flags(nwg::ProgressBarFlags::MARQUEE);
        self.c.progress_bar.set_pos(1);
        if !success {
            self.c.progress_bar.set_state(nwg::ProgressBarState::Error)
        }
    }
}

impl ui::PopupDialog<LoadTablesDialogArgs, LoadTablesDialogResult> for LoadTablesDialog {
    fn popup(args: LoadTablesDialogArgs) -> ui::PopupJoinHandle<LoadTablesDialogResult> {
        let join_handle = thread::spawn(move || {
            let data = Self {
                args,
                ..Default::default()
            };
            let mut dialog = Self::build_ui(data).expect("Failed to build UI");
            nwg::dispatch_thread_events();
            dialog.result()
        });
        ui::PopupJoinHandle::from(join_handle)
    }

    fn init(&mut self) {
        let complete_sender = self.c.complete_notice.sender();
        let progress_sender = self.c.progress_notice.sender();
        let cconf = self.args.conn_config.clone();
        let dbname = self.args.dbname.clone();
        let join_handle = thread::spawn(move || {
            let start = Instant::now();
            let progress_fun = |st: &str| {
                progress_sender.send_value(st)
            };
            let res = match common::load_tables_from_db(&progress_fun, &cconf, &dbname) {
                Ok(dbnames) => LoadTablesResult::success(dbnames),
                Err(e) => LoadTablesResult::failure(format!("{}", e))
            };
            let remaining = 1000 - start.elapsed().as_millis() as i64;
            if remaining > 0 {
                thread::sleep(Duration::from_millis(remaining as u64));
            }
            complete_sender.send();
            res
        });
        self.load_join_handle = ui::PopupJoinHandle::from(join_handle);
    }

    fn result(&mut self) -> LoadTablesDialogResult {
        self.dialog_result.clone()
    }

    fn close(&mut self, _: nwg::EventData) {
        self.args.send_notice();
        self.c.window.set_visible(false);
        nwg::stop_thread_dispatch();
    }

    fn on_resize(&mut self, _: nwg::EventData) {
        self.c.update_tab_order();
    }
}

