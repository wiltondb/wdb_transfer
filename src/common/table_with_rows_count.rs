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

#[derive(Default, Clone)]
pub struct TableWithRowsCount {
    pub schema: String,
    pub table: String,
    pub row_count: i64,
    pub export: bool,
}

impl TableWithRowsCount {
    pub fn new(schema: &str, table: &str, row_count: i64) -> Self {
        Self {
            schema: schema.to_string(),
            table: table.to_string(),
            row_count,
            export: false
        }
    }

    pub fn set_export(&mut self, export: bool) {
        self.export = export;
    }
}
