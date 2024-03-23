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

    fn load_tables_from_db(progress: &ui::SyncNoticeValueSender<String>, conn_config: &TdsConnConfig, dbname: &str) -> Result<Vec<TableWithRowsCount>, TransferError> {
        let runtime = conn_config.create_runtime()?;
        let mut client = conn_config.open_connection_to_db(&runtime, dbname)?;
        runtime.block_on(async {
            progress.send_value("Loading tables ...");
            let qr_bbf = tiberius::Query::new("\
                select
                    schema_name(tb.schema_id) as table_schema,
                    tb.name as table_name,
                    case
                        when pc.reltuples is null then cast(-1 as bigint)
                        when pc.reltuples = -1 then cast(0 as bigint)
                        else cast(pc.reltuples as bigint)
                    end as row_count
                from sys.tables as tb
                left join pg_catalog.pg_class pc
                  on pc.relnamespace = tb.schema_id
                  and pc.relname = tb.name
                where
                    pc.relkind in ('r', 'f', 'p')");
            let qs_bbf = qr_bbf.query(&mut client).await;
            let qs = if let Ok(qs) = qs_bbf {
                qs
            } else {
                let qr_mssql = tiberius::Query::new("\
                    select
                        schema_name(tb.schema_id) as table_schema,
                        tb.name as table_name,
                        case
                            when st.row_count is null then cast(-1 as bigint)
                            else st.row_count
                        end as row_count
                    from sys.tables as tb
                    left join sys.dm_db_partition_stats as st
                        on tb.object_id = st.object_id
                    where
                        tb.type_desc = 'USER_TABLE'
                        and st.index_id IN (0, 1)");
                std::mem::drop(qs_bbf);
                let qs_mssql = qr_mssql.query(&mut client).await;
                if let Ok(qs) = qs_mssql {
                    qs
                } else {
                    let mut qr_generic = tiberius::Query::new("\
                        select
                            table_schema,
                            table_name,
                            cast(-1 as bigint) as row_count
                        from information_schema.tables
                        where table_type = 'BASE TABLE'
                        and table_catalog = @P1");
                    qr_generic.bind(dbname);
                    std::mem::drop(qs_mssql);
                    qr_generic.query(&mut client).await?
                }
            };
            let rows = qs.into_first_result().await?;
            let mut tables = Vec::new();
            let msg = "Tables select error";
            for row in rows.iter() {
                let schema: &str = row.get(0).ok_or(TransferError::from_str(msg))?;
                let table: &str = row.get(1).ok_or(TransferError::from_str(msg))?;
                let count: i64 = row.get(2).ok_or(TransferError::from_str(msg))?;
                progress.send_value(format!("{}.{} {} rows", schema, table, count));
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
        let complete_sender = self.c.complete_notice.sender();
        let progress_sender = self.c.progress_notice.sender();
        let cconf = self.args.conn_config.clone();
        let dbname = self.args.dbname.clone();
        let join_handle = thread::spawn(move || {
            let start = Instant::now();
            let res = match LoadTablesDialog::load_tables_from_db(&progress_sender, &cconf, &dbname) {
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

