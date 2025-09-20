#!/bin/bash
# set -x
set -e

readonly script_dir="$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
readonly bin_file="my-arbolitos"
readonly error_bin_file_not_found=80

if [[ ! -f "${script_dir}/target/release/${bin_file}" ]]; then
    echo "error: binary file ${bin_file} not found" >&2
    exit ${error_bin_file_not_found}
fi

"${script_dir}/target/release/${bin_file}" $*

exit