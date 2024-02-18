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
use std::path::Path;
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
    export_dialog_join_handle: ui::PopupJoinHandle<ExportDialogResult>,
}

impl AppWindow {

    pub fn new() -> Self {
        Default::default()
    }

    pub(super) fn init(&mut self) {
        use chrono::{DateTime, Local};
        self.conn_config.hostname = String::from("localhost");
        self.conn_config.port = 1433;
        self.conn_config.username = String::from("wilton");
        // todo: removeme
        self.conn_config.password = String::from("wilton");
        self.conn_config.database = String::from("master");
        self.conn_config.accept_invalid_tls = true;

        self.set_status_bar_dbconn_label("none");
        let date = Local::now().format("%Y%m%d");
        self.c.export_filename_input.set_text(&format!("bcp_export_{}.zip", date));
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
            self.sort_tables_by_schema_and_name();
        }
        self.reload_tables_view();
    }

    pub(super) fn open_export_dialog(&mut self, _: nwg::EventData) {
        let dbname = match self.c.export_dbnames_combo.selection_string() {
            Some(name) => name,
            None => return
        };
        let tables: Vec<TableWithRowsCount> = self.tables.iter()
            .filter(|t| t.export)
            .map(|t| t.clone())
            .collect();
        let dir = self.c.export_dest_dir_input.text();
        let filename = self.c.export_filename_input.text();
        let dest_path = Path::new(&dir).join(&filename);
        let mut go_on = true;
        if dest_path.exists() {
            let dest_path_st = dest_path.to_string_lossy().to_string();
            go_on = ui::message_box_warning_yn(&format!(
                "Destination file already exists:\r\n{}\r\n\r\nWould you like to overwrite it?", dest_path_st));
        }
        if go_on {
            self.c.window.set_enabled(false);
            let args = ExportDialogArgs::new(
                &self.c.export_notice, &self.conn_config,  &dbname, &tables, &dir, &filename);
            self.export_dialog_join_handle = ExportDialog::popup(args);
        }
    }

    pub(super) fn await_export_dialog(&mut self, _: nwg::EventData) {
        self.c.window.set_enabled(true);
        self.c.export_notice.receive();
        let _ = self.export_dialog_join_handle.join();
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

    pub(super) fn on_export_dbname_changed(&mut self, _: nwg::EventData) {
        if let Some(name) = &self.c.export_dbnames_combo.selection_string() {
            self.open_load_tables_dialog(nwg::EventData::NoData);
        }
    }

    pub(super) fn on_export_tables_view_sort(&mut self, ed: nwg::EventData) {
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

    pub(super) fn on_export_tables_view_click(&mut self, ed: nwg::EventData) {
        if let nwg::EventData::OnListViewItemIndex
        { row_index: row_idx, .. } = ed {
            self.flip_export_flag(row_idx);
        };
    }

    pub(super) fn on_export_mark_all_button(&mut self, _: nwg::EventData) {
        self.set_all_export_flags(true);
    }

    pub(super) fn on_export_clear_button(&mut self, _: nwg::EventData) {
        self.set_all_export_flags(false);
    }

    pub(super) fn on_export_filter_button(&mut self, _: nwg::EventData) {
        self.reload_tables_view();
    }

    pub(super) fn choose_export_dest_dir(&mut self, _: nwg::EventData) {
        if let Ok(dir) = std::env::current_dir() {
            if let Some(d) = dir.to_str() {
                let _ = self.c.export_dest_dir_chooser.set_default_folder(d);
            }
        }

        if self.c.export_dest_dir_chooser.run(Some(&self.c.window)) {
            self.c.export_dest_dir_input.set_text("");
            if let Ok(sel) = self.c.export_dest_dir_chooser.get_selected_item() {
                let dir = sel.to_string_lossy().to_string();
                self.c.export_dest_dir_input.set_text(&dir);
            }
        }
    }

    pub(super) fn choose_import_file(&mut self, _: nwg::EventData) {
        if let Ok(dir) = std::env::current_dir() {
            if let Some(d) = dir.to_str() {
                let _ = self.c.import_file_chooser.set_default_folder(d);
            }
        }

        if self.c.import_file_chooser.run(Some(&self.c.window)) {
            self.c.import_file_input.set_text("");
            if let Ok(file) = self.c.import_file_chooser.get_selected_item() {
                let fpath_st = file.to_string_lossy().to_string();
                self.c.import_file_input.set_text(&fpath_st);
                self.load_import_file_entries();
            }
        }
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
        self.c.export_dbnames_combo.set_collection(dbnames.clone());
        if count > 0 {
            //self.c.export_dbnames_combo.set_selection(Some(0));
            // todo: removeme
            self.c.export_dbnames_combo.set_selection(Some(count - 1));
            self.on_export_dbname_changed(nwg::EventData::NoData);
        }
        self.c.import_dbnames_combo.set_collection(dbnames);
        if count > 0 {
            self.c.import_dbnames_combo.set_selection(Some(0));
        }
    }

    fn table_matches_filters(&self, rec: &TableWithRowsCount) -> bool {
        let filter = self.c.export_tables_filter_input.text();
        if 0 == filter.len() {
            return true;
        }
        if rec.table.starts_with(&filter) {
            return true;
        }
        wildmatch::WildMatch::new(&filter).matches(&rec.table)
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
                    column_index: 0,
                    text: Some(if rec.export { "YES".to_string() } else { "No".to_string() }),
                    image: None
                });
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

    fn sort_tables_by_schema_and_name(&mut self) {
        self.tables.sort_by(|a, b| {
            let a_schema = a.schema.to_lowercase();
            let b_schema = b.schema.to_lowercase();
            if a_schema.gt(&b_schema) {
                std::cmp::Ordering::Greater
            } else if a_schema.lt(&b_schema) {
                std::cmp::Ordering::Less
            } else {
                a.table.to_lowercase().cmp(&b.table.to_lowercase())
            }
        });
    }

    fn sort_tables(&mut self, col_idx: usize, desc: bool) {
        if col_idx > 3 {
            return;
        }
        self.tables.sort_by(|a, b| {
            if 0 == col_idx {
                if desc {
                    b.export.cmp(&a.export)
                } else {
                    a.export.cmp(&b.export)
                }
            } else if 1 == col_idx {
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

    fn set_all_export_flags(&mut self, export: bool) {
        let tv = &self.c.export_tables_view;
        tv.set_redraw(false);
        for rec in self.tables.iter_mut() {
            rec.export = export
        };
        for row_idx in 0..tv.len() {
            self.c.export_tables_view.update_item(row_idx, nwg::InsertListViewItem {
                index: Some(row_idx as i32),
                column_index: 0,
                text: Some(if export { "YES".to_string() } else { "No".to_string() }),
                image: None
            });
        }
        tv.set_redraw(true);
    }

    fn flip_export_flag(&mut self, row_idx: usize) {
        let export = match self.c.export_tables_view.item(row_idx, 0, 1<<16) {
            Some(mut item) => "no" == item.text.to_lowercase(),
            None => return
        };
        let schema = match self.c.export_tables_view.item(row_idx, 1, 1<<16) {
            Some(item) => item.text,
            None => return
        };
        let table = match self.c.export_tables_view.item(row_idx, 2, 1<<16) {
            Some(item) => item.text,
            None => return
        };
        for rec in self.tables.iter_mut() {
            if schema == rec.schema && table == rec.table {
                rec.export = export;
                break;
            }
        };
        self.c.export_tables_view.update_item(row_idx, nwg::InsertListViewItem {
            index: Some(row_idx as i32),
            column_index: 0,
            text: Some(if export { "YES".to_string() } else { "No".to_string() }),
            image: None
        });

    }

    fn load_import_file_entries(&mut self) {
        use std::fs::File;
        use std::io::BufReader;
        use zip::ZipArchive;

        let file_path = self.c.import_file_input.text();
        if !Path::new(&file_path).exists() {
            ui::message_box_error(&format!("Specified file is not found, path: {}", file_path));
            return;
        }
        let file = match File::open(&file_path) {
            Ok(file) => file,
            Err(e) => {
                ui::message_box_error(&format!("Error opening file, path: {}, message: {}", file_path, e.to_string()));
                return;
            }
        };
        let mut reader = BufReader::new(file);
        let zip = match ZipArchive::new(reader) {
            Ok(zip) => zip,
            Err(e) => {
                ui::message_box_error(&format!("Error opening ZIP file, path: {}, message: {}", file_path, e.to_string()));
                return;
            }
        };
        for name in zip.file_names() {
            println!("{}", name);
        }
    }
}
