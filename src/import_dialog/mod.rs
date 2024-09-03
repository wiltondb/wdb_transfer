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
use std::time::Duration;
use std::time::Instant;

use clipboard_win::formats;
use clipboard_win::set_clipboard;
use nwg::NativeUi;

use crate::*;
use common_gui::TableWithSize;
use common::TdsConnConfig;
use common::TransferError;
use nwg_ui as ui;
use ui::Controls;
use ui::Events;
use ui::Layout;
use ui::PopupDialog;

pub use args::ImportArgs;
pub use args::ImportDialogArgs;
pub(self) use controls::ImportDialogControls;
pub use dialog::ImportDialog;
use events::ImportDialogEvents;
use layout::ImportDialogLayout;
pub use result::ImportDialogResult;
use result::ImportResult;
