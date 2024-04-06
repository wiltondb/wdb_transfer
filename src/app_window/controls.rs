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

const COLOR_WHITE: [u8; 3] = [255, 255, 255];

#[derive(Default)]
pub(super) struct AppWindowControls {
    layout: AppWindowLayout,

    pub(super) font_normal: nwg::Font,
    pub(super) font_small: nwg::Font,

    pub(super) icon: nwg::Icon,
    pub(super) window: nwg::Window,

    pub(super) file_menu: nwg::Menu,
    pub(super) file_connect_menu_item: nwg::MenuItem,
    pub(super) file_exit_menu_item: nwg::MenuItem,
    pub(super) help_menu: nwg::Menu,
    pub(super) help_about_menu_item: nwg::MenuItem,
    pub(super) help_website_menu_item: nwg::MenuItem,

    pub(super) tabs_container: nwg::TabsContainer,
    pub(super) export_tab: nwg::Tab,
    pub(super) import_tab: nwg::Tab,

    pub(super) export_tables_mark_all_button: nwg::Button,
    pub(super) export_tables_clear_button: nwg::Button,
    pub(super) export_tables_copy_name_button: nwg::Button,
    pub(super) export_tables_filter_input: nwg::TextInput,
    pub(super) export_tables_filter_button: nwg::Button,
    pub(super) export_tables_view: nwg::ListView,
    pub(super) export_dbnames_label: nwg::Label,
    pub(super) export_dbnames_combo: nwg::ComboBox<String>,
    pub(super) export_dbnames_reload_button: nwg::Button,
    pub(super) export_dest_dir_label: nwg::Label,
    pub(super) export_dest_dir_input: nwg::TextInput,
    pub(super) export_dest_dir_button: nwg::Button,
    pub(super) export_dest_dir_chooser: nwg::FileDialog,
    pub(super) export_filename_label: nwg::Label,
    pub(super) export_filename_input: nwg::TextInput,
    pub(super) export_run_button: nwg::Button,
    pub(super) export_close_button: nwg::Button,

    pub(super) import_tables_mark_all_button: nwg::Button,
    pub(super) import_tables_clear_button: nwg::Button,
    pub(super) import_tables_copy_name_button: nwg::Button,
    pub(super) import_tables_filter_input: nwg::TextInput,
    pub(super) import_tables_filter_button: nwg::Button,
    pub(super) import_tables_view: nwg::ListView,
    pub(super) import_dbnames_label: nwg::Label,
    pub(super) import_dbnames_combo: nwg::ComboBox<String>,
    pub(super) import_dbnames_reload_button: nwg::Button,
    pub(super) import_file_label: nwg::Label,
    pub(super) import_file_input: nwg::TextInput,
    pub(super) import_file_button: nwg::Button,
    pub(super) import_file_chooser: nwg::FileDialog,
    pub(super) import_run_button: nwg::Button,
    pub(super) import_close_button: nwg::Button,

    pub(super) status_bar: nwg::StatusBar,

    pub(super) about_notice: ui::SyncNotice,
    pub(super) connect_notice: ui::SyncNotice,
    pub(super) load_dbnames_notice: ui::SyncNotice,
    pub(super) load_tables_notice: ui::SyncNotice,
    pub(super) export_notice: ui::SyncNotice,
    pub(super) import_notice: ui::SyncNotice,
}

