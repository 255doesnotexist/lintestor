#!/bin/bash

PACKAGE_NAME="ocaml"
PACKAGE_TYPE="Programming Language"
REPORT_FILE="report.json"

# 检查 OCaml 是否已安装
is_ocaml_installed() {
    dpkg -l | grep -qw $PACKAGE_NAME
    return $?
}

# 安装 OCaml 包
install_ocaml_package() {
    apt-get update
    apt-get install -y $PACKAGE_NAME ocamlbuild
    return $?
}

# 测试 OCaml 的功能
test_ocaml_functionality() {
    local initial_dir=$(pwd)
    local temp_dir=$(mktemp -d)
    local ocaml_file="${temp_dir}/hello.ml"
    local executable="${temp_dir}/hello"

    # 创建 OCaml 源文件
    cat <<EOF > "$ocaml_file"
let () = print_endline "Hello, OCaml!"
EOF

    cd "$temp_dir"
    ocamlc -o hello hello.ml

    if [[ -x "$executable" ]]; then
        local output=$("$executable")
        cd "$initial_dir"  # 返回到初始目录
        if [[ "$output" == "Hello, OCaml!" ]]; then
            return 0
        fi
    fi
    cd "$initial_dir"  # 返回到初始目录
    return 1
}

# 生成报告
generate_report() {
    local test_passed=$1
    local os_version=$(cat /proc/version)
    local kernel_version=$(uname -r)
    local package_version=$(dpkg -l | grep $PACKAGE_NAME | head -n 1 | awk '{print $3}')
    local test_name="OCaml Functionality Test"

    local report_content=$(cat <<EOF
{
    "distro": "debian",
    "os_version": "$os_version",
    "kernel_version": "$kernel_version",
    "package_name": "$PACKAGE_NAME",
    "package_type": "$PACKAGE_TYPE",
    "package_version": "$package_version",
    "test_results": [
        {
            "test_name": "$test_name",
            "passed": $test_passed
        }
    ],
    "all_tests_passed": $test_passed
}
EOF
)
    echo "$report_content" > $REPORT_FILE
}

# 主函数逻辑
main() {
    if is_ocaml_installed; then
        echo "Package $PACKAGE_NAME is installed."
    else
        echo "Package $PACKAGE_NAME is not installed. Attempting to install..."
        if install_ocaml_package; then
            echo "Package $PACKAGE_NAME installed successfully."
        else
            echo "Failed to install package $PACKAGE_NAME."
            exit 1
        fi
    fi

    if test_ocaml_functionality; then
        echo "OCaml is functioning correctly."
        generate_report true
    else
        echo "OCaml is not functioning correctly."
        generate_report false
    fi

    echo "Report generated at $REPORT_FILE"
}

# 执行主函数
main
