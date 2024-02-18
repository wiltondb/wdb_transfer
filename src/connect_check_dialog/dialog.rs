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

use super::*;

#[derive(Default)]
pub struct ConnectCheckDialog {
    pub(super) c: ConnectCheckDialogControls,

    args: ConnectCheckDialogArgs,
    check_join_handle: ui::PopupJoinHandle<ConnectCheckDialogResult>,
    result: ConnectCheckDialogResult
}

impl ConnectCheckDialog {
    pub(super) fn on_connection_check_complete(&mut self, _: nwg::EventData) {
        self.c.check_notice.receive();
        self.result = self.check_join_handle.join();
        self.stop_progress_bar(self.result.success);
        let label = if self.result.success {
            "Connection successful"
        } else {
            "Connection failed"
        };
        self.c.label.set_text(label);
        self.c.details_box.set_text(&self.result.message);
        self.c.copy_clipboard_button.set_enabled(true);
        self.c.close_button.set_enabled(true);
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

    fn check_tds_conn(conn_config: &TdsConnConfig) -> Result<String, TransferError> {
        let runtime = conn_config.create_runtime()?;
        let mut client = conn_config.open_connection_default(&runtime)?;
        runtime.block_on(async {
            let mut qr = tiberius::Query::new("select @@version");
            let mut stream = qr.query(&mut client).await?;
            let row_opt = stream.into_row().await?;
            let msg = "Invalid empty response to version query".to_string();
            let row = row_opt.ok_or(TransferError::new(&msg))?;
            let res: &str = row.get(0).ok_or(TransferError::new(&msg))?;
            Ok(res.to_string())
        })
    }
}

impl ui::PopupDialog<ConnectCheckDialogArgs, ConnectCheckDialogResult> for ConnectCheckDialog {
    fn popup(args: ConnectCheckDialogArgs) -> ui::PopupJoinHandle<ConnectCheckDialogResult> {
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
        let sender = self.c.check_notice.sender();
        let cc = self.args.conn_config.clone();
        let join_handle = thread::spawn(move || {
            let start = Instant::now();
            let res = match ConnectCheckDialog::check_tds_conn(&cc) {
                Ok(version) => ConnectCheckDialogResult::success(version),
                Err(e) => ConnectCheckDialogResult::failure(format!("{}", e))
            };
            let remaining = 1000 - start.elapsed().as_millis() as i64;
            if remaining > 0 {
                thread::sleep(Duration::from_millis(remaining as u64));
            }
            sender.send();
            res
        });
        self.check_join_handle = ui::PopupJoinHandle::from(join_handle);
    }

    fn result(&mut self) -> ConnectCheckDialogResult {
        self.result.clone()
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

