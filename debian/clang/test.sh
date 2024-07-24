#!/bin/bash

# 检查并安装clang
check_and_install_clang() {
    if ! command -v clang &> /dev/null; then
        echo "clang not found. Attempting to install..."
        if command -v apt-get &> /dev/null; then
            sudo apt-get update && sudo apt-get install -y clang
        elif command -v yum &> /dev/null; then
            sudo yum install -y clang
        elif command -v dnf &> /dev/null; then
            sudo dnf install -y clang
        elif command -v zypper &> /dev/null; then
            sudo zypper install -y clang
        elif command -v pacman &> /dev/null; then
            sudo pacman -S --noconfirm clang
        else
            echo "Unable to install clang. Please install it manually."
            exit 1
        fi
    fi
}

# 执行检查和安装
check_and_install_clang

# 创建一个简单的C程序
cat << EOF > test.c
#include <stdio.h>

int main() {
    printf("Hello, World!\n");
    return 0;
}
EOF

# 使用clang编译程序
clang test.c -o test_program

# 检查编译是否成功
if [ $? -eq 0 ]; then
    test_passed=true
else
    test_passed=false
fi

# 运行编译后的程序
./test_program

# 清理临时文件
rm test.c test_program

# 获取系统信息
os_version=$(uname -v)
kernel_version=$(uname -r)
clang_version=$(clang --version | head -n 1 | awk '{print $3}')
distro="debian"

# 生成report.json
cat << EOF > report.json
{
    "distro": "$distro",
    "os_version": "$os_version",
    "kernel_version": "$kernel_version",
    "package_name": "clang",
    "package_type": "Compiler",
    "package_version": "$clang_version",
    "test_results": [
        {
            "test_name": "Clang Compilation Test",
            "passed": $test_passed
        }
    ],
    "all_tests_passed": $test_passed
}
EOF

echo "测试完成,结果已保存到report.json"