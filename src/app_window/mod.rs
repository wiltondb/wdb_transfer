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

mod controls;
mod events;
mod layout;
mod nui;
mod window;

use crate::*;
use nwg_ui as ui;
use ui::Controls;
use ui::Events;
use ui::Layout;
use ui::PopupDialog;

use about_dialog::AboutDialog;
use about_dialog::AboutDialogArgs;
use common::TableWithRowsCount;
use common::TableWithSize;
use common::TdsConnConfig;
use connect_dialog::ConnectDialog;
use connect_dialog::ConnectDialogArgs;
use connect_dialog::ConnectDialogResult;
use export_dialog::ExportDialog;
use export_dialog::ExportDialogArgs;
use export_dialog::ExportDialogResult;
use import_dialog::ImportDialog;
use import_dialog::ImportDialogArgs;
use import_dialog::ImportDialogResult;
use load_dbnames_dialog::LoadDbnamesDialog;
use load_dbnames_dialog::LoadDbnamesDialogArgs;
use load_dbnames_dialog::LoadDbnamesDialogResult;
use load_tables_dialog::LoadTablesDialog;
use load_tables_dialog::LoadTablesDialogArgs;
use load_tables_dialog::LoadTablesDialogResult;

pub(self) use controls::AppWindowControls;
pub(self) use events::AppWindowEvents;
use layout::AppWindowLayout;
pub use window::AppWindow;
