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

use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::os::windows::process::CommandExt;
use std::path::Path;

use regex::Regex;

#[derive(Default, Clone)]
pub struct ExportArgs {
    pub dbname: String,
    pub tables: Vec<TableWithRowsCount>,
    pub parent_dir: String,
    pub dest_filename: String,
}

#[derive(Default)]
pub struct ExportResult {
    pub error: String
}

impl ExportResult {
    pub(super) fn success() -> Self {
        Self {
            error: Default::default()
        }
    }

    pub(super) fn failure(error: String) -> Self {
        Self {
            error
        }
    }
}

fn strip_collation_from_format_file(dest_dir: &str, format_filename: &str) -> Result<(), TransferError> {
    let format_path = Path::new(dest_dir).join(&format_filename);
    let bytes = match fs::read(&format_path) {
        Ok(bytes) => bytes,
        Err(e) => return Err(TransferError::from_string(format!(
            "Format file post-processing error: {}", e)))
    };
    let codepoints: Vec<u16> = bytes
        .chunks_exact(2)
        .into_iter()
        .map(|a| u16::from_le_bytes([a[0], a[1]]))
        .collect();
    let text = match String::from_utf16(&codepoints) {
        Ok(text) => text,
        Err(e) => return Err(TransferError::from_string(format!(
            "Format file post-processing error: {}", e)))
    };
    let re = match Regex::new("(?i)\\sCOLLATION=\"\\w+\"") {
        Ok(re) => re,
        Err(e) => return Err(TransferError::from_string(format!(
            "Format file post-processing error: {}", e)))
    };
    let replaced = re.replace_all(&text, " COLLATION=\"\"").to_string();
    match fs::write(&format_path, replaced) {
        Ok(_) => { },
        Err(e) => return Err(TransferError::from_string(format!(
            "Format file post-processing error: {}", e)))
    };

    Ok(())
}

fn run_bcp_format<P: Fn(&str)->()>(progress_fun: &P, cc: &TdsConnConfig, dest_dir: &str,
                  dbname: &str, schema: &str, table: &str) -> Result<String, TransferError> {
    progress_fun(&format!("Creating bcp format file: {}.{}", schema, table));
    let format_filename = format!("{}.{}.xml", schema, table);
    let mut args: Vec<String> = vec!(
        format!("[{}].[{}].[{}]", dbname, schema, table),
        "format".to_string(),
        "nul".to_string(),
        "-f".to_string(),
        format_filename.clone(),
        "-x".to_string(),
        "-n".to_string(),
        "-k".to_string(),
        "-K".to_string(),
        "ReadOnly".to_string(),
        "-S".to_string(),
    );
    if cc.use_named_instance {
        args.push(format!("tcp:{}\\{}", &cc.hostname, &cc.instance));
    } else {
        args.push(format!("tcp:{},{}", &cc.hostname, &cc.port));
    }
    if cc.use_win_auth {
        args.push("-T".to_string());
    } else {
        args.push("-U".to_string());
        args.push(cc.username.clone());
        args.push("-P".to_string());
        args.push(cc.password.clone());
    }
    let cmd = duct::cmd("bcp.exe", args)
        .dir(dest_dir)
        .stdin_null()
        .stderr_to_stdout()
        .stdout_capture()
        .before_spawn(|pcmd| {
            // create no window
            let _ = pcmd.creation_flags(0x08000000);
            Ok(())
        });
    let reader = match cmd.reader() {
        Ok(reader) => reader,
        Err(e) => return Err(TransferError::from_bcp_error(
            "bcp process spawn failure", e.to_string()))
    };
    let mut buf_reader = BufReader::new(&reader);
    loop {
        let mut buf = vec!();
        match buf_reader.read_until(b'\n', &mut buf) {
            Ok(len) => {
                if 0 == len {
                    break;
                }
                if buf.len() >= 2 {
                    let ln = String::from_utf8_lossy(&buf[0..buf.len() - 2]);
                    progress_fun(&ln);
                }
            },
            Err(e) => return Err(TransferError::from_bcp_error(
                "bcp process failure", e.to_string()))
        };
    };
    match reader.try_wait() {
        Ok(opt) => match opt {
            Some(_) => { },
            None => return Err(TransferError::from_str("bcp process failure"))
        },
        Err(e) => return Err(TransferError::from_bcp_error(
            "bcp process failure", e.to_string()))
    }

    strip_collation_from_format_file(dest_dir, &format_filename)?;

    Ok(format_filename)
}

