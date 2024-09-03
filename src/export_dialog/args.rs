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
use crate::common_gui::TableWithRowsCount;


#[derive(Default, Clone)]
pub struct ExportArgs {
    pub(super) dbname: String,
    pub(super) tables: Vec<TableWithRowsCount>,
    pub(super) parent_dir: String,
    pub(super) dest_filename: String,
}

#[derive(Default)]
pub struct ExportDialogArgs {
    pub(super) notice_sender: ui::SyncNoticeSender,
    pub(super) conn_config: TdsConnConfig,
    pub(super) export_args: ExportArgs,
}

impl ExportDialogArgs {
    pub fn new(notice: &ui::SyncNotice, conn_config: &TdsConnConfig, dbname: &str, tables: &Vec<TableWithRowsCount>, parent_dir: &str, dest_filename: &str) -> Self {
        Self {
            notice_sender: notice.sender(),
            conn_config: conn_config.clone(),
            export_args: ExportArgs {
                dbname: dbname.to_string(),
                tables: tables.clone(),
                parent_dir: parent_dir.to_string(),
                dest_filename: dest_filename.to_string()
            },
        }
    }

    pub fn send_notice(&self) {
        self.notice_sender.send()
    }
}

impl ui::PopupArgs for ExportDialogArgs {
    fn notify_parent(&self) {
        self.notice_sender.send()
    }
}
