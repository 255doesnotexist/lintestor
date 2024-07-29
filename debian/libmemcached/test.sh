#!/bin/bash

# 定义包的详细信息
PACKAGE_NAME="libmemcached11t64"
PACKAGE_SHOW_NAME="libmemcached"
PACKAGE_DEV_NAME="libmemcached-dev"
MEMCACHED_PACKAGE="memcached"
PACKAGE_TYPE="Caching Library"
REPORT_FILE="report.json"

# 检查包是否已安装
is_package_installed() {
    dpkg -l | grep -qw $1
    return $?
}

# 安装 libmemcached 和 memcached 包
install_libmemcached_package() {
    apt-get update
    apt-get install -y $PACKAGE_NAME $PACKAGE_DEV_NAME $MEMCACHED_PACKAGE
    return $?
}

# 启动 memcached 服务
start_memcached() {
    memcached -d -m 64 -p 11211 -u memcache
    return $?
}

# 停止 memcached 服务
stop_memcached() {
    pkill memcached
    return $?
}

# 测试 libmemcached 的功能
test_libmemcached_functionality() {
    local initial_dir=$(pwd)
    local temp_dir=$(mktemp -d)
    local test_file="${temp_dir}/test_libmemcached.c"
    local executable="${temp_dir}/test_libmemcached"
    local memcached_server="localhost"

    # 写一个简单的 C 程序来测试 libmemcached 功能
    cat <<EOF > "$test_file"
#include <stdio.h>
#include <string.h>
#include <libmemcached/memcached.h>

int main() {
    memcached_st *memc;
    memcached_server_st *servers;
    memcached_return rc;
    char *retrieved_value;
    size_t value_length;
    uint32_t flags;

    memc = memcached_create(NULL);
    servers = memcached_server_list_append(NULL, "$memcached_server", 11211, &rc);
    rc = memcached_server_push(memc, servers);
    if (rc != MEMCACHED_SUCCESS) {
        fprintf(stderr, "Couldn't add server: %s\\n", memcached_strerror(memc, rc));
        return 1;
    }

    const char *key = "key";
    const char *value = "value";
    rc = memcached_set(memc, key, strlen(key), value, strlen(value), (time_t)0, (uint32_t)0);
    if (rc != MEMCACHED_SUCCESS) {
        fprintf(stderr, "Couldn't store key: %s\\n", memcached_strerror(memc, rc));
        return 1;
    }

    retrieved_value = memcached_get(memc, key, strlen(key), &value_length, &flags, &rc);
    if (rc != MEMCACHED_SUCCESS) {
        fprintf(stderr, "Couldn't retrieve key: %s\\n", memcached_strerror(memc, rc));
        return 1;
    }

    printf("Retrieved value: %s\\n", retrieved_value);
    free(retrieved_value);
    memcached_free(memc);

    return 0;
}
EOF

    # 编译 C 程序
    gcc "$test_file" -o "$executable" -lmemcached

    # 检查可执行文件是否创建并运行无错误
    if [[ -x "$executable" ]]; then
        "$executable"
        local result=$?
        cd "$initial_dir"  # 返回到初始目录
        return $result
    else
        cd "$initial_dir"  # 返回到初始目录
        return 1
    fi
}

# 生成报告
generate_report() {
    local test_passed=$1
    local os_version=$(cat /proc/version)
    local kernel_version=$(uname -r)
    local package_version=$(dpkg -l | grep $PACKAGE_NAME | head -n 1 | awk '{print $3}')
    local test_name="libmemcached Functionality Test"

    local report_content=$(cat <<EOF
{
    "distro": "debian",
    "os_version": "$os_version",
    "kernel_version": "$kernel_version",
    "package_name": "$PACKAGE_SHOW_NAME",
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
    # 检查包是否已安装
    if is_package_installed $PACKAGE_NAME; then
        echo "Package $PACKAGE_NAME is installed."
    else
        echo "Package $PACKAGE_NAME is not installed. Attempting to install..."
        if install_libmemcached_package; then
            echo "Package $PACKAGE_NAME installed successfully."
        else
            echo "Failed to install package $PACKAGE_NAME."
            exit 1
        fi
    fi

    # 启动 memcached 服务
    if command -v memcached &> /dev/null; then
        if start_memcached; then
            echo "memcached service started successfully."
        else
            echo "Failed to start memcached service."
            exit 1
        fi
    else
        echo "memcached command not found. Please ensure memcached is installed correctly."
        exit 1
    fi

    # 测试 libmemcached 的功能
    if test_libmemcached_functionality; then
        echo "libmemcached is functioning correctly."
        generate_report true
    else
        echo "libmemcached is not functioning correctly."
        generate_report false
    fi

    # 停止 memcached 服务
    stop_memcached
    echo "memcached service stopped."

    echo "Report generated at $REPORT_FILE"
}

# 执行主函数
main
