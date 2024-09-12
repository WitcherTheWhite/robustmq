// Copyright 2023 RobustMQ Team
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{
    fs,
    path::{self, Path},
    time::{SystemTime, UNIX_EPOCH},
};

use local_ip_address::local_ip;
use log::warn;
use uuid::Uuid;

use crate::error::common::CommonError;

/// Create multi-level directory
///
/// If the specified directory path does not exist, attempt to create the path and all its parent directories.
/// This function does not perform any operations on existing directories.
///
/// # Parameters
/// Fold - a string that specifies the directory path to be created.
///
/// # Return value
/// * ` OK() ` - Indicates that the directory has been successfully created or the specified directory already exists.
/// Err (CommonError) - Indicates that the directory creation failed, and CommonError is the error type that includes the reason for the failure.
///
/// # Possible errors
/// If the directory creation fails, a 'CommonError' containing the reason for the error will be returned.
pub fn create_fold(fold: &String) -> Result<(), CommonError> {
    if !Path::new(fold).exists() {
        fs::create_dir_all(fold)?
    }
    Ok(())
}

/// Retrieve the current timestamp (in milliseconds)
///
/// Returns the number of milliseconds since the Unix era (January 1, 1970 00:00:00 UTC).
/// This function uses system time and converts it to a duration with millisecond precision.
///
/// # Return value
/// Returns the number of milliseconds since the Unix era (type u128).
pub fn now_mills() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

/// Get current seconds
///
/// # Return value
/// Returns the number of seconds since the Unix era (January 1, 1970 00:00:00 UTC).
pub fn now_second() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Generate a unique Uuid
///
/// This function generates a version 4 UUID (Universally Unique Identifier) and converts it to string form,
/// Remove the connecting character (-) from it
/// 
/// # Return value
/// String - A Uuid without connecting characters
pub fn unique_id() -> String {
    let uuid = Uuid::new_v4();
    uuid.to_string().replace("-", "")
}

/// Obtain local IP address
/// 
/// This function attempts to obtain the local IP address of the device and returns the address as a string upon success
/// If obtaining the IP address fails, this function will output a warning message and return the string '127.0.0.1'
/// 
/// # Return value
/// - When successfully obtaining a local IP address, return a string representation of the address
/// - When obtaining the IP address fails, return the string '127.0.0.1'
pub fn get_local_ip() -> String {
    match local_ip() {
        Ok(data) => {
            data.to_string()
        }
        Err(e) => {
            warn!(
                "If the local IP fails, stop the process.error message:{}",
                e.to_string()
            );
            "127.0.0.1".to_string()
        }
    }
}

/// Check if the file exists
///
/// # Parameters
/// * ` path ` - The path of the file, as a string reference
///
/// # Return value
/// Return a Boolean value indicating whether the file exists
pub fn file_exists(path: &String) -> bool {
    Path::new(path).exists()
}

/// Read the content of a text file
///
/// # Parameters
/// * ` path ` - a string type reference
///
/// # Return value
/// Return a result type that includes a string containing the file content upon success and a generic error upon failure
///
/// # Error
/// If the file does not exist, return a generic error indicating that the file does not exist
pub fn read_file(path: &String) -> Result<String, CommonError> {
    if !path::Path::new(path).exists() {
        return Err(CommonError::CommmonError(format!(
            "File {} does not exist",
            path
        )));
    }

    Ok(fs::read_to_string(&path)?)
}

#[cfg(test)]
mod tests {
    use crate::tools::{get_local_ip, unique_id};

    #[test]
    fn get_local_ip_test() {
        println!("{}", get_local_ip());
    }

    #[test]
    fn unique_id_test() {
        println!("{}", unique_id());
    }
}
