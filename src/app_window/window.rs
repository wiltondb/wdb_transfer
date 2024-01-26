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

use std::os::windows::process::CommandExt;
use std::process::Command;
use std::process::Stdio;

use super::*;

const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Default)]
pub struct AppWindow {
    pub(super) c: AppWindowControls,

    conn_config: TdsConnConfig,

    tables: Vec<TableWithRowsCount>,

    about_dialog_join_handle: ui::PopupJoinHandle<()>,
    connect_dialog_join_handle: ui::PopupJoinHandle<ConnectDialogResult>,
    load_dbnames_dialog_join_handle: ui::PopupJoinHandle<LoadDbnamesDialogResult>,
    load_tables_dialog_join_handle: ui::PopupJoinHandle<LoadTablesDialogResult>,
}

impl AppWindow {

    pub fn new() -> Self {
        Default::default()
    }

    pub(super) fn init(&mut self) {
        self.conn_config.hostname = String::from("localhost");
        self.conn_config.port = 1433;
        self.conn_config.username = String::from("wilton");
        // todo: removeme
        self.conn_config.password = String::from("wilton");
        self.conn_config.database = String::from("master");
        self.conn_config.accept_invalid_tls = true;

        self.set_status_bar_dbconn_label("none");
        self.open_connect_dialog(nwg::EventData::NoData);
    }

    pub(super) fn close(&mut self, _: nwg::EventData) {
        self.c.window.set_visible(false);
        nwg::stop_thread_dispatch();
    }

    pub(super) fn open_about_dialog(&mut self, _: nwg::EventData) {
        self.c.window.set_enabled(false);
        let args = AboutDialogArgs::new(&self.c.about_notice);
        self.about_dialog_join_handle = AboutDialog::popup(args);
    }

    pub(super) fn await_about_dialog(&mut self, _: nwg::EventData) {
        self.c.window.set_enabled(true);
        self.c.about_notice.receive();
        let _ = self.about_dialog_join_handle.join();
    }

    pub(super) fn open_connect_dialog(&mut self, _: nwg::EventData) {
        self.c.window.set_enabled(false);
        let args = ConnectDialogArgs::new(&self.c.connect_notice, self.conn_config.clone());
        self.connect_dialog_join_handle = ConnectDialog::popup(args);
    }

    pub(super) fn await_connect_dialog(&mut self, _: nwg::EventData) {
        self.c.window.set_enabled(true);
        self.c.connect_notice.receive();
        let res = self.connect_dialog_join_handle.join();
        if !res.cancelled {
            self.set_dbnames(&res.dbnames);
            self.conn_config = res.conn_config;
            let sbar_label = format!(
                "{}:{}", &self.conn_config.hostname, &self.conn_config.port);
            self.set_status_bar_dbconn_label(&sbar_label);
        }
    }

    pub(super) fn open_load_dbnames_dialog(&mut self, _: nwg::EventData) {
        self.c.window.set_enabled(false);
        let cc = self.conn_config.clone();
        let args = LoadDbnamesDialogArgs::new(&self.c.load_dbnames_notice, cc);
        self.load_dbnames_dialog_join_handle = LoadDbnamesDialog::popup(args);
    }

    pub(super) fn await_load_dbnames_dialog(&mut self, _: nwg::EventData) {
        self.c.window.set_enabled(true);
        self.c.load_dbnames_notice.receive();
        let res = self.load_dbnames_dialog_join_handle.join();
        if res.success {
            self.set_dbnames(&res.dbnames);
        }
    }

    pub(super) fn open_load_tables_dialog(&mut self, _: nwg::EventData) {
        if let Some(dbname) = &self.c.export_dbnames_combo.selection_string() {
            self.c.window.set_enabled(false);
            let cc = self.conn_config.clone();
            let args = LoadTablesDialogArgs::new(&self.c.load_tables_notice, cc, dbname);
            self.load_tables_dialog_join_handle = LoadTablesDialog::popup(args);
        }
    }

    pub(super) fn await_load_tables_dialog(&mut self, _: nwg::EventData) {
        self.c.window.set_enabled(true);
        self.c.load_tables_notice.receive();
        let mut res = self.load_tables_dialog_join_handle.join();
        self.tables = Vec::new();
        if res.success {
            self.tables = res.tables;
        }
        self.reload_tables_view();
    }

