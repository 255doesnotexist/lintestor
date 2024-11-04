#!/bin/bash

PACKAGE_NAME="httpd"
APACHE_CONF_FILE="/etc/httpd/conf/ports.conf"
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
    systemctl is-active --quiet httpd
    return $?
}

# 检查 Apache 服务是否启用
is_apache_enabled() {
    systemctl is-enabled --quiet httpd
    return $?
}

# 检查包是否安装
is_package_installed() {
    rpm -qa | grep -qw $PACKAGE_NAME
    return $?
}

# 安装 Apache 包
install_apache_package() {
    dnf install -y $PACKAGE_NAME
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

    PACKAGE_VERSION=$(rpm -qi $PACKAGE_NAME | grep Version | awk '{print $2}')
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

        systemctl start httpd
        if is_apache_active; then
            echo "Apache service started successfully."
            exit_status=0
        else
            echo "Failed to start Apache service."
            exit_status=1
        fi
    fi

    if [ "$initial_state_active" -eq 0 ]; then
        systemctl start httpd
    else
        systemctl stop httpd
    fi

    if [ "$initial_state_enabled" -eq 0 ]; then
        systemctl enable httpd
    else
        systemctl disable httpd
    fi

    echo "Apache service state has been restored."
}

# 执行主函数
main

return $exit_status