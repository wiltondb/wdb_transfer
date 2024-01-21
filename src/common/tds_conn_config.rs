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

use tiberius::{AuthMethod, Client, Config};
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
}

impl TdsConnConfig {

    pub fn create_runtime(&self) -> Result<Runtime, TransferError> {
        let res = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .build()?;
        Ok(res)
    }

    pub fn open_connection(&self, runtime: &Runtime) -> Result<Client<Compat<TcpStream>>, TransferError> {
        runtime.block_on(async {
            let mut config = Config::new();
            config.host(&self.hostname);
            config.port(self.port);
            config.database(&self.database);
            config.authentication(AuthMethod::sql_server(&self.username, &self.password));
            if self.accept_invalid_tls {
                config.trust_cert();
            }
            let tcp = TcpStream::connect(config.get_addr()).await?;
            tcp.set_nodelay(true)?;
            let client = Client::connect(config, tcp.compat_write()).await?;
            Ok(client)
        })
    }

}