    pub(super) fn open_website(&mut self, _: nwg::EventData) {
        let _ = Command::new("cmd")
            .arg("/c")
            .arg("start")
            .arg("https://wiltondb.com")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .status();
    }

    pub(super) fn on_dbname_changed(&mut self, _: nwg::EventData) {
        if let Some(name) = &self.c.export_dbnames_combo.selection_string() {
            self.open_load_tables_dialog(nwg::EventData::NoData);
        }
    }

    pub(super) fn on_tables_view_sort(&mut self, ed: nwg::EventData) {
        let col_idx = if let nwg::EventData::OnListViewItemIndex
        { column_index: col_idx, .. } = ed {
            col_idx
        } else {
            return;
        };
        let old_arrow = self.c.export_tables_view
            .column_sort_arrow(col_idx)
            .expect("Sort not initialized");
        let arrow = match old_arrow {
            nwg::ListViewColumnSortArrow::Up => nwg::ListViewColumnSortArrow::Down,
            nwg::ListViewColumnSortArrow::Down => nwg::ListViewColumnSortArrow::Up
        };
        let desc = match arrow {
            nwg::ListViewColumnSortArrow::Up => true,
            nwg::ListViewColumnSortArrow::Down => false
        };
        self.sort_tables(col_idx, desc);
        self.c.export_tables_view.set_column_sort_arrow(col_idx, Some(arrow));
        self.reload_tables_view();
    }

    pub(super) fn on_resize(&mut self, _: nwg::EventData) {
        self.c.update_tab_order();
    }

    fn set_status_bar_dbconn_label(&self, text: &str) {
        self.c.status_bar.set_text(0, &format!("  DB connection: {}", text));
    }

    fn set_dbnames(&mut self, dbnames_all: &Vec<String>) {
        use std::cmp::Ordering;
        let mut dbnames: Vec<String> = dbnames_all.iter().filter(|name| {
            !vec!("msdb", "tempdb").contains(&name.as_str())
        }).map(|name| name.clone()).collect();
        dbnames.sort_by(|a, b| {
            if "master" == a {
                Ordering::Less
            } else if "master" == b {
                Ordering::Greater
            } else {
                a.to_lowercase().cmp(&b.to_lowercase())
            }
        });
        let count = dbnames.len();
        self.c.export_dbnames_combo.set_collection(dbnames);
        if count > 0 {
            self.c.export_dbnames_combo.set_selection(Some(0));
            self.on_dbname_changed(nwg::EventData::NoData);
        }
    }

    fn table_matches_filters(&self, _table: &TableWithRowsCount) -> bool {
        // todo
        true
    }

    fn reload_tables_view(&self) {
        let tv = &self.c.export_tables_view;
        tv.set_redraw(false);
        loop {
            let removed = tv.remove_item(0);
            if !removed {
                break;
            }
        };
        let mut idx = 0 as i32;
        for rec in &self.tables {
            if self.table_matches_filters(rec) {
                tv.insert_item(nwg::InsertListViewItem {
                    index: Some(idx as i32),
                    column_index: 1,
                    text: Some(rec.schema.clone()),
                    image: None
                });
                tv.insert_item(nwg::InsertListViewItem {
                    index: Some(idx as i32),
                    column_index: 2,
                    text: Some(rec.table.clone()),
                    image: None
                });
                tv.insert_item(nwg::InsertListViewItem {
                    index: Some(idx as i32),
                    column_index: 3,
                    text: Some(rec.row_count.to_string()),
                    image: None
                });
                idx += 1;
            }
        }
        tv.set_redraw(true);
    }

    fn sort_tables(&mut self, col_idx: usize, desc: bool) {
        if col_idx < 1 || col_idx > 3 {
            return;
        }
        self.tables.sort_by(|a, b| {
            if 1 == col_idx {
                if desc {
                    b.schema.to_lowercase().cmp(&a.schema.to_lowercase())
                } else {
                    a.schema.to_lowercase().cmp(&b.schema.to_lowercase())
                }
            } else if 2 == col_idx {
                if desc {
                    b.table.to_lowercase().cmp(&a.table.to_lowercase())
                } else {
                    a.table.to_lowercase().cmp(&b.table.to_lowercase())
                }
            } else if 3 == col_idx {
                if desc {
                    b.row_count.cmp(&a.row_count)
                } else {
                    a.row_count.cmp(&b.row_count)
                }
            } else {
                std::cmp::Ordering::Equal
            }
        });
    }
}
