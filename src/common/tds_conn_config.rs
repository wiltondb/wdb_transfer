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

use tiberius::{AuthMethod, Client, Config, SqlBrowser};
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio_util::compat::Compat;
use tokio_util::compat::TokioAsyncWriteCompatExt;

use super::*;

#[derive(Default, Debug, Clone)]
pub struct TdsConnConfig {
    pub hostname: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub accept_invalid_tls: bool,
    pub use_win_auth: bool,
    pub instance: String,
}

impl TdsConnConfig {

    pub fn create_runtime(&self) -> Result<Runtime, TransferError> {
        let res = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()?;
        Ok(res)
    }

    fn open_connection(&self, runtime: &Runtime, dbname: &str) -> Result<Client<Compat<TcpStream>>, TransferError> {
        runtime.block_on(async {
            let mut config = Config::new();
            config.host(&self.hostname);
            config.database(dbname);
            if self.accept_invalid_tls {
                config.trust_cert();
            }
            let client = if self.use_win_auth {
                config.authentication(AuthMethod::Integrated);
                config.instance_name(&self.instance);
                let tcp = TcpStream::connect_named(&config).await?;
                Client::connect(config, tcp.compat_write()).await?
            } else {
                config.authentication(AuthMethod::sql_server(&self.username, &self.password));
                config.port(self.port);
                let tcp = TcpStream::connect(config.get_addr()).await?;
                tcp.set_nodelay(true)?;
                Client::connect(config, tcp.compat_write()).await?
            };
            Ok(client)
        })
    }

    pub fn open_connection_default(&self, runtime: &Runtime) -> Result<Client<Compat<TcpStream>>, TransferError> {
        self.open_connection(runtime, &self.database)
    }

    pub fn open_connection_to_db(&self, runtime: &Runtime, dbname: &str) -> Result<Client<Compat<TcpStream>>, TransferError> {
        self.open_connection(runtime, dbname)
    }
}
