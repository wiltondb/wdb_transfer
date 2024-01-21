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
pub(super) struct ConnectDialogEvents {
    pub(super) events: Vec<ui::Event<ConnectDialog>>
}

impl ui::Events<ConnectDialogControls> for ConnectDialogEvents {
    fn build(&mut self, c: &ConnectDialogControls) -> Result<(), nwg::NwgError> {
        ui::event_builder()
            .control(&c.window)
            .event(nwg::Event::OnWindowClose)
            .handler(ConnectDialog::close)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.window)
            .event(nwg::Event::OnResizeEnd)
            .handler(ConnectDialog::on_resize)
            .build(&mut self.events)?;

        ui::event_builder()
            .control(&c.port_input)
            .event(nwg::Event::OnTextInput)
            .handler(ConnectDialog::on_port_input_changed)
            .build(&mut self.events)?;

        ui::event_builder()
            .control(&c.test_button)
            .event(nwg::Event::OnButtonClick)
            .handler(ConnectDialog::open_check_dialog)
            .build(&mut self.events)?;

        ui::event_builder()
            .control(&c.cancel_button)
            .event(nwg::Event::OnButtonClick)
            .handler(ConnectDialog::close)
            .build(&mut self.events)?;

        ui::event_builder()
            .control(&c.check_notice.notice)
            .event(nwg::Event::OnNotice)
            .handler(ConnectDialog::await_check_dialog)
            .build(&mut self.events)?;

        Ok(())
    }
}
