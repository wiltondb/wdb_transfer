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
pub(super) struct ConnectDialogControls {
    layout: ConnectDialogLayout,

    pub(super) font_normal: nwg::Font,

    pub(super) icon: nwg::Icon,
    pub(super) window: nwg::Window,

    pub(super) hostname_label: nwg::Label,
    pub(super) hostname_input: nwg::TextInput,
    pub(super) port_label: nwg::Label,
    pub(super) port_input: nwg::TextInput,
    pub(super) username_label: nwg::Label,
    pub(super) username_input: nwg::TextInput,
    pub(super) password_label: nwg::Label,
    pub(super) password_input: nwg::TextInput,
    pub(super) database_label: nwg::Label,
    pub(super) database_input: nwg::TextInput,
    pub(super) accept_invalid_tls_checkbox: nwg::CheckBox,

    pub(super) test_button: nwg::Button,
    pub(super) cancel_button: nwg::Button,

    pub(super) check_notice: ui::SyncNotice,
    pub(super) load_notice: ui::SyncNotice,
}

impl ui::Controls for ConnectDialogControls {

    fn build(&mut self) -> Result<(), nwg::NwgError> {
        nwg::Font::builder()
            .size(ui::font_size_builder()
                .normal()
                .build())
            .build(&mut self.font_normal)?;

        nwg::Icon::builder()
            .source_embed(Some(&nwg::EmbedResource::load(None)
                .expect("Error loading embedded resource")))
            .source_embed_id(2)
            .build(&mut self.icon)?;

        nwg::Window::builder()
            .size((480, 240))
            .icon(Some(&self.icon))
            .center(true)
            .title("DB Connection")
            .build(&mut self.window)?;

        nwg::Label::builder()
            .text("Hostname:")
            .font(Some(&self.font_normal))
            .h_align(nwg::HTextAlign::Left)
            .parent(&self.window)
            .build(&mut self.hostname_label)?;
        nwg::TextInput::builder()
            .font(Some(&self.font_normal))
            .parent(&self.window)
            .build(&mut self.hostname_input)?;
        nwg::Label::builder()
            .text("Port:")
            .font(Some(&self.font_normal))
            .h_align(nwg::HTextAlign::Left)
            .parent(&self.window)
            .build(&mut self.port_label)?;
        nwg::TextInput::builder()
            .flags(nwg::TextInputFlags::VISIBLE | nwg::TextInputFlags::NUMBER)
            .font(Some(&self.font_normal))
            .parent(&self.window)
            .build(&mut self.port_input)?;
        nwg::Label::builder()
            .text("Username:")
            .font(Some(&self.font_normal))
            .h_align(nwg::HTextAlign::Left)
            .parent(&self.window)
            .build(&mut self.username_label)?;
        nwg::TextInput::builder()
            .font(Some(&self.font_normal))
            .parent(&self.window)
            .build(&mut self.username_input)?;
        nwg::Label::builder()
            .text("Password:")
            .font(Some(&self.font_normal))
            .h_align(nwg::HTextAlign::Left)
            .parent(&self.window)
            .build(&mut self.password_label)?;
        nwg::TextInput::builder()
            .password(Some('*'))
            .font(Some(&self.font_normal))
            .parent(&self.window)
            .build(&mut self.password_input)?;
        nwg::Label::builder()
            .text("Database:")
            .font(Some(&self.font_normal))
            .h_align(nwg::HTextAlign::Left)
            .parent(&self.window)
            .build(&mut self.database_label)?;
        nwg::TextInput::builder()
            .font(Some(&self.font_normal))
            .parent(&self.window)
            .build(&mut self.database_input)?;
        nwg::CheckBox::builder()
            .check_state(nwg::CheckBoxState::Checked)
            .text("Accept invalid TLS certificates/hosts")
            .font(Some(&self.font_normal))
            .parent(&self.window)
            .build(&mut self.accept_invalid_tls_checkbox)?;

        nwg::Button::builder()
            .text("Test connection")
            .font(Some(&self.font_normal))
            .parent(&self.window)
            .build(&mut self.test_button)?;

        nwg::Button::builder()
            .text("Cancel")
            .font(Some(&self.font_normal))
            .parent(&self.window)
            .build(&mut self.cancel_button)?;

        ui::notice_builder()
            .parent(&self.window)
            .build(&mut self.check_notice)?;
        ui::notice_builder()
            .parent(&self.window)
            .build(&mut self.load_notice)?;

        self.layout.build(&self)?;

        Ok(())
    }

    fn update_tab_order(&self) {
        ui::tab_order_builder()
            .control(&self.hostname_input)
            .control(&self.port_input)
            .control(&self.username_input)
            .control(&self.password_input)
            .control(&self.database_input)
            .control(&self.accept_invalid_tls_checkbox)
            .control(&self.test_button)
            .control(&self.cancel_button)
            .build();
    }
}
