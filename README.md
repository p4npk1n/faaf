# FAAF (Firmware Analysis Assistance Framework)

## Current Status

This project is in its very early stages. As of now, the codebase is non-functional and primarily serves as a conceptual placeholder. The framework's functionality and project name are still under development and subject to change.

## Objectives

The goal is to analyze firmware extracted and decompressed from ROMs, and then store this data in a database. Ideally, the collected data would then be neatly organized and exported into formats such as HTML or Excel.

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

# Writing a analyzer

For analyzer written in Python (py) or as a shared object (so), the entry point is a function called analyzer_main. This function will receive a JSON-formatted string as its argument from faaf and should return a JSON-formatted string as its output. The output JSON must have a result key at its root.

```py
import base64
import json
def analyzer_main(json_str: str) -> str:

    jsn_dict = json.loads(json_str)
    path = jsn_dict["path"]  # If `dependencies` or `arguments` are not set in the toml configuration file, only the `path` of the file being analyzed is available by default.
    size = jsn_dict["basic_info"]["size"]

    # Perform some analysis

    rst = {
        "result": {
            # data
        }
    }

    return json.dumps(rst)
```