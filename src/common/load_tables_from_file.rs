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

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use human_bytes::human_bytes;
use zip::ZipArchive;

pub fn load_tables_from_file<P: Fn(&str)->()>(progress_fun: &P, file_path: &str) -> Result<Vec<TableWithSize>, TransferError> {
    if !Path::new(&file_path).exists() {
        return Err(TransferError::from_string(format!(
            "Specified file is not found, path: {}", file_path)));
    }
    let file = match File::open(&file_path) {
        Ok(file) => file,
        Err(e) => return Err(TransferError::from_string(format!(
            "Error opening file, path: {}, message: {}", file_path, e.to_string())))
    };
    let reader = BufReader::new(file);
    let mut zip = match ZipArchive::new(reader) {
        Ok(zip) => zip,
        Err(e) => return Err(TransferError::from_string(format!(
            "Error opening ZIP file, path: {}, message: {}", file_path, e.to_string())))
    };
    let mut tables: Vec<TableWithSize> = Vec::new();
    progress_fun("Loading tables ...");
    for i in 0..zip.len() {
        let entry = match zip.by_index(i) {
            Ok(entry) => entry,
            Err(e) => return Err(TransferError::from_string(format!(
                "Error opening ZIP file, path: {}, message: {}", file_path, e.to_string())))
        };
        if entry.name().ends_with(".bcp.gz") || entry.name().ends_with(".bcp.zstd") {
            let name_parts = entry.name().split("/").collect::<Vec<&str>>();
            let name = name_parts[name_parts.len() - 1];
            let tab = TableWithSize::new(name, entry.size())?;
            progress_fun(&format!("{}.{} {}", &tab.schema, &tab.table, human_bytes(tab.size_bytes as f64)));
            tables.push(tab);
        }
    };

    Ok(tables)
}
