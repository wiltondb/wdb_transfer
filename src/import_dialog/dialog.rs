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

use std::fs;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::path::PathBuf;
use std::time;

use flate2::bufread::GzDecoder;
use zip::ZipArchive;

use super::*;

#[derive(Default)]
pub struct ImportDialog {
    pub(super) c: ImportDialogControls,

    args: ImportDialogArgs,
    command_join_handle: ui::PopupJoinHandle<ImportResult>,
    dialog_result: ImportDialogResult,

    progress_pending: Vec<String>,
    progress_last_updated: u128,
}

impl ImportDialog {

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
            self.dialog_result = ImportDialogResult::failure();
            self.c.label.set_text("Import failed");
            self.progress_pending.push(res.error);
            self.c.copy_clipboard_button.set_enabled(true);
            self.c.close_button.set_enabled(true);
        } else {
            self.dialog_result = ImportDialogResult::success();
            self.c.label.set_text("Import complete");
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

    fn unzip_table_files(progress: &ui::SyncNoticeValueSender<String>, table: &TableWithSize, import_file: &str, work_dir: &Path) -> Result<(PathBuf, PathBuf), TransferError> {
        let import_file_path = Path::new(import_file);
        let bcp_filename = format!("{}.{}.bcp", &table.schema, &table.table);
        progress.send_value(format!("Unpacking {} into directory {}", &bcp_filename, work_dir.to_string_lossy().to_string()));
        let zip_file = File::open(import_file_path)?;
        let zip_reader = BufReader::new(zip_file);
        let mut zip =  ZipArchive::new(zip_reader)?;
        let dirname: String = match zip.file_names().find(|nm| nm.ends_with("/")) {
            Some(dirname) => dirname.chars().take(dirname.len() - 1).collect(),
            None => return Err(TransferError::from_str("Directory entry not found in ZIP file"))
        };
        let bcp_gz_file = work_dir.join(&bcp_filename);
        {
            let file = File::create(&bcp_gz_file)?;
            let mut writer = BufWriter::new(file);
            let entry_name_base = format!("{}/{}", &dirname, &bcp_filename);
            let entry_name_gz = format!("{}.gz", &entry_name_base);
            let entry_name_zstd = format!("{}.zstd", &entry_name_base);
            let entry_res = zip.by_name(&entry_name_zstd);
            let entry = match entry_res {
                Ok(entry) => entry,
                Err(_) => {
                    std::mem::drop(entry_res);
                    match zip.by_name(&entry_name_gz) {
                        Ok(entry) => entry,
                        Err(_) => return Err(TransferError::from_string(
                            format!("Table data entry not found in ZIP file, name: {} or {}", entry_name_zstd, entry_name_gz)))
                    }
                }
            };
            let entry_name = entry.name().to_string();
            let entry_buffered = BufReader::new(entry);
            if entry_name.ends_with(".zstd") {
                let mut entry_decomp = BufReader::new(zstd::Decoder::new(entry_buffered)?);
                std::io::copy(&mut entry_decomp, &mut writer)?;
            } else {
                let mut entry_decomp = BufReader::new(GzDecoder::new(entry_buffered));
                std::io::copy(&mut entry_decomp, &mut writer)?;
            };
        }

        let format_filename = format!("{}.{}.xml", &table.schema, &table.table);
        let format_file = work_dir.join(&format_filename);
        {
            let file = File::create(&format_file)?;
            let mut writer = BufWriter::new(file);
            let entry = zip.by_name(&format!("{}/{}", &dirname, &format_filename))?;
            let mut entry_buffered = BufReader::new(entry);
            std::io::copy(&mut entry_buffered, &mut writer)?;
        }
        Ok((bcp_gz_file, format_file))
    }

    fn run_bcp(progress: &ui::SyncNoticeValueSender<String>, cc: &TdsConnConfig, dbname: &str,
               table: &TableWithSize, bcp_file: &Path, format_file: &Path, work_dir: &Path) -> Result<(), TransferError> {
        let bcp_filename = bcp_file.file_name().ok_or(
            TransferError::from_str("Filename error"))?.to_string_lossy().to_string();
        let format_filename = format_file.file_name().ok_or(
            TransferError::from_str("Filename error"))?.to_string_lossy().to_string();
        progress.send_value(format!("Importing file: {}", bcp_filename));
        let mut args: Vec<String> = vec!(
            format!("[{}].[{}].[{}]", dbname, &table.schema, &table.table),
            "in".to_string(),
            bcp_filename.clone(),
            "-f".to_string(),
            format_filename.clone(),
            "-k".to_string(),
            "-E".to_string(),
            "-m".to_string(),
            "1".to_string(),
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
            .dir(work_dir)
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
                        progress.send_value(ln);
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

        Ok(())
    }

    fn import_tables(progress: &ui::SyncNoticeValueSender<String>, cc: &TdsConnConfig, iargs: &ImportArgs, work_dir: &Path) -> Result<(), TransferError> {
        for table in iargs.tables.iter() {
            let (bcp_file, format_file) = Self::unzip_table_files(progress, &table, &iargs.import_file, work_dir)?;
            Self::run_bcp(progress, cc, &iargs.dbname, &table, &bcp_file, &format_file, work_dir)?;
        }
        Ok(())
    }

    fn prepare_work_dir(work_dir: &str) -> Result<PathBuf, io::Error> {
        let dir_path = Path::new(work_dir);
        let _ = fs::remove_dir_all(&dir_path);
        if dir_path.exists() {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, format!(
                "Error removing directory: {}", work_dir)));
        }
        fs::create_dir_all(&dir_path)?;
        Ok(dir_path.to_path_buf())
    }

    fn run_import(progress: &ui::SyncNoticeValueSender<String>, cc: &TdsConnConfig, iargs: &ImportArgs) -> ImportResult {
        progress.send_value(format!("Running import: {} ...", iargs.import_file));

        // ensure empty work dir
        let work_dir = match Self::prepare_work_dir(&iargs.work_dir) {
            Ok(tup) => tup,
            Err(e) => return ImportResult::failure(e.to_string())
        };

        // spawn and wait
        if let Err(e) = ImportDialog::import_tables(progress, cc, iargs, &work_dir) {
            return ImportResult::failure(e.to_string());
        };

        // clean up
        progress.send_value("Cleaning up work directory ....");
        let _ = fs::remove_dir_all(&work_dir);

        progress.send_value("Import complete");
        ImportResult::success()
    }
}

impl ui::PopupDialog<ImportDialogArgs, ImportDialogResult> for ImportDialog {
    fn popup(args: ImportDialogArgs) -> ui::PopupJoinHandle<ImportDialogResult> {
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
        let iargs = self.args.import_args.clone();
        let join_handle = thread::spawn(move || {
            let start = Instant::now();
            let res = ImportDialog::run_import(&progress_sender, &cc, &iargs);
            let remaining = 1000 - start.elapsed().as_millis() as i64;
            if remaining > 0 {
                thread::sleep(Duration::from_millis(remaining as u64));
            }
            complete_sender.send();
            res
        });
        self.command_join_handle = ui::PopupJoinHandle::from(join_handle);
    }

    fn result(&mut self) -> ImportDialogResult {
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

