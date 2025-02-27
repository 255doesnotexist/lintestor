#!/bin/bash

# 检查并安装clang
check_and_install_clang() {
    if ! command -v clang &> /dev/null; then
        echo "clang not found. Attempting to install..."
        if command -v dnf &> /dev/null; then
            sudo dnf install -y clang
        elif command -v yum &> /dev/null; then
            sudo yum install -y clang
        elif command -v zypper &> /dev/null; then
            sudo zypper install -y clang
        elif command -v pacman &> /dev/null; then
            sudo pacman -S --noconfirm clang
        else
            echo "Unable to install clang. Please install it manually."
            return 1
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
sudo clang test.c -o test_program

# 检查编译是否成功
if [ $? -eq 0 ]; then
    exit_status=0
else
    exit_status=1
fi

# 运行编译后的程序
./test_program

# 清理临时文件
rm test.c test_program

PACKAGE_VERSION=$(sudo clang --version | grep -oP "version\W?\K.*")

return $exit_status
