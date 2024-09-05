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

mod common;

use std::env;
use std::path::PathBuf;
use std::process;

use clap::Arg;
use clap::ArgAction;
use clap::ArgMatches;
use clap::Command;

use common::ExportArgs;
use common::ImportArgs;
use common::TdsConnConfig;
use common::TransferError;

fn main() {
    let args = Command::new("WiltonDB data transfer command-line tool")
        .author("WiltonDB Software")
        .version(common::labels::VERSION)
        .about("Data transfer tool for WiltonDB")
        .arg(Arg::new("command")
            .required(true)
            .help("Specifies the task to perform, either 'export' or 'import'"))
        .arg(Arg::new("hostname")
            .short('s')
            .long("hostname")
            .required(true)
            .help("Specifies the hostname of the DB to which to connect."))
        .arg(Arg::new("port")
            .short('p')
            .long("port")
            .required(false)
            .conflicts_with("instance")
            .help("Specifies the TCP port of the DB to which to connect."))
        .arg(Arg::new("instance")
            .short('n')
            .long("instance")
            .required(false)
            .conflicts_with("port")
            .help("Specifies the instance name of SQL Server to which to connect."))
        .arg(Arg::new("username")
            .short('u')
            .long("username")
            .required(false)
            .conflicts_with("windows_auth")
            .help("Specifies the login name used to connect to DB."))
        .arg(Arg::new("password")
            .short('x')
            .long("password")
            .required(false)
            .conflicts_with("windows_auth")
            .help("Specifies the password for the login ID. If this option isn't used, the password is read from WDBTRANSFERPASSWORD environment variable."))
        .arg(Arg::new("windows_auth")
            .short('w')
            .long("windows_auth")
            .required(false)
            .action(ArgAction::SetTrue)
            .conflicts_with("password")
            .conflicts_with("username")
            .help("Specifies that the tool connects to DB with a trusted connection using integrated security."))
        .arg(Arg::new("database")
            .short('d')
            .long("database")
            .required(true)
            .help("Specifies the database to connect to."))
        .arg(Arg::new("check_certificate")
            .short('c')
            .long("check_certificate")
            .required(false)
            .action(ArgAction::SetTrue)
            .help("Enforce the validation of the DB server certificate."))
        .arg(Arg::new("input_file")
            .short('i')
            .long("input_file")
            .required(false)
            .conflicts_with("output_file")
            .help("Specifies the path to input data file."))
        .arg(Arg::new("output_file")
            .short('o')
            .long("output_file")
            .required(false)
            .conflicts_with("input_file")
            .help("Specifies the path to output data file."))
        .arg(Arg::new("overwrite_output_file")
            .short('r')
            .long("overwrite_output_file")
            .required(false)
            .action(ArgAction::SetTrue)
            .help("Overwrite existing output file."))
        .get_matches();

    match run(&args) {
        Ok(()) => {
            process::exit(0);
        },
        Err(e) => {
            println!("ERROR: {}.", e);
            process::exit(1);
        },
    };
}

fn run(args: &ArgMatches) -> Result<(), TransferError> {
    let (cmd, file_path) = check_command(&args)?;
    let cfg = create_conn_cfg(&args)?;

    if "export" == cmd {
        run_export(cfg, file_path)
    } else if "import" == cmd {
        run_import(cfg, file_path)
    } else {
        Err(TransferError::from_string(format!("invalid comand name: {}", cmd)))
    }
}

fn run_export(cfg: TdsConnConfig, output_file_path: PathBuf) -> Result<(), TransferError> {
    let progress_fun = |st: &str| {
        println!("{}", st);
    };

    let output_file = output_file_path.to_string_lossy().to_string();
    let output_file_name_ost = output_file_path.file_name().ok_or(TransferError::from_string(format!(
        "cannot get file name from path: {}", &output_file)))?;
    let output_file_name = output_file_name_ost.to_str().ok_or(TransferError::from_string(format!(
        "cannot get file name from path: {}", &output_file)))?;
    let parent_dir_path = output_file_path.parent().ok_or(TransferError::from_string(format!(
        "cannot get parent directory from path: {}", &output_file)))?;
    let parent_dir = parent_dir_path.to_string_lossy().to_string();

    let tables = common::load_tables_from_db(&progress_fun, &cfg, &cfg.database)?;
    let eargs = ExportArgs {
        dbname: cfg.database.to_string(),
        tables: tables,
        parent_dir: parent_dir,
        dest_filename: output_file_name.to_string(),
    };
    let res = common::run_export(&progress_fun, &cfg, &eargs);
    if !res.error.is_empty() {
        return Err(TransferError::from_string(res.error));
    }

    Ok(())
}

