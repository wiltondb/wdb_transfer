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

use std::fmt;
use std::io;

use regex::Regex;

#[derive(Debug)]
pub struct TransferError {
    message: String
}

impl TransferError {
    pub fn new<E: fmt::Display>(e: &E) -> Self {
        Self {
            message: format!("{}", e)
        }
    }

    pub fn from_string(message: String) -> Self {
        Self {
            message
        }
    }

    pub fn from_str(message: &str) -> Self {
        Self {
            message: message.to_string()
        }
    }

    pub fn from_bcp_error(prefix: &str, msg_with_password: String) -> Self {
        let re = match Regex::new("(?P<pre>,\\s\"-P\",\\s\")(.+)(?P<post>\"])") {
            Ok(re) => re,
            Err(_) => return Self {
                message: prefix.to_string()
            }
        };
        let msg_wo_password = re.replace_all(&msg_with_password, "$pre******$post");
        Self {
            message: format!("{}: {}", prefix, msg_wo_password)
        }
    }
}

impl fmt::Display for TransferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<io::Error> for TransferError {
    fn from(value: io::Error) -> Self {
        Self::new(&value)
    }
}

impl From<tiberius::error::Error> for TransferError {
    fn from(value: tiberius::error::Error) -> Self {
        Self::new(&value)
    }
}

impl From<zip::result::ZipError> for TransferError {
    fn from(value: zip::result::ZipError) -> Self {
        Self::new(&value)
    }
}
