# FAAF (Firmware Analysis Assistance Framework)

## Current Status

Some functions of this program are now working. 
However, functionalities such as `dependencies` in the config file, are still not operational.

## Objectives

The goal is to analyze firmware using specific analyzers. The results from these analyzers are stored in a database.

### Usage

```
Usage: faaf --firmware-root-dir <FIRMWARE_ROOT_DIR> --script-directory <SCRIPT_DIRECTORY> --config-file <CONFIG_FILE> --database-file <DATABASE_FILE>

Options:
  -f, --firmware-root-dir <FIRMWARE_ROOT_DIR>  Firmware root directory
  -s, --script-directory <SCRIPT_DIRECTORY>    Analyzer directory
  -c, --config-file <CONFIG_FILE>              Config file for the analyzer
  -d, --database-file <DATABASE_FILE>          Output database file(sqlite)
  -h, --help                                   Print help
  -V, --version  
```

## Analysis Methodology

The framework will iterate through multiple files in the extracted firmware. For each file, specific analysis scripts will be run to collect information.

### Configuration File

The analysis scripts are specified in a toml configuration file, structured as follows:

```toml
[[analyzer]]
# Name of the analysis script (required)
name = "basic_info"
# Extension of the analysis script (required)
extension = "so"

[[analyzer]]
name = "ldd"
extension = "sh"
# Dependencies. Write the name of the analyzers.
dependencies = ["basic_info"]
# Arguments to pass to the analyzer (dependencies required)
# Structure not yet finalized
arguments = ["basic_info"]
# Conditions for the analyzer to be executed (dependencies required)
# Structure not yet finalized
conditions = """
basic_info.mime == \"application/x-pie-executable\" and
basic_info.size > 5000
"""

[[analyzer]]
name = "ghidra"
extension = "sh"
dependencies = ["basic_info", "ldd"]
```

The analyzer will support file types .so, .py, and .sh.

## Writing a analyzer

For analyzer written in Python (py) or as a shared object (so), the entry point is a function called analyzer_main. This function will receive a JSON-formatted string as its argument from faaf and should return a JSON-formatted string as its output. The output JSON must have a result key at its root.

### Analyzer for py

```py
import base64
import json
def analyzer_main(json_str: str) -> str:

    jsn_dict = json.loads(json_str)
    path = jsn_dict["absolute_path"] 

    rst = {
        "result": {
            # data
        }
    }

    return json.dumps(rst)
```

### Analyzer for so(rust)

```rs
extern crate serde;
extern crate serde_json;
extern crate magic;

use std::fs;
use serde::{Deserialize, Serialize};
use std::os::unix::fs::PermissionsExt;
use magic::Cookie;

#[derive(Deserialize)]
struct Input {
    path: String,
}

#[repr(C)]
pub struct OutputData {
    data: *mut u8,
    len: usize,
}

#[derive(Serialize)]
struct FileInfo {
    file_size: u64,
    mime: String,
    permissions: u32,
    is_dir: bool,
    is_file: bool,
    last_modified: std::time::SystemTime,
    last_accessed: std::time::SystemTime,
    created: Option<std::time::SystemTime>,
}

#[no_mangle]
pub extern "C" fn analyzer_main(data: *const u8, len: usize) -> OutputData {
    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    let input_data = String::from_utf8_lossy(slice);
    let v: serde_json::Value = serde_json::from_str(&input_data).unwrap();
    //println!("{:?}", v);

    let path = v["absolute_path"].as_str().unwrap();
    let metadata = fs::metadata(path).unwrap();
    let permissions = metadata.permissions().mode();

    let cookie = Cookie::open(magic::cookie::Flags::MIME_TYPE | magic::cookie::Flags::MIME_ENCODING).unwrap();
    let cookie = cookie.load(&Default::default()).unwrap();
    let mime_type = cookie.file(&path).unwrap();

	let file_info = FileInfo {
	    file_size: metadata.len(),
	    mime: mime_type,
	    permissions: metadata.permissions().mode(),
	    is_dir: metadata.is_dir(),
	    is_file: metadata.is_file(),
	    last_modified: metadata.modified().unwrap(),
	    last_accessed: metadata.accessed().unwrap(),
	    created: metadata.created().ok(),
	};

    let output = serde_json::to_string(&file_info).unwrap();
    //println!("output = {:?}", output);
    let output_bytes = output.into_bytes();
    let length = output_bytes.len();
    //let output_ptr = output_bytes.as_ptr();
    let output_ptr = output_bytes.as_ptr() as *mut u8;

    //println!("dont forget");
    std::mem::forget(output_bytes);
    //println!("forget");
    OutputData {
        data: output_ptr,
        len: length,
    }
}
```
