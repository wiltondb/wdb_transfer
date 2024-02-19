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
use std::io::BufReader;
use std::io::BufWriter;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;

use zip::result::ZipError;
use zip::result::ZipResult;
use zip::ZipWriter;
use zip::write::FileOptions;
use chrono::{Datelike, Timelike};

fn strip_prefix(parent: &Path, child: &Path) -> Result<PathBuf, io::Error> {
    match child.strip_prefix(parent) {
        Ok(rel_path) => Ok(rel_path.to_path_buf()),
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!(
            "Strip prefix error, path: {}, error: {}", child.to_str().unwrap_or(""), e)))
    }
}

fn path_to_string(path: &Path) -> Result<String, io::Error> {
    let st = match path.to_str() {
        Some(name) => name.to_string(),
        None => return Err(io::Error::new(io::ErrorKind::Other, format!(
            "Path access error")))
    };
    let res = st.replace("\\", "/");
    Ok(res)
}

fn time_to_zip_time(system_time: &SystemTime) -> zip::DateTime {
    let tm: chrono::DateTime<chrono::Utc> = system_time.clone().into();
    match zip::DateTime::from_date_and_time(
        tm.year() as u16, tm.month() as u8, tm.day() as u8,
        tm.hour() as u8, tm.minute() as u8, tm.second() as u8) {
        Ok(zdt) => zdt,
        Err(_) => zip::DateTime::default()
    }
}

fn read_dir_paths(dir: &Path) -> Result<Vec<PathBuf>, io::Error> {
    let rd = fs::read_dir(dir)?;
    let mut res: Vec<PathBuf> = Vec::new();
    for en in rd {
        let en = en?;
        res.push(en.path())
    }
    res.sort_by(|a, b| {
        if a.is_dir() && !b.is_dir() {
            std::cmp::Ordering::Greater
        } else if b.is_dir() && !a.is_dir() {
            std::cmp::Ordering::Less
        } else {
            a.cmp(b)
        }
    });
    Ok(res)
}

fn zip_file
        <T: io::Seek + io::Write, F: Fn (&str) -> ()>
        (zip: &mut ZipWriter<T>, root_dir: &Path, path: &Path, listener: &F) -> ZipResult<()> {
    let file = File::open(path)?;
    let meta = file.metadata()?;
    let system_time = meta.modified()?;
    let zip_time = time_to_zip_time(&system_time);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .last_modified_time(zip_time);
    let rel_path = match root_dir.parent() {
        Some(parent) => strip_prefix(parent, path)?,
        None => path.to_path_buf()
    };
    let name = path_to_string(&rel_path)?;
    listener(&name);
    zip.start_file(name, options)?;
    let mut reader = BufReader::new(file);
    std::io::copy(&mut reader, zip)?;
    Ok(())
}

fn zip_dir_recursive
        <T: io::Seek + io::Write, F: Fn (&str) -> ()>
        (zip: &mut ZipWriter<T>, root_dir: &Path, dir: &Path, listener: &F) -> ZipResult<()> {
    if !dir.is_dir() {
        return Err(ZipError::FileNotFound);
    }
    let rel_path = match root_dir.parent() {
        Some(parent) => strip_prefix(parent, dir)?,
        None => dir.to_path_buf()
    };
    let name = path_to_string(&rel_path)?;
    listener(&name);
    let medatata = dir.metadata()?;
    let system_time = medatata.modified()?;
    let zip_time = time_to_zip_time(&system_time);
    let options = FileOptions::default()
        .last_modified_time(zip_time);
    zip.add_directory(name, options)?;
    for path in read_dir_paths(dir)? {
        if path.is_dir() {
            zip_dir_recursive(zip, root_dir, &path, listener)?;
        } else {
            zip_file(zip, root_dir, &path, listener)?;
        }
    }
    zip.finish()?;
    Ok(())
}

pub fn zip_directory<F: Fn (&str) -> ()>(src_dir: &str, dst_file: &str, comp_level: u8, listener: &F) -> zip::result::ZipResult<()> {
    let src_dir_path = Path::new(src_dir);
    if !src_dir_path.is_dir() {
        return Err(ZipError::FileNotFound);
    }
    if 0 != comp_level {
        return Err(ZipError::UnsupportedArchive("Compression not supported"));
    };
    let file = match File::create(Path::new(dst_file)) {
        Ok(file) => file,
        Err(e) => return Err(ZipError::Io(e))
    };
    let mut zip = zip::ZipWriter::new(BufWriter::new(file));
    zip_dir_recursive(&mut zip, &src_dir_path, &src_dir_path, listener)?;
    Ok(())
}

#[allow(dead_code)]
pub fn unzip_directory<F: Fn (&str) -> ()>(zip_file: &str, dest_dir: &str, listener: &F) -> zip::result::ZipResult<String> {
    let file = match File::open(Path::new(zip_file)) {
        Ok(file) => file,
        Err(e) => return Err(ZipError::Io(e))
    };
    let mut zip = zip::ZipArchive::new(BufReader::new(file))?;
    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        listener(file.name());
        let filepath = file
            .enclosed_name()
            .ok_or(ZipError::InvalidArchive("Invalid file path"))?;
        let outpath = Path::new(dest_dir).join(filepath);
        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            let mut outfile = fs::File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }
    let entry = zip.by_index(0)?;
    Ok(entry.name().to_string())
}