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

#[derive(Default, Clone)]
pub struct TableWithSize {
    pub schema: String,
    pub table: String,
    pub size_bytes: u64,
    pub import: bool,
}

impl TableWithSize {
    pub fn new(zip_entry_name: &str, size_bytes: u64) -> Result<Self, TransferError> {
        let parts = zip_entry_name.split(".").collect::<Vec<&str>>();
        if !(4 == parts.len() && "bcp" == parts[2] && ("gz" == parts[3] || "zstd" == parts[3])) {
            return Err(TransferError::from_string(format!(
                "Unexpected ZIP entry name: {}", zip_entry_name)));
        }
        Ok(Self {
            schema: parts[0].to_string(),
            table: parts[1].to_string(),
            size_bytes,
            import: false
        })
    }
}
