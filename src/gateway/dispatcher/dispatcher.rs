use pyo3::prelude::*;
use pyo3::types::PyTuple;
use serde_json::Value;
use libloading::{Library, Symbol};
use crate::gateway::dispatcher::error::Error;
extern crate libc;

#[repr(C)]
pub struct OutputData {
    data: *mut u8,
    len: usize,
}

//fn execute_shared_object(script_dir: &str, script_name: &str, args: &Value) -> Result<String, Error> {
//    let lib_path = format!("{}/lib{}.so", script_dir, script_name);
//    let data_string = args.to_string();
//    let handler = std::thread::spawn(move || {
//        let lib = unsafe { Library::new(&lib_path).unwrap() };
//        type AnalyzerMainFunc = unsafe extern "C" fn(*const u8, usize) -> *mut u8;
//        let func: Symbol<AnalyzerMainFunc> = unsafe { lib.get(b"analyzer_main").unwrap() };
//        let data_bytes = data_string.as_bytes();
//        let result_ptr = unsafe {func(data_bytes.as_ptr(), data_bytes.len()) };
//        unsafe {String::from_raw_parts(result_ptr, data_bytes.len(), data_bytes.len())}
//    });
//
//    handler.join().map_err(|_| Error::SoPanicError())
//}


fn execute_shared_object(script_dir: &str, script_name: &str, args: &Value) -> Result<String, Error> {
    std::panic::catch_unwind(|| {
        let lib_path = format!("{}/lib{}.so", script_dir, script_name);
        let lib = unsafe { Library::new(&lib_path) }?;
        type AnalyzerMainFunc = unsafe extern "C" fn(*const u8, usize) -> OutputData;
        let func: Symbol<AnalyzerMainFunc> = unsafe { lib.get(b"analyzer_main")? };

        let json_string = unsafe {
            let data_string = args.to_string();
            let data_bytes = data_string.as_bytes();
            let output_data = func(data_bytes.as_ptr(), data_bytes.len());
            let result_ptr = output_data.data;
            let result_len = output_data.len;
            let result_string = String::from_raw_parts(result_ptr, result_len, result_len);
            //libc::free(result_ptr as *mut libc::c_void);
            result_string
        };

        Ok(json_string)
    }).map_err(|_| Error::SoPanicError())?
}

//fn execute_shared_object(script_dir: &str, script_name: &str, args: &Value) -> Result<String, Error> {
//    std::panic::catch_unwind(|| {
//        let lib_path = format!("{}/lib{}.so", script_dir, script_name);
//        let lib = unsafe { Library::new(&lib_path) }?;
//        type AnalyzerMainFunc = unsafe extern "C" fn(*const u8, usize) -> *mut u8;
//        let func: Symbol<AnalyzerMainFunc> = unsafe { lib.get(b"analyzer_main")? };
//
//        let json_string = unsafe {
//            let data_string = args.to_string();
//            let data_bytes = data_string.as_bytes();
//            let result_ptr = func(data_bytes.as_ptr(), data_bytes.len());
//            String::from_raw_parts(result_ptr, data_bytes.len(), data_bytes.len())
//        };
//
//        Ok(json_string)
//    }).map_err(|_| Error::SoPanicError())?
//}

fn execute_python(script_dir: &str, script_name: &str, args: &Value) -> Result<String, Error> {
    pyo3::Python::with_gil(|py| {
        let sys = py.import("sys")?;
        let os_path = py.import("os.path")?;
        let path = os_path.call_method1("join", (script_dir, script_name))?;
        sys.getattr("path")?.call_method1("append", (script_dir,))?;

        let script = py.import(script_name)?;
        let func = script.getattr("analyzer_main")?;
        let py_data = args.to_string().into_py(py);
        let args_tuple = PyTuple::new(py, &[py_data]);
        let py_result = func.call1(args_tuple)?;
        let result_string = py_result.extract::<String>()?;
        Ok(result_string)
    })
}

fn execute_sh(script_dir: &str, script_name: &str, args: &Value) -> Result<String, Error> {
    let script_path = format!("{}/{}.sh", script_dir, script_name);
    let data_string = args.to_string();

    let output: std::process::Output = std::process::Command::new("sh")
        .arg(&script_path)
        .arg(data_string)
        .output()?;

    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout).into_owned();
        Ok(output_str)
    } else {
        let error_str = String::from_utf8_lossy(&output.stderr).into_owned();
        Ok(format!( "{{ \"error\": \"{}\" }} ", error_str))
    }
}

pub fn execute_analyzer(script_dir: &std::path::Path, analyzer_name: &str, extension: &str, args: &Value) -> Result<Value, Error> {
    let script_dir_str = script_dir.to_str().unwrap();


    match extension {
        "py" => {
            let json_string = execute_python(script_dir_str, analyzer_name, args)?;
            let json_value: Value = serde_json::from_str(&json_string)?;
            Ok(json_value)
        },
        "so" => {
            let json_string = execute_shared_object(script_dir_str, analyzer_name, args)?;
            let json_value: Value = serde_json::from_str(&json_string)?;
            Ok(json_value)
        },
        "sh" => {
            let json_string = execute_sh(script_dir_str, analyzer_name, args)?;
            let json_value: Value = serde_json::from_str(&json_string)?;
            Ok(json_value)
        },
        _ => return Err(Error::UndefinedExtensionError()),
    }
}