fn run_import(cfg: TdsConnConfig, input_file_path: PathBuf) -> Result<(), TransferError> {
    let progress_fun = |st: &str| {
        println!("{}", st);
    };

    let input_file = input_file_path.to_string_lossy().to_string();
    let dir_path = input_file_path.with_extension("");
    let dir_path_st = dir_path.to_string_lossy().to_string();

    let tables = common::load_tables_from_file(&progress_fun, &input_file)?;
    let iargs = ImportArgs {
        dbname: cfg.database.to_string(),
        tables: tables,
        import_file: input_file,
        work_dir: dir_path_st,
    };
    let res = common::run_import(&progress_fun, &cfg, &iargs);
    if !res.error.is_empty() {
        return Err(TransferError::from_string(res.error));
    }

    Ok(())
}

fn check_command(args: &ArgMatches) -> Result<(String, PathBuf), TransferError> {
    let command = args.get_one::<String>("command").map(|s| s.to_string()).unwrap_or_default();
    let input_file = args.get_one::<String>("input_file").map(|s| s.to_string()).unwrap_or_default();
    let output_file = args.get_one::<String>("output_file").map(|s| s.to_string()).unwrap_or_default();
    let overwrite_output_file = args.get_one::<bool>("overwrite_output_file").map(|v| *v).unwrap_or(false);

    if "export" == command {
        if !output_file.is_empty() {
            let output_file_path = PathBuf::from(output_file);
            if !output_file_path.exists() || overwrite_output_file {
                Ok((command.to_string(), output_file_path))
            } else {
                Err(TransferError::from_str("specified output file already exists, add 'overwrite_output_file' (-r) option to overwrite it"))
            }
        } else {
            Err(TransferError::from_str("'output_file' option must be specified"))
        }
    } else if "import" == command {
        let input_file_path = PathBuf::from(input_file);
        if input_file_path.exists() {
            Ok((command.to_string(), input_file_path))
        } else {
            Err(TransferError::from_str("specified input file does not exist"))
        }
    } else {
        Err(TransferError::from_str("invalid command, either 'export' or 'import' command must be specified"))
    }
}

fn create_conn_cfg(args: &ArgMatches) -> Result<TdsConnConfig, TransferError> {
    let hostname = args.get_one::<String>("hostname").map(|s| s.to_string()).unwrap_or_default();
    let port_st = args.get_one::<String>("port").map(|s| s.to_string()).unwrap_or_default();
    let instance = args.get_one::<String>("instance").map(|s| s.to_string()).unwrap_or_default();
    let username = args.get_one::<String>("username").map(|s| s.to_string()).unwrap_or_default();
    let mut password = args.get_one::<String>("password").map(|s| s.to_string()).unwrap_or_default().to_string();
    let windows_auth = args.get_one::<bool>("windows_auth").map(|v| *v).unwrap_or(false);
    let database = args.get_one::<String>("database").map(|s| s.to_string()).unwrap_or_default();
    let check_certificate = args.get_one::<bool>("check_certificate").map(|v| *v).unwrap_or(false);

    if hostname.is_empty() {
        return Err(TransferError::from_str("'hostname' option must be specified"));
    }
    let port: i32 = if !port_st.is_empty() {
        port_st.parse()?
    } else {
        0
    };
    if instance.is_empty() && (port <= 0 || port >= 1<<16) {
        return Err(TransferError::from_str("'port' option must be specified with a value between 1 and 65535"));
    }
    if password.is_empty() {
        if let Ok(pwd) = env::var("WDBTRANSFERPASSWORD") {
            password = pwd;
        }
    }
    if !windows_auth && (username.is_empty() || password.is_empty()) {
        return Err(TransferError::from_str("'username' and 'password' options must be specified"));
    }
    if database.is_empty() {
        return Err(TransferError::from_str("'database' option must be specified"));
    }

    Ok(TdsConnConfig {
        hostname: hostname.to_string(),
        port: port as u16,
        instance: instance.to_string(),
        use_named_instance: !instance.is_empty(),
        username: username.to_string(),
        password: password.to_string(),
        use_win_auth: windows_auth,
        database: database.to_string(),
        accept_invalid_tls: !check_certificate,
    })
}
