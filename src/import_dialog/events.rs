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
pub(super) struct ImportDialogEvents {
    pub(super) events: Vec<ui::Event<ImportDialog>>
}

impl ui::Events<ImportDialogControls> for ImportDialogEvents {
    fn build(&mut self, c: &ImportDialogControls) -> Result<(), nwg::NwgError> {
        ui::event_builder()
            .control(&c.window)
            .event(nwg::Event::OnWindowClose)
            .handler(ImportDialog::close)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.window)
            .event(nwg::Event::OnResizeEnd)
            .handler(ImportDialog::on_resize)
            .build(&mut self.events)?;

        ui::event_builder()
            .control(&c.copy_clipboard_button)
            .event(nwg::Event::OnButtonClick)
            .handler(ImportDialog::copy_to_clipboard)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.close_button)
            .event(nwg::Event::OnButtonClick)
            .handler(ImportDialog::close)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.progress_notice.notice)
            .event(nwg::Event::OnNotice)
            .handler(ImportDialog::on_progress)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.complete_notice.notice)
            .event(nwg::Event::OnNotice)
            .handler(ImportDialog::on_complete)
            .build(&mut self.events)?;

        Ok(())
    }
}