fn run_bcp_data<P: Fn(&str)->()>(progress_fun: &P, cc: &TdsConnConfig, dest_dir: &str,
                dbname: &str, schema: &str, table: &str, format_filename: &str) -> Result<String, TransferError> {
    progress_fun(&format!("Exporting data: {}.{}", schema, table));
    let data_filename = format!("{}.{}.bcp", schema, table);
    let mut args: Vec<String> = vec!(
        format!("[{}].[{}].[{}]", dbname, schema, table),
        "out".to_string(),
        data_filename.clone(),
        "-f".to_string(),
        format_filename.to_string(),
        "-k".to_string(),
        "-K".to_string(),
        "ReadOnly".to_string(),
        "-S".to_string(),
    );
    if cc.use_named_instance {
        args.push(format!("tcp:{}\\{}", &cc.hostname, &cc.instance));
    } else {
        args.push(format!("tcp:{},{}", &cc.hostname, &cc.port));
    }
    if cc.use_win_auth {
        args.push("-T".to_string());
    } else {
        args.push("-U".to_string());
        args.push(cc.username.clone());
        args.push("-P".to_string());
        args.push(cc.password.clone());
    }
    let cmd = duct::cmd("bcp.exe", args)
        .dir(dest_dir)
        .stdin_null()
        .stderr_to_stdout()
        .stdout_capture()
        .before_spawn(|pcmd| {
            // create no window
            let _ = pcmd.creation_flags(0x08000000);
            Ok(())
        });
    let reader = match cmd.reader() {
        Ok(reader) => reader,
        Err(e) => return Err(TransferError::from_bcp_error(
            "bcp process spawn failure", e.to_string()))
    };
    let mut buf_reader = BufReader::new(&reader);
    loop {
        let mut buf = vec!();
        match buf_reader.read_until(b'\n', &mut buf) {
            Ok(len) => {
                if 0 == len {
                    break;
                }
                if buf.len() >= 2 {
                    let ln = String::from_utf8_lossy(&buf[0..buf.len() - 2]);
                    progress_fun(&ln);
                }
            },
            Err(e) => return Err(TransferError::from_bcp_error(
                "bcp process failure", e.to_string()))
        };
    };
    match reader.try_wait() {
        Ok(opt) => match opt {
            Some(_) => { },
            None => return Err(TransferError::from_str("bcp process failure"))
        },
        Err(e) => return Err(TransferError::from_bcp_error(
            "bcp process failure", e.to_string()))
    }

    Ok(data_filename)
}

fn compress_bcp_file<P: Fn(&str)->()>(progress_fun: &P, dest_dir: &str,
                     data_filename: &str) -> Result<String, TransferError> {
    progress_fun(&format!("Compressing: {}", data_filename));
    progress_fun("");
    let compressed_filename = format!("{}.zstd", data_filename);
    let src_file_path = Path::new(dest_dir).join(data_filename);
    let dest_file_path = Path::new(dest_dir).join(&compressed_filename);
    {
        let src_file = File::open(&src_file_path)?;
        let dest_file = File::create(&dest_file_path)?;
        let mut reader = BufReader::new(src_file);
        let mut writer = zstd::stream::Encoder::new(BufWriter::new(dest_file), 1)?;
        let _ = writer.multithread(3);
        std::io::copy(&mut reader, &mut writer)?;
        let _ = writer.finish()?;
    }
    fs::remove_file(&src_file_path)?;
    Ok(compressed_filename)
}

