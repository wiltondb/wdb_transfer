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

use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::time;

use regex::Regex;

use super::*;

#[derive(Default)]
pub struct ExportDialog {
    pub(super) c: ExportDialogControls,

    args: ExportDialogArgs,
    command_join_handle: ui::PopupJoinHandle<ExportResult>,
    dialog_result: ExportDialogResult,

    progress_pending: Vec<String>,
    progress_last_updated: u128,
}

impl ExportDialog {

    pub(super) fn on_progress(&mut self, _: nwg::EventData) {
        let msg = self.c.progress_notice.receive();
        let flush_msg = msg.is_empty();
        if !flush_msg {
            self.progress_pending.push(msg);
        }
        let now = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis();
        if flush_msg || now - self.progress_last_updated > 100 {
            let joined = self.progress_pending.join("\r\n");
            self.progress_pending.clear();
            self.progress_last_updated = now;
            self.c.details_box.appendln(&joined);
        }
    }

    pub(super) fn on_complete(&mut self, _: nwg::EventData) {
        self.c.complete_notice.receive();
        let res = self.command_join_handle.join();
        let success = res.error.is_empty();
        self.stop_progress_bar(success.clone());
        if !success {
            self.dialog_result = ExportDialogResult::failure();
            self.c.label.set_text("Export failed");
            self.progress_pending.push(res.error);
            self.c.copy_clipboard_button.set_enabled(true);
            self.c.close_button.set_enabled(true);
        } else {
            self.dialog_result = ExportDialogResult::success();
            self.c.label.set_text("Export complete");
            self.c.copy_clipboard_button.set_enabled(true);
            self.c.close_button.set_enabled(true);
        }
        if self.progress_pending.len() > 0 {
            let joined = self.progress_pending.join("\r\n");
            self.c.details_box.appendln(&joined);
            self.progress_pending.clear();
        }
    }

    pub(super) fn copy_to_clipboard(&mut self, _: nwg::EventData) {
        let text = self.c.details_box.text();
        let _ = set_clipboard(formats::Unicode, &text);
    }

    fn stop_progress_bar(&self, success: bool) {
        self.c.progress_bar.set_marquee(false, 0);
        self.c.progress_bar.remove_flags(nwg::ProgressBarFlags::MARQUEE);
        self.c.progress_bar.set_pos(1);
        if !success {
            self.c.progress_bar.set_state(nwg::ProgressBarState::Error)
        }
    }

