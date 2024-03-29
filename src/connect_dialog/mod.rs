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

mod args;
mod controls;
mod dialog;
mod events;
mod layout;
mod nui;
mod result;

use std::thread;

use nwg::NativeUi;

use crate::*;
use nwg_ui as ui;
use ui::Controls;
use ui::Events;
use ui::Layout;
use ui::PopupArgs;
use ui::PopupDialog;

use common::TdsConnConfig;
use connect_check_dialog::ConnectCheckDialog;
use connect_check_dialog::ConnectCheckDialogArgs;
use connect_check_dialog::ConnectCheckDialogResult;
use load_dbnames_dialog::LoadDbnamesDialog;
use load_dbnames_dialog::LoadDbnamesDialogArgs;
use load_dbnames_dialog::LoadDbnamesDialogResult;

pub use args::ConnectDialogArgs;
pub(self) use controls::ConnectDialogControls;
pub use dialog::ConnectDialog;
use events::ConnectDialogEvents;
use layout::ConnectDialogLayout;
pub use result::ConnectDialogResult;