fn export_tables<P: Fn(&str)->()>(progress_fun: &P, cc: &TdsConnConfig, eargs: &ExportArgs, dest_dir: &str) -> Result<(), TransferError> {
    for table in eargs.tables.iter() {
        let format_filename = run_bcp_format(progress_fun, cc, dest_dir, &eargs.dbname, &table.schema, &table.table)?;
        let data_filename = run_bcp_data(progress_fun, cc, dest_dir, &eargs.dbname, &table.schema, &table.table, &format_filename)?;
        let _ = compress_bcp_file(progress_fun, &dest_dir, &data_filename)?;
    }
    Ok(())
}

fn zip_dest_directory<P: Fn(&str)->()>(progress_fun: &P, dest_dir: &str, filename: &str) -> Result<(), TransferError> {
    let dest_dir_path = Path::new(dest_dir);
    let parent_path = match dest_dir_path.parent() {
        Some(path) => path,
        None => return Err(TransferError::from_str(
            "Error accessing destination directory parent"))
    };
    let dest_dir_st = match dest_dir_path.to_str() {
        Some(st) => st,
        None => return Err(TransferError::from_str(
            "Error accessing destination directory"))
    };
    let dest_file_buf = parent_path.join(filename);
    let dest_file_st = match dest_file_buf.to_str() {
        Some(st) => st,
        None => return Err(TransferError::from_str(
            "Error accessing destination file"))
    };
    let listener = |en: &str| {
        progress_fun(en);
    };
    zip_recurse::zip_directory_listen(dest_dir_st, dest_file_st, 0, listener)?;
    std::fs::remove_dir_all(dest_dir_path)?;
    Ok(())
}

fn prepare_dest_dir(dest_parent_dir: &str, dest_filename: &str) -> Result<(String, String), TransferError> {
    let mut ext = Path::new(dest_filename).extension().unwrap_or(OsStr::new(""))
        .to_str().unwrap_or("").to_string();
    let mut filename = dest_filename.to_string();
    if ext.is_empty() {
        ext = "zip".to_string();
        filename = format!("{}.{}", filename, ext);
    }
    let dirname: String = filename.chars().take(filename.len() - (ext.len() + 1)).collect();
    let parent_dir_path = Path::new(dest_parent_dir);
    let dir_path = parent_dir_path.join(dirname);
    let dir_path_st = match dir_path.to_str() {
        Some(st) => st.to_string(),
        None => return Err(TransferError::from_str("Error reading directory name"))
    };
    let _ = fs::remove_dir_all(&dir_path);
    if dir_path.exists() {
        return Err(TransferError::from_string(format!(
            "Error removing directory: {}", dir_path_st)))
    }
    fs::create_dir_all(dir_path)?;
    Ok((dir_path_st, filename))
}

pub fn run_export<P: Fn(&str)->()>(progress_fun: &P, cc: &TdsConnConfig, eargs: &ExportArgs) -> ExportResult {
    progress_fun("Running export ...");

    // ensure no dest dir
    let (dest_dir, filename) = match prepare_dest_dir(&eargs.parent_dir, &eargs.dest_filename) {
        Ok(tup) => tup,
        Err(e) => return ExportResult::failure(e.to_string())
    };
    let dest_file = Path::new(&eargs.parent_dir).join(Path::new(&filename)).to_string_lossy().to_string();
    progress_fun(&format!("Export file: {}", dest_file));

    // spawn and wait
    progress_fun("Running bcp ....");
    if let Err(e) = export_tables(progress_fun, cc, eargs, &dest_dir) {
        return ExportResult::failure(e.to_string());
    };

    // zip results
    progress_fun("Zipping destination directory ....");
    if let Err(e) = zip_dest_directory(progress_fun, &dest_dir, &filename) {
        return ExportResult::failure(format!(
            "Error zipping destination directory, path: {}, error: {}", &dest_dir, e));
    };

    progress_fun("Export complete");
    ExportResult::success()
}
