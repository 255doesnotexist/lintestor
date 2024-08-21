#!/bin/bash

PACKAGE_NAME="ocaml"

# 检查 OCaml 是否已安装
is_ocaml_installed() {
    dpkg -l | grep -qw $PACKAGE_NAME
    return $?
}

# 安装 OCaml 包
install_ocaml_package() {
    export DEBIAN_FRONTEND=noninteractive
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
            return 1
        fi
    fi

    PACKAGE_VERSION=$(dpkg -l | grep $PACKAGE_NAME | head -n 1 | awk '{print $3}')

    if test_ocaml_functionality; then
        echo "OCaml is functioning correctly."
        return 0
    else
        echo "OCaml is not functioning correctly."
        return 1
    fi
}

# 执行主函数
main
