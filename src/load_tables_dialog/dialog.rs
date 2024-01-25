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
use nwg::EventData;
use crate::load_tables_dialog::result::TableWithRowsCount;

#[derive(Default)]
pub struct LoadTablesDialog {
    pub(super) c: LoadTablesDialogControls,

    args: LoadTablesDialogArgs,
    load_join_handle: ui::PopupJoinHandle<LoadTablesResult>,
    dialog_result: LoadTablesDialogResult
}

impl LoadTablesDialog {
    pub(super) fn on_load_complete(&mut self, _: nwg::EventData) {
        self.c.load_notice.receive();
        let res = self.load_join_handle.join();
        let success = res.error.is_empty();
        self.stop_progress_bar(success.clone());
        if !success {
            self.dialog_result = LoadTablesDialogResult::failure();
            self.c.label.set_text("Load failed");
            self.c.details_box.set_text(&res.error);
            self.c.copy_clipboard_button.set_enabled(true);
            self.c.close_button.set_enabled(true);
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

    fn load_tables_from_db(conn_config: &TdsConnConfig, dbname: &str) -> Result<Vec<TableWithRowsCount>, TransferError> {
        let runtime = conn_config.create_runtime()?;
        let mut client = conn_config.open_connection(&runtime)?;
        runtime.block_on(async {
            let mut qr_use = tiberius::Query::new(format!("use [{}]", dbname));
            qr_use.execute(&mut client).await?;
            let mut qr_tables = tiberius::Query::new("\
                select table_schema, table_name
                from information_schema.tables
                where table_type = 'BASE TABLE'
                and table_catalog = @P1");
            qr_tables.bind(dbname);
            let mut stream_tables = qr_tables.query(&mut client).await?;
            let rows = stream_tables.into_first_result().await?;
            let mut tables = Vec::new();
            let msg = "Tables select error";
            for row in rows.iter() {
                let schema: &str = row.get(0).ok_or(TransferError::from_str(msg))?;
                let table: &str = row.get(1).ok_or(TransferError::from_str(msg))?;
                let mut qr_count = tiberius::Query::new(format!("select count(1) from [{}].[{}]", schema, table));
                let count = match qr_count.query(&mut client).await {
                    Ok(stream_count) => {
                        let row_opt = stream_count.into_row().await?;
                        let row = row_opt.ok_or(TransferError::from_str(msg))?;
                        let count_i32 : i32 = row.get(0).ok_or(TransferError::from_str(&msg))?;
                        count_i32
                    },
                    Err(_) => -1
                };
                tables.push(TableWithRowsCount::new(schema, table, count));
            }
            Ok(tables)
        })
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
        let sender = self.c.load_notice.sender();
        let cconf = self.args.conn_config.clone();
        let dbname = self.args.dbname.clone();
        let join_handle = thread::spawn(move || {
            let start = Instant::now();
            let res = match LoadTablesDialog::load_tables_from_db(&cconf, &dbname) {
                Ok((dbnames)) => LoadTablesResult::success(dbnames),
                Err(e) => LoadTablesResult::failure(format!("{}", e))
            };
            let remaining = 1000 - start.elapsed().as_millis() as i64;
            if remaining > 0 {
                thread::sleep(Duration::from_millis(remaining as u64));
            }
            sender.send();
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

    fn on_resize(&mut self, _: EventData) {
        self.c.update_tab_order();
    }
}