impl ui::Controls for AppWindowControls {
    fn build(&mut self) -> Result<(), nwg::NwgError> {
        // fonts
        nwg::Font::builder()
            .size(ui::font_size_builder()
                .normal()
                .build())
            .build(&mut self.font_normal)?;
        nwg::Font::builder()
            .size(ui::font_size_builder()
                .small()
                .build())
            .build(&mut self.font_small)?;

        // window

        nwg::Icon::builder()
            .source_embed(Some(&nwg::EmbedResource::load(None)
                .expect("Error loading embedded resource")))
            .source_embed_id(2)
            .build(&mut self.icon)?;

        nwg::Window::builder()
            .size((520, 480))
            .icon(Some(&self.icon))
            .center(true)
            .title("WiltonDB Data Transfer Tool")
            .build(&mut self.window)?;

        // menu

        nwg::Menu::builder()
            .parent(&self.window)
            .text("File")
            .build(&mut self.file_menu)?;
        nwg::MenuItem::builder()
            .parent(&self.file_menu)
            .text("DB Connection")
            .build(&mut self.file_connect_menu_item)?;
        nwg::MenuItem::builder()
            .parent(&self.file_menu)
            .text("Exit")
            .build(&mut self.file_exit_menu_item)?;

        nwg::Menu::builder()
            .parent(&self.window)
            .text("Help")
            .build(&mut self.help_menu)?;
        nwg::MenuItem::builder()
            .parent(&self.help_menu)
            .text("About")
            .build(&mut self.help_about_menu_item)?;
        nwg::MenuItem::builder()
            .parent(&self.help_menu)
            .text("Website")
            .build(&mut self.help_website_menu_item)?;

        // tabs

        nwg::TabsContainer::builder()
            .font(Some(&self.font_normal))
            .parent(&self.window)
            .build(&mut self.tabs_container)?;
        nwg::Tab::builder()
            .text("Export")
            .parent(&self.tabs_container)
            .build(&mut self.export_tab)?;
        nwg::Tab::builder()
            .text("Import")
            .parent(&self.tabs_container)
            .build(&mut self.import_tab)?;

        // export top form

        nwg::Button::builder()
            .parent(&self.export_tab)
            .text("Mark all")
            .font(Some(&self.font_normal))
            .build(&mut self.export_tables_mark_all_button)?;
        nwg::Button::builder()
            .parent(&self.export_tab)
            .text("Clear all")
            .font(Some(&self.font_normal))
            .build(&mut self.export_tables_clear_button)?;
        nwg::Button::builder()
            .parent(&self.export_tab)
            .text("Copy name")
            .font(Some(&self.font_normal))
            .enabled(false)
            .build(&mut self.export_tables_copy_name_button)?;
        nwg::TextInput::builder()
            .parent(&self.export_tab)
            .placeholder_text(Some("Table name with '*' and '?'"))
            .font(Some(&self.font_normal))
            .build(&mut self.export_tables_filter_input)?;
        nwg::Button::builder()
            .parent(&self.export_tab)
            .text("Search")
            .font(Some(&self.font_normal))
            .build(&mut self.export_tables_filter_button)?;

        // export tables view

        nwg::ListView::builder()
            .parent(&self.export_tab)
            .item_count(10)
            .list_style(nwg::ListViewStyle::Detailed)
            .focus(true)
            .flags(nwg::ListViewFlags::VISIBLE | nwg::ListViewFlags::TAB_STOP |
                nwg::ListViewFlags::SINGLE_SELECTION | nwg::ListViewFlags::ALWAYS_SHOW_SELECTION)
            .ex_flags(nwg::ListViewExFlags::GRID | nwg::ListViewExFlags::FULL_ROW_SELECT)
            .build(&mut self.export_tables_view)?;
        self.export_tables_view.set_headers_enabled(true);
        self.export_tables_view.insert_column(nwg::InsertListViewColumn{
            index: Some(0),
            fmt: Some(nwg::ListViewColumnFlags::LEFT),
            width: Some(50),
            text: Some("Export".to_string())
        });
        self.export_tables_view.set_column_sort_arrow(0, Some(nwg::ListViewColumnSortArrow::Down));
        self.export_tables_view.insert_column(nwg::InsertListViewColumn{
            index: Some(1),
            fmt: Some(nwg::ListViewColumnFlags::LEFT),
            width: Some(80),
            text: Some("Schema".to_string())
        });
        self.export_tables_view.set_column_sort_arrow(1, Some(nwg::ListViewColumnSortArrow::Down));
        self.export_tables_view.insert_column(nwg::InsertListViewColumn{
            index: Some(2),
            fmt: Some(nwg::ListViewColumnFlags::LEFT),
            width: Some(240),
            text: Some("Table name".to_string())
        });
        self.export_tables_view.set_column_sort_arrow(2, Some(nwg::ListViewColumnSortArrow::Down));
        self.export_tables_view.insert_column(nwg::InsertListViewColumn{
            index: Some(3),
            fmt: Some(nwg::ListViewColumnFlags::LEFT),
            width: Some(80),
            text: Some("Rows est.".to_string())
        });
        self.export_tables_view.set_column_sort_arrow(3, Some(nwg::ListViewColumnSortArrow::Down));

        // export bottom form

        nwg::Label::builder()
            .parent(&self.export_tab)
            .text("Database:")
            .font(Some(&self.font_normal))
            .background_color(Some(COLOR_WHITE))
            .h_align(nwg::HTextAlign::Left)
            .build(&mut self.export_dbnames_label)?;
        nwg::ComboBox::builder()
            .parent(&self.export_tab)
            .font(Some(&self.font_normal))
            .build(&mut self.export_dbnames_combo)?;
        nwg::Button::builder()
            .parent(&self.export_tab)
            .text("Reload")
            .font(Some(&self.font_normal))
            .build(&mut self.export_dbnames_reload_button)?;
        nwg::Label::builder()
            .parent(&self.export_tab)
            .text("Destination dir.:")
            .font(Some(&self.font_normal))
            .background_color(Some(COLOR_WHITE))
            .h_align(nwg::HTextAlign::Left)
            .build(&mut self.export_dest_dir_label)?;
        nwg::TextInput::builder()
            .parent(&self.export_tab)
            .font(Some(&self.font_normal))
            .text(&std::env::var("USERPROFILE").unwrap_or(String::new()))
            .build(&mut self.export_dest_dir_input)?;
        nwg::Button::builder()
            .parent(&self.export_tab)
            .text("Choose")
            .font(Some(&self.font_normal))
            .build(&mut self.export_dest_dir_button)?;
        nwg::FileDialog::builder()
            .title("Choose destination directory")
            .action(nwg::FileDialogAction::OpenDirectory)
            .build(&mut self.export_dest_dir_chooser)?;
        nwg::Label::builder()
            .parent(&self.export_tab)
            .text("Export file name:")
            .font(Some(&self.font_normal))
            .background_color(Some(COLOR_WHITE))
            .h_align(nwg::HTextAlign::Left)
            .build(&mut self.export_filename_label)?;
        nwg::TextInput::builder()
            .parent(&self.export_tab)
            .font(Some(&self.font_normal))
            .build(&mut self.export_filename_input)?;

        // export buttons

        nwg::Button::builder()
            .parent(&self.export_tab)
            .text("Run data export")
            .font(Some(&self.font_normal))
            .enabled(false)
            .build(&mut self.export_run_button)?;
        nwg::Button::builder()
            .parent(&self.export_tab)
            .text("Close")
            .font(Some(&self.font_normal))
            .build(&mut self.export_close_button)?;

        // import top form

        nwg::Button::builder()
            .parent(&self.import_tab)
            .text("Mark all")
            .font(Some(&self.font_normal))
            .build(&mut self.import_tables_mark_all_button)?;
        nwg::Button::builder()
            .parent(&self.import_tab)
            .text("Clear all")
            .font(Some(&self.font_normal))
            .build(&mut self.import_tables_clear_button)?;
        nwg::Button::builder()
            .parent(&self.import_tab)
            .text("Copy name")
            .font(Some(&self.font_normal))
            .enabled(false)
            .build(&mut self.import_tables_copy_name_button)?;
        nwg::TextInput::builder()
            .parent(&self.import_tab)
            .placeholder_text(Some("Table name with '*' and '?'"))
            .font(Some(&self.font_normal))
            .build(&mut self.import_tables_filter_input)?;
        nwg::Button::builder()
            .parent(&self.import_tab)
            .text("Search")
            .font(Some(&self.font_normal))
            .build(&mut self.import_tables_filter_button)?;

        // import tables view

        nwg::ListView::builder()
            .parent(&self.import_tab)
            .item_count(10)
            .list_style(nwg::ListViewStyle::Detailed)
            .focus(true)
            .flags(nwg::ListViewFlags::VISIBLE | nwg::ListViewFlags::TAB_STOP |
                nwg::ListViewFlags::SINGLE_SELECTION | nwg::ListViewFlags::ALWAYS_SHOW_SELECTION)
            .ex_flags(nwg::ListViewExFlags::GRID | nwg::ListViewExFlags::FULL_ROW_SELECT)
            .build(&mut self.import_tables_view)?;
        self.import_tables_view.set_headers_enabled(true);
        self.import_tables_view.insert_column(nwg::InsertListViewColumn{
            index: Some(0),
            fmt: Some(nwg::ListViewColumnFlags::LEFT),
            width: Some(50),
            text: Some("Import".to_string())
        });
        self.import_tables_view.set_column_sort_arrow(0, Some(nwg::ListViewColumnSortArrow::Down));
        self.import_tables_view.insert_column(nwg::InsertListViewColumn{
            index: Some(1),
            fmt: Some(nwg::ListViewColumnFlags::LEFT),
            width: Some(80),
            text: Some("Schema".to_string())
        });
        self.import_tables_view.set_column_sort_arrow(1, Some(nwg::ListViewColumnSortArrow::Down));
        self.import_tables_view.insert_column(nwg::InsertListViewColumn{
            index: Some(2),
            fmt: Some(nwg::ListViewColumnFlags::LEFT),
            width: Some(240),
            text: Some("Table name".to_string())
        });
        self.import_tables_view.set_column_sort_arrow(2, Some(nwg::ListViewColumnSortArrow::Down));
        self.import_tables_view.insert_column(nwg::InsertListViewColumn{
            index: Some(3),
            fmt: Some(nwg::ListViewColumnFlags::LEFT),
            width: Some(80),
            text: Some("Size comp.".to_string())
        });
        self.import_tables_view.set_column_sort_arrow(3, Some(nwg::ListViewColumnSortArrow::Down));

        // import bottom form

        nwg::Label::builder()
            .parent(&self.import_tab)
            .text("Database:")
            .font(Some(&self.font_normal))
            .background_color(Some(COLOR_WHITE))
            .h_align(nwg::HTextAlign::Left)
            .build(&mut self.import_dbnames_label)?;
        nwg::ComboBox::builder()
            .parent(&self.import_tab)
            .font(Some(&self.font_normal))
            .build(&mut self.import_dbnames_combo)?;
        nwg::Button::builder()
            .parent(&self.import_tab)
            .text("Reload")
            .font(Some(&self.font_normal))
            .build(&mut self.import_dbnames_reload_button)?;
        nwg::Label::builder()
            .parent(&self.import_tab)
            .text("Import file:")
            .font(Some(&self.font_normal))
            .background_color(Some(COLOR_WHITE))
            .h_align(nwg::HTextAlign::Left)
            .build(&mut self.import_file_label)?;
        nwg::TextInput::builder()
            .parent(&self.import_tab)
            .font(Some(&self.font_normal))
            .text("")
            .build(&mut self.import_file_input)?;
        nwg::Button::builder()
            .parent(&self.import_tab)
            .text("Choose")
            .font(Some(&self.font_normal))
            .build(&mut self.import_file_button)?;
        nwg::FileDialog::builder()
            .title("Choose import file")
            .action(nwg::FileDialogAction::Open)
            .build(&mut self.import_file_chooser)?;

        // import buttons

        nwg::Button::builder()
            .parent(&self.import_tab)
            .text("Run data import")
            .font(Some(&self.font_normal))
            .enabled(false)
            .build(&mut self.import_run_button)?;
        nwg::Button::builder()
            .parent(&self.import_tab)
            .text("Close")
            .font(Some(&self.font_normal))
            .build(&mut self.import_close_button)?;

        // other

        nwg::StatusBar::builder()
            .parent(&self.window)
            .font(Some(&self.font_small))
            .build(&mut self.status_bar)?;

        ui::notice_builder()
            .parent(&self.window)
            .build(&mut self.about_notice)?;
        ui::notice_builder()
            .parent(&self.window)
            .build(&mut self.connect_notice)?;
        ui::notice_builder()
            .parent(&self.window)
            .build(&mut self.load_dbnames_notice)?;
        ui::notice_builder()
            .parent(&self.window)
            .build(&mut self.load_tables_notice)?;
        ui::notice_builder()
            .parent(&self.window)
            .build(&mut self.export_notice)?;
        ui::notice_builder()
            .parent(&self.window)
            .build(&mut self.import_notice)?;

        self.layout.build(&self)?;

        Ok(())
    }

    fn update_tab_order(&self) {
        ui::tab_order_builder()
            .control(&self.export_tables_mark_all_button)
            .control(&self.export_tables_clear_button)
            .control(&self.export_tables_copy_name_button)
            .control(&self.export_tables_filter_input)
            .control(&self.export_tables_filter_button)
            .control(&self.export_dbnames_combo)
            .control(&self.export_dbnames_reload_button)
            .control(&self.export_dest_dir_input)
            .control(&self.export_dest_dir_button)
            .control(&self.export_filename_input)
            .control(&self.export_run_button)
            .control(&self.export_close_button)
            .build();

        ui::tab_order_builder()
            .control(&self.import_tables_mark_all_button)
            .control(&self.import_tables_clear_button)
            .control(&self.import_tables_copy_name_button)
            .control(&self.import_tables_filter_input)
            .control(&self.import_tables_filter_button)
            .control(&self.import_dbnames_combo)
            .control(&self.import_file_input)
            .control(&self.import_file_button)
            .control(&self.import_run_button)
            .control(&self.import_close_button)
            .build();
    }
}
