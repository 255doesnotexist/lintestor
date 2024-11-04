#!/bin/bash

# 定义包的详细信息
PACKAGE_NAME="libmemcached"
PACKAGE_DEV_NAME="libmemcached-devel"
MEMCACHED_PACKAGE="memcached"

# 检查包是否已安装
is_package_installed() {
    rpm -q $1 &> /dev/null
    return $?
}

# 安装 libmemcached 和 memcached 包
install_libmemcached_package() {
    dnf install -y $PACKAGE_NAME $PACKAGE_DEV_NAME $MEMCACHED_PACKAGE
    return $?
}

# 启动 memcached 服务
start_memcached() {
    systemctl enable memcached
    systemctl start memcached
    return $?
}

# 停止 memcached 服务
stop_memcached() {
    systemctl stop memcached
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
            return 1
        fi
    fi

    PACKAGE_VERSION=$(rpm -q --qf "%{VERSION}\n" $PACKAGE_NAME)
    # 启动 memcached 服务
    if command -v memcached &> /dev/null; then
        if start_memcached; then
            echo "memcached service started successfully."
        else
            echo "Failed to start memcached service."
            return 1
        fi
    else
        echo "memcached command not found. Please ensure memcached is installed correctly."
        return 1
    fi

    # 测试 libmemcached 的功能
    if test_libmemcached_functionality; then
        echo "libmemcached is functioning correctly."
        return 0
    else
        echo "libmemcached is not functioning correctly."
        return 0
    fi

    # 停止 memcached 服务
    stop_memcached
    echo "memcached service stopped."
}

# 执行主函数
main