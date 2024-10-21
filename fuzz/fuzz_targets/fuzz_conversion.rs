// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![no_main]

#[macro_use] extern crate libfuzzer_sys;
extern crate httpdate;

use httpdate::HttpDate;
use std::time::{SystemTime, UNIX_EPOCH};
use std::convert::TryInto;

fuzz_target!(|data: &[u8]| {
    // Skip this round if data is not enough
    if data.len() < 8 {
        return;
    }

    // Create system time object
    let secs_since_epoch = u64::from_le_bytes(data[0..8].try_into().unwrap_or_default());
    let duration = std::time::Duration::from_secs(secs_since_epoch);
    let system_time = match UNIX_EPOCH.checked_add(duration) {
        Some(time) => time,
        None => return,
    };

    // Skip value that could make HttpDate panic
    if secs_since_epoch >= 253402300800 {
        return;
    }

    // Fuzz other functions
    let http_date = HttpDate::from(system_time);
    let _ = SystemTime::from(http_date);
    let _ = http_date.to_string();

    // Fuzz partial_cmp if enough data is left
    if data.len() >= 16 {
        let other_secs_since_epoch = u64::from_le_bytes(data[8..16].try_into().unwrap_or_default());
        let other_duration = std::time::Duration::from_secs(other_secs_since_epoch);
        let other_system_time = match UNIX_EPOCH.checked_add(other_duration) {
            Some(time) => time,
            None => return,
        };

        // Skip value that could make HttpDate panic
        if other_secs_since_epoch >= 253402300800 {
            return;
        }

        let other_http_date = HttpDate::from(other_system_time);
        let _ = http_date.partial_cmp(&other_http_date);
    }
});
