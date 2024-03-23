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
pub(super) struct AppWindowLayout {
    tabs_container_layout: nwg::FlexboxLayout,

    export_tab_layout: nwg::FlexboxLayout,
    export_tables_top_layout: nwg::FlexboxLayout,
    export_tables_view_layout: nwg::FlexboxLayout,
    export_dbnames_layout: nwg::FlexboxLayout,
    export_dest_dir_layout: nwg::FlexboxLayout,
    export_filename_layout: nwg::FlexboxLayout,
    export_buttons_layout: nwg::FlexboxLayout,

    import_tab_layout: nwg::FlexboxLayout,
    import_tables_top_layout: nwg::FlexboxLayout,
    import_tables_view_layout: nwg::FlexboxLayout,
    import_dbnames_layout: nwg::FlexboxLayout,
    import_file_layout: nwg::FlexboxLayout,
    import_buttons_layout: nwg::FlexboxLayout,
}

impl ui::Layout<AppWindowControls> for AppWindowLayout {

    fn build(&self, c: &AppWindowControls) -> Result<(), nwg::NwgError> {

        nwg::FlexboxLayout::builder()
            .parent(&c.import_tab)
            .flex_direction(ui::FlexDirection::Row)
            .auto_spacing(None)
            .child(&c.export_tables_mark_all_button)
            .child_size(ui::size_builder()
                .width_button_normal()
                .height_button()
                .build())
            .child(&c.export_tables_clear_button)
            .child_size(ui::size_builder()
                .width_button_normal()
                .height_button()
                .build())
            .child_margin(ui::margin_builder()
                .start_pt(5)
                .build())
            .child(&c.export_tables_copy_name_button)
            .child_size(ui::size_builder()
                .width_pt(80)
                .height_button()
                .build())
            .child_margin(ui::margin_builder()
                .start_pt(5)
                .build())
            .child(&c.export_tables_filter_input)
            .child_margin(ui::margin_builder()
                .start_pt(5)
                .build())
            .child_flex_grow(1.0)
            .child(&c.export_tables_filter_button)
            .child_size(ui::size_builder()
                .width_button_normal()
                .height_button()
                .build())
            .child_margin(ui::margin_builder()
                .start_pt(5)
                .build())
            .build_partial(&self.export_tables_top_layout)?;

        nwg::FlexboxLayout::builder()
            .parent(&c.export_tab)
            .flex_direction(ui::FlexDirection::Row)
            .child(&c.export_tables_view)
            .child_flex_grow(1.0)
            .auto_spacing(None)
            .build_partial(&self.export_tables_view_layout)?;

        nwg::FlexboxLayout::builder()
            .parent(&c.export_tab)
            .flex_direction(ui::FlexDirection::Row)
            .auto_spacing(None)
            .child(&c.export_dbnames_label)
            .child_size(ui::size_builder()
                .width_label_normal()
                .height_input_form_row()
                .build())
            .child(&c.export_dbnames_combo)
            .child_margin(ui::margin_builder()
                .start_pt(5)
                .build())
            .child_flex_grow(1.0)
            .child(&c.export_dbnames_reload_button)
            .child_size(ui::size_builder()
                .width_button_normal()
                .height_button()
                .build())
            .child_margin(ui::margin_builder()
                .start_pt(5)
                .build())
            .build_partial(&self.export_dbnames_layout)?;

        nwg::FlexboxLayout::builder()
            .parent(&c.export_tab)
            .flex_direction(ui::FlexDirection::Row)
            .auto_spacing(None)
            .child(&c.export_dest_dir_label)
            .child_size(ui::size_builder()
                .width_label_normal()
                .height_input_form_row()
                .build())
            .child(&c.export_dest_dir_input)
            .child_margin(ui::margin_builder()
                .start_pt(5)
                .build())
            .child_flex_grow(1.0)
            .child(&c.export_dest_dir_button)
            .child_size(ui::size_builder()
                .width_button_normal()
                .height_button()
                .build())
            .child_margin(ui::margin_builder()
                .start_pt(5)
                .build())
            .build_partial(&self.export_dest_dir_layout)?;

        nwg::FlexboxLayout::builder()
            .parent(&c.export_tab)
            .flex_direction(ui::FlexDirection::Row)
            .auto_spacing(None)
            .child(&c.export_filename_label)
            .child_size(ui::size_builder()
                .width_label_normal()
                .height_input_form_row()
                .build())
            .child(&c.export_filename_input)
            .child_margin(ui::margin_builder()
                .start_pt(5)
                .build())
            .child_flex_grow(1.0)
            .build_partial(&self.export_filename_layout)?;

        nwg::FlexboxLayout::builder()
            .parent(&c.export_tab)
            .flex_direction(ui::FlexDirection::Row)
            .justify_content(ui::JustifyContent::FlexEnd)
            .auto_spacing(None)
            .child(&c.export_run_button)
            .child_size(ui::size_builder()
                .width_button_xwide()
                .height_button()
                .build())
            .child_margin(ui::margin_builder()
                .top_pt(10)
                .build())
            .child(&c.export_close_button)
            .child_size(ui::size_builder()
                .width_button_normal()
                .height_button()
                .build())
            .child_margin(ui::margin_builder()
                .start_pt(5)
                .top_pt(10)
                .build())
            .build_partial(&self.export_buttons_layout)?;

        nwg::FlexboxLayout::builder()
            .parent(&c.export_tab)
            .flex_direction(ui::FlexDirection::Column)
            .child_layout(&self.export_tables_top_layout)
            .child_layout(&self.export_tables_view_layout)
            .child_flex_grow(1.0)
            .child_layout(&self.export_dbnames_layout)
            .child_layout(&self.export_dest_dir_layout)
            .child_layout(&self.export_filename_layout)
            .child_layout(&self.export_buttons_layout)
            .build(&self.export_tab_layout)?;

        // import

        nwg::FlexboxLayout::builder()
            .parent(&c.import_tab)
            .flex_direction(ui::FlexDirection::Row)
            .auto_spacing(None)
            .child(&c.import_tables_mark_all_button)
            .child_size(ui::size_builder()
                .width_button_normal()
                .height_button()
                .build())
            .child(&c.import_tables_clear_button)
            .child_size(ui::size_builder()
                .width_button_normal()
                .height_button()
                .build())
            .child_margin(ui::margin_builder()
                .start_pt(5)
                .build())
            .child(&c.import_tables_copy_name_button)
            .child_size(ui::size_builder()
                .width_pt(80)
                .height_button()
                .build())
            .child_margin(ui::margin_builder()
                .start_pt(5)
                .build())
            .child(&c.import_tables_filter_input)
            .child_margin(ui::margin_builder()
                .start_pt(5)
                .build())
            .child_flex_grow(1.0)
            .child(&c.import_tables_filter_button)
            .child_size(ui::size_builder()
                .width_button_normal()
                .height_button()
                .build())
            .child_margin(ui::margin_builder()
                .start_pt(5)
                .build())
            .build_partial(&self.import_tables_top_layout)?;

        nwg::FlexboxLayout::builder()
            .parent(&c.import_tab)
            .flex_direction(ui::FlexDirection::Row)
            .child(&c.import_tables_view)
            .child_flex_grow(1.0)
            .auto_spacing(None)
            .build_partial(&self.import_tables_view_layout)?;

        nwg::FlexboxLayout::builder()
            .parent(&c.import_tab)
            .flex_direction(ui::FlexDirection::Row)
            .auto_spacing(None)
            .child(&c.import_dbnames_label)
            .child_size(ui::size_builder()
                .width_label_normal()
                .height_input_form_row()
                .build())
            .child(&c.import_dbnames_combo)
            .child_margin(ui::margin_builder()
                .start_pt(5)
                .build())
            .child_flex_grow(1.0)
            .child(&c.import_dbnames_reload_button)
            .child_size(ui::size_builder()
                .width_button_normal()
                .height_button()
                .build())
            .child_margin(ui::margin_builder()
                .start_pt(5)
                .build())
            .build_partial(&self.import_dbnames_layout)?;

        nwg::FlexboxLayout::builder()
            .parent(&c.import_tab)
            .flex_direction(ui::FlexDirection::Row)
            .auto_spacing(None)
            .child(&c.import_file_label)
            .child_size(ui::size_builder()
                .width_label_normal()
                .height_input_form_row()
                .build())
            .child(&c.import_file_input)
            .child_margin(ui::margin_builder()
                .start_pt(5)
                .build())
            .child_flex_grow(1.0)
            .child(&c.import_file_button)
            .child_size(ui::size_builder()
                .width_button_normal()
                .height_button()
                .build())
            .child_margin(ui::margin_builder()
                .start_pt(5)
                .build())
            .build_partial(&self.import_file_layout)?;

        nwg::FlexboxLayout::builder()
            .parent(&c.import_tab)
            .flex_direction(ui::FlexDirection::Row)
            .justify_content(ui::JustifyContent::FlexEnd)
            .auto_spacing(None)
            .child(&c.import_run_button)
            .child_size(ui::size_builder()
                .width_button_xwide()
                .height_button()
                .build())
            .child_margin(ui::margin_builder()
                .top_pt(10)
                .build())
            .child(&c.import_close_button)
            .child_size(ui::size_builder()
                .width_button_normal()
                .height_button()
                .build())
            .child_margin(ui::margin_builder()
                .start_pt(5)
                .top_pt(10)
                .build())
            .build_partial(&self.import_buttons_layout)?;

        nwg::FlexboxLayout::builder()
            .parent(&c.import_tab)
            .flex_direction(ui::FlexDirection::Column)
            .child_layout(&self.import_tables_top_layout)
            .child_layout(&self.import_tables_view_layout)
            .child_flex_grow(1.0)
            .child_layout(&self.import_dbnames_layout)
            .child_layout(&self.import_file_layout)
            .child_layout(&self.import_buttons_layout)
            .build(&self.import_tab_layout)?;

        // tabs container

        nwg::FlexboxLayout::builder()
            .parent(&c.window)
            .flex_direction(ui::FlexDirection::Column)
            .child(&c.tabs_container)
            .child_margin(ui::margin_builder()
                .start_default()
                .top_default()
                .end_default()
                .bottom_pt(30)
                .build())
            .build(&self.tabs_container_layout)?;

        Ok(())
    }
}