    fn strip_collation_from_format_file(dest_dir: &str, format_filename: &str) -> Result<(), io::Error> {
        let format_path = Path::new(dest_dir).join(&format_filename);
        let bytes = match fs::read(&format_path) {
            Ok(bytes) => bytes,
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!(
                "Format file post-processing error: {}", e)))
        };
        let codepoints: Vec<u16> = bytes
            .chunks_exact(2)
            .into_iter()
            .map(|a| u16::from_le_bytes([a[0], a[1]]))
            .collect();
        let text = match String::from_utf16(&codepoints) {
            Ok(text) => text,
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!(
                "Format file post-processing error: {}", e)))
        };
        let re = match Regex::new("(?i)\\sCOLLATION=\"\\w+\"") {
            Ok(re) => re,
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!(
                "Format file post-processing error: {}", e)))
        };
        let replaced = re.replace_all(&text, " COLLATION=\"\"").to_string();
        match fs::write(&format_path, replaced) {
            Ok(_) => { },
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!(
                "Format file post-processing error: {}", e)))
        };

        Ok(())
    }

    fn run_bcp_format(progress: &ui::SyncNoticeValueSender<String>, cc: &TdsConnConfig, dest_dir: &str,
               dbname: &str, schema: &str, table: &str) -> Result<String, io::Error> {
        progress.send_value(format!("Creating bcp format file: {}.{}", schema, table));
        let format_filename = format!("{}.{}.xml", schema, table);
        let cmd = duct::cmd!(
            "bcp.exe",
            format!("[{}].[{}].[{}]", dbname, schema, table),
            "format", "nul",
            "-f", &format_filename,
            "-x",
            "-n",
            "-k",
            "-K", "ReadOnly",
            "-S", format!("{},{}", &cc.hostname, &cc.port),
            "-U", &cc.username,
            "-P", &cc.password
        )
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
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!(
                "bcp process spawn failure: {}", e)))
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
                        progress.send_value(ln);
                    }
                },
                Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!(
                    "bcp process failure: {}", e)))
            };
        };
        match reader.try_wait() {
            Ok(opt) => match opt {
                Some(_) => { },
                None => return Err(io::Error::new(io::ErrorKind::Other, format!(
                        "bcp process failure")))
            },
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!(
                "bcp process failure: {}", e)))
        }

        Self::strip_collation_from_format_file(dest_dir, &format_filename)?;

        Ok(format_filename)
    }

    fn run_bcp_data(progress: &ui::SyncNoticeValueSender<String>, cc: &TdsConnConfig, dest_dir: &str,
                      dbname: &str, schema: &str, table: &str, format_filename: &str) -> Result<String, io::Error> {
        progress.send_value(format!("Exporting data: {}.{}", schema, table));
        let data_filename = format!("{}.{}.bcp", schema, table);
        let cmd = duct::cmd!(
            "bcp.exe",
            format!("[{}].[{}].[{}]", dbname, schema, table),
            "out", &data_filename,
            "-f", &format_filename,
            "-S", format!("tcp:{},{}", &cc.hostname, &cc.port),
            "-k",
            "-K", "ReadOnly",
            "-U", &cc.username,
            "-P", &cc.password
        )
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
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!(
                "bcp process spawn failure: {}", e)))
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
                        progress.send_value(ln);
                    }
                },
                Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!(
                    "bcp process failure: {}", e)))
            };
        };
        match reader.try_wait() {
            Ok(opt) => match opt {
                Some(_) => { },
                None => return Err(io::Error::new(io::ErrorKind::Other, format!(
                    "bcp process failure")))
            },
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!(
                "bcp process failure: {}", e)))
        }

        Ok(data_filename)
    }

    fn compress_bcp_file(progress: &ui::SyncNoticeValueSender<String>, dest_dir: &str,
                    data_filename: &str) -> Result<String, io::Error> {
        progress.send_value(format!("Compressing: {}", data_filename));
        progress.send_value("");
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

    fn export_tables(progress: &ui::SyncNoticeValueSender<String>, cc: &TdsConnConfig, eargs: &ExportArgs, dest_dir: &str) -> Result<(), io::Error> {
        for table in eargs.tables.iter() {
            let format_filename = Self::run_bcp_format(progress, cc, dest_dir, &eargs.dbname, &table.schema, &table.table)?;
            let data_filename = Self::run_bcp_data(progress, cc, dest_dir, &eargs.dbname, &table.schema, &table.table, &format_filename)?;
            let _ = Self::compress_bcp_file(&progress, &dest_dir, &data_filename)?;
        }
        Ok(())
    }

    fn zip_dest_directory(progress: &ui::SyncNoticeValueSender<String>, dest_dir: &str, filename: &str) -> Result<(), io::Error> {
        let dest_dir_path = Path::new(dest_dir);
        let parent_path = match dest_dir_path.parent() {
            Some(path) => path,
            None => return Err(io::Error::new(io::ErrorKind::PermissionDenied, format!(
                "Error accessing destination directory parent")))
        };
        let dest_dir_st = match dest_dir_path.to_str() {
            Some(st) => st,
            None => return Err(io::Error::new(io::ErrorKind::PermissionDenied, format!(
                "Error accessing destination directory")))
        };
        let dest_file_buf = parent_path.join(filename);
        let dest_file_st = match dest_file_buf.to_str() {
            Some(st) => st,
            None => return Err(io::Error::new(io::ErrorKind::PermissionDenied, format!(
                "Error accessing destination file")))
        };
        let listener = |en: &str| {
            progress.send_value(en);
        };
        if let Err(e) = zip_recurse::zip_directory_listen(dest_dir_st, dest_file_st, 0, listener) {
            return Err(io::Error::new(io::ErrorKind::Other, e.to_string()))
        };
        std::fs::remove_dir_all(dest_dir_path)?;
        Ok(())
    }

    fn prepare_dest_dir(dest_parent_dir: &str, dest_filename: &str) -> Result<(String, String), io::Error> {
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
            None => return Err(io::Error::new(io::ErrorKind::Other, format!(
                "Error reading directory name")))
        };
        let _ = fs::remove_dir_all(&dir_path);
        if dir_path.exists() {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, format!(
                "Error removing directory: {}", dir_path_st)));
        }
        fs::create_dir_all(dir_path)?;
        Ok((dir_path_st, filename))
    }

    fn run_export(progress: &ui::SyncNoticeValueSender<String>, cc: &TdsConnConfig, eargs: &ExportArgs) -> ExportResult {
        progress.send_value("Running export ...");

        // ensure no dest dir
        let (dest_dir, filename) = match Self::prepare_dest_dir(&eargs.parent_dir, &eargs.dest_filename) {
            Ok(tup) => tup,
            Err(e) => return ExportResult::failure(e.to_string())
        };
        let dest_file = Path::new(&eargs.parent_dir).join(Path::new(&filename)).to_string_lossy().to_string();
        progress.send_value(format!("Export file: {}", dest_file));

        // spawn and wait
        progress.send_value("Running bcp ....");
        if let Err(e) = ExportDialog::export_tables(progress, cc, eargs, &dest_dir) {
            return ExportResult::failure(e.to_string());
        };

        // zip results
        progress.send_value("Zipping destination directory ....");
        if let Err(e) = Self::zip_dest_directory(progress, &dest_dir, &filename) {
            return ExportResult::failure(format!(
                "Error zipping destination directory, path: {}, error: {}", &dest_dir, e));
        };

        progress.send_value("Export complete");
        ExportResult::success()
    }
}

