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
pub(super) struct LoadDbnamesDialogEvents {
    pub(super) events: Vec<ui::Event<LoadDbnamesDialog>>
}

impl ui::Events<LoadDbnamesDialogControls> for LoadDbnamesDialogEvents {
    fn build(&mut self, c: &LoadDbnamesDialogControls) -> Result<(), nwg::NwgError> {
        ui::event_builder()
            .control(&c.window)
            .event(nwg::Event::OnWindowClose)
            .handler(LoadDbnamesDialog::close)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.window)
            .event(nwg::Event::OnResizeEnd)
            .handler(LoadDbnamesDialog::on_resize)
            .build(&mut self.events)?;

        ui::event_builder()
            .control(&c.copy_clipboard_button)
            .event(nwg::Event::OnButtonClick)
            .handler(LoadDbnamesDialog::copy_to_clipboard)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.close_button)
            .event(nwg::Event::OnButtonClick)
            .handler(LoadDbnamesDialog::close)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.load_notice.notice)
            .event(nwg::Event::OnNotice)
            .handler(LoadDbnamesDialog::on_load_complete)
            .build(&mut self.events)?;

        Ok(())
    }
}
