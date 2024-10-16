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
pub(super) struct AppWindowEvents {
    pub(super) events: Vec<ui::Event<AppWindow>>
}

impl ui::Events<AppWindowControls> for AppWindowEvents {
    fn build(&mut self, c: &AppWindowControls) -> Result<(), nwg::NwgError> {
        ui::event_builder()
            .control(&c.window)
            .event(nwg::Event::OnWindowClose)
            .handler(AppWindow::close)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.window)
            .event(nwg::Event::OnResizeEnd)
            .handler(AppWindow::on_resize)
            .build(&mut self.events)?;

        ui::event_builder()
            .control(&c.file_connect_menu_item)
            .event(nwg::Event::OnMenuItemSelected)
            .handler(AppWindow::open_connect_dialog)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.file_exit_menu_item)
            .event(nwg::Event::OnMenuItemSelected)
            .handler(AppWindow::close)
            .build(&mut self.events)?;

        ui::event_builder()
            .control(&c.help_about_menu_item)
            .event(nwg::Event::OnMenuItemSelected)
            .handler(AppWindow::open_about_dialog)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.help_website_menu_item)
            .event(nwg::Event::OnMenuItemSelected)
            .handler(AppWindow::open_website)
            .build(&mut self.events)?;

        ui::event_builder()
            .control(&c.export_tables_mark_all_button)
            .event(nwg::Event::OnButtonClick)
            .handler(AppWindow::on_export_mark_all_button)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.export_tables_clear_button)
            .event(nwg::Event::OnButtonClick)
            .handler(AppWindow::on_export_clear_button)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.export_tables_copy_name_button)
            .event(nwg::Event::OnButtonClick)
            .handler(AppWindow::on_export_copy_name_button)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.export_tables_filter_button)
            .event(nwg::Event::OnButtonClick)
            .handler(AppWindow::on_export_filter_button)
            .build(&mut self.events)?;

        ui::event_builder()
            .control(&c.export_tables_view)
            .event(nwg::Event::OnListViewColumnClick)
            .handler(AppWindow::on_export_tables_view_sort)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.export_tables_view)
            .event(nwg::Event::OnListViewClick)
            .handler(AppWindow::on_export_tables_view_click)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.export_tables_view)
            .event(nwg::Event::OnListViewDoubleClick)
            .handler(AppWindow::on_export_tables_view_double_click)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.export_tables_view)
            .event(nwg::Event::OnListViewFocusLost)
            .handler(AppWindow::on_export_tables_view_focus_lost)
            .build(&mut self.events)?;

        ui::event_builder()
            .control(&c.export_dbnames_combo)
            .event(nwg::Event::OnComboBoxClosed)
            .handler(AppWindow::on_export_dbname_changed)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.export_dbnames_reload_button)
            .event(nwg::Event::OnButtonClick)
            .handler(AppWindow::open_load_dbnames_dialog)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.export_dest_dir_button)
            .event(nwg::Event::OnButtonClick)
            .handler(AppWindow::on_choose_export_dest_dir)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.export_run_button)
            .event(nwg::Event::OnButtonClick)
            .handler(AppWindow::open_export_dialog)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.export_close_button)
            .event(nwg::Event::OnButtonClick)
            .handler(AppWindow::close)
            .build(&mut self.events)?;

        ui::event_builder()
            .control(&c.import_tables_mark_all_button)
            .event(nwg::Event::OnButtonClick)
            .handler(AppWindow::on_import_mark_all_button)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.import_tables_clear_button)
            .event(nwg::Event::OnButtonClick)
            .handler(AppWindow::on_import_clear_button)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.import_tables_copy_name_button)
            .event(nwg::Event::OnButtonClick)
            .handler(AppWindow::on_import_copy_name_button)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.import_tables_filter_button)
            .event(nwg::Event::OnButtonClick)
            .handler(AppWindow::on_import_filter_button)
            .build(&mut self.events)?;

        ui::event_builder()
            .control(&c.import_tables_view)
            .event(nwg::Event::OnListViewColumnClick)
            .handler(AppWindow::on_import_tables_view_sort)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.import_tables_view)
            .event(nwg::Event::OnListViewClick)
            .handler(AppWindow::on_import_tables_view_click)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.import_tables_view)
            .event(nwg::Event::OnListViewDoubleClick)
            .handler(AppWindow::on_import_tables_view_double_click)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.import_tables_view)
            .event(nwg::Event::OnListViewFocusLost)
            .handler(AppWindow::on_import_tables_view_focus_lost)
            .build(&mut self.events)?;

        ui::event_builder()
            .control(&c.import_dbnames_reload_button)
            .event(nwg::Event::OnButtonClick)
            .handler(AppWindow::open_load_dbnames_dialog)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.import_file_button)
            .event(nwg::Event::OnButtonClick)
            .handler(AppWindow::on_choose_import_file)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.import_run_button)
            .event(nwg::Event::OnButtonClick)
            .handler(AppWindow::open_import_dialog)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.import_close_button)
            .event(nwg::Event::OnButtonClick)
            .handler(AppWindow::close)
            .build(&mut self.events)?;

        ui::event_builder()
            .control(&c.about_notice.notice)
            .event(nwg::Event::OnNotice)
            .handler(AppWindow::await_about_dialog)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.connect_notice.notice)
            .event(nwg::Event::OnNotice)
            .handler(AppWindow::await_connect_dialog)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.load_dbnames_notice.notice)
            .event(nwg::Event::OnNotice)
            .handler(AppWindow::await_load_dbnames_dialog)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.load_tables_notice.notice)
            .event(nwg::Event::OnNotice)
            .handler(AppWindow::await_load_tables_dialog)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.export_notice.notice)
            .event(nwg::Event::OnNotice)
            .handler(AppWindow::await_export_dialog)
            .build(&mut self.events)?;
        ui::event_builder()
            .control(&c.import_notice.notice)
            .event(nwg::Event::OnNotice)
            .handler(AppWindow::await_import_dialog)
            .build(&mut self.events)?;

        Ok(())
    }
}