impl ui::PopupDialog<ExportDialogArgs, ExportDialogResult> for ExportDialog {
    fn popup(args: ExportDialogArgs) -> ui::PopupJoinHandle<ExportDialogResult> {
        let join_handle = thread::spawn(move || {
            let data = Self {
                args,
                ..Default::default()
            };
            let mut dialog = Self::build_ui(data).expect("Failed to build UI");
            nwg::dispatch_thread_events();
            dialog.result()
        });
        ui::PopupJoinHandle::from(join_handle)
    }

    fn init(&mut self) {
        let complete_sender = self.c.complete_notice.sender();
        let progress_sender = self.c.progress_notice.sender();
        let cc: TdsConnConfig = self.args.conn_config.clone();
        let eargs = self.args.export_args.clone();
        let join_handle = thread::spawn(move || {
            let start = Instant::now();
            let res = ExportDialog::run_export(&progress_sender, &cc, &eargs);
            let remaining = 1000 - start.elapsed().as_millis() as i64;
            if remaining > 0 {
                thread::sleep(Duration::from_millis(remaining as u64));
            }
            complete_sender.send();
            res
        });
        self.command_join_handle = ui::PopupJoinHandle::from(join_handle);
    }

    fn result(&mut self) -> ExportDialogResult {
        self.dialog_result.clone()
    }

    fn close(&mut self, _: nwg::EventData) {
        self.args.send_notice();
        self.c.window.set_visible(false);
        nwg::stop_thread_dispatch();
    }

    fn on_resize(&mut self, _: nwg::EventData) {
        self.c.update_tab_order();
    }
}

