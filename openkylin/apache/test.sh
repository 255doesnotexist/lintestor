#!/bin/bash

PACKAGE_NAME="apache2"
APACHE_CONF_FILE="/etc/apache2/ports.conf"
DEFAULT_PORT=80

# 检查端口是否被占用
is_port_in_use() {
    local port=$1
    netstat -tuln | grep -q ":${port} "
    return $?
}

# 找到一个随机的可用端口
find_available_port() {
    local port
    while true; do
        port=$((RANDOM % 65535 + 1024))
        if ! is_port_in_use $port; then
            echo $port
            return
        fi
    done
}

# 更新 Apache 配置使用新的端口
update_apache_port() {
    local new_port=$1
    sed -i "s/Listen $DEFAULT_PORT/Listen $new_port/" $APACHE_CONF_FILE
    return $?
}

# 检查 Apache 服务是否正在运行
is_apache_active() {
    systemctl is-active --quiet apache2
    return $?
}

# 检查 Apache 服务是否启用
is_apache_enabled() {
    systemctl is-enabled --quiet apache2
    return $?
}

# 检查包是否安装
is_package_installed() {
    dpkg -l | grep -qw $PACKAGE_NAME
    return $?
}

# 安装 Apache 包
install_apache_package() {
    export DEBIAN_FRONTEND=noninteractive
    apt-get update
    apt-get install -y $PACKAGE_NAME
    return $?
}

# 主函数逻辑
main() {
    if is_package_installed; then
        echo "Package $PACKAGE_NAME is installed."
    else
        echo "Package $PACKAGE_NAME is not installed. Attempting to install..."
        if install_apache_package; then
            echo "Package $PACKAGE_NAME installed successfully."
        else
            echo "Failed to install package $PACKAGE_NAME."
            return 1
        fi
    fi

    PACKAGE_VERSION=$(dpkg -l | grep $PACKAGE_NAME | head -n 1 | awk '{print $3}')
    initial_state_active=$(is_apache_active; echo $?)
    initial_state_enabled=$(is_apache_enabled; echo $?)

    if is_apache_active; then
        echo "Apache service is running."
        exit_status=0
    else
        echo "Apache service is not running."
        if is_port_in_use $DEFAULT_PORT; then
            echo "Port $DEFAULT_PORT is in use. Finding a new port..."
            new_port=$(find_available_port)
            echo "Configuring Apache to use port $new_port..."
            update_apache_port $new_port
        fi

        systemctl start apache2
        if is_apache_active; then
            echo "Apache service started successfully."
            exit_status=0
        else
            echo "Failed to start Apache service."
            exit_status=1
        fi
    fi

    if [ "$initial_state_active" -eq 0 ]; then
        systemctl start apache2
    else
        systemctl stop apache2
    fi

    if [ "$initial_state_enabled" -eq 0 ]; then
        systemctl enable apache2
    else
        systemctl disable apache2
    fi

    echo "Apache service state has been restored."
}

# 执行主函数
main

return $exit_status
