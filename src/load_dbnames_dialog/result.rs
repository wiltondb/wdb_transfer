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

#[derive(Default)]
pub(super) struct LoadDbnamesResult {
    pub(super) dbnames: Vec<String>,
    pub(super) error: String,
}

impl LoadDbnamesResult {
    pub(super) fn success(dbnames: Vec<String>) -> Self {
        Self {
            dbnames,
            error: String::new()
        }
    }

    pub(super) fn failure(error: String) -> Self {
        Self {
            error,
            ..Default::default()
        }
    }
}

#[derive(Default, Clone)]
pub struct LoadDbnamesDialogResult {
    pub success: bool,
    pub dbnames: Vec<String>,
}

impl LoadDbnamesDialogResult {
    pub fn success(dbnames: Vec<String>) -> Self {
        Self {
            success: true,
            dbnames,
        }
    }

    pub fn failure() -> Self {
        Self {
            success: false,
            ..Default::default()
        }
    }
}
