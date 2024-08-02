#!/bin/bash

PACKAGE_NAME="apache2"
PACKAGE_SHOWNAME="apache"
PACKAGE_TYPE="Web Server"
REPORT_FILE="report.json"
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
    apt-get update
    apt-get install -y $PACKAGE_NAME
    return $?
}

# 生成报告
generate_report() {
    local test_passed=$1
    local os_version=$(cat /proc/version)
    local kernel_version=$(uname -r)
    local package_version=$(dpkg -l | grep $PACKAGE_NAME | head -n 1 | awk '{print $3}')
    local test_name="Apache Service Test"

    local report_content=$(cat <<EOF
{
    "distro": "debian",
    "os_version": "$os_version",
    "kernel_version": "$kernel_version",
    "package_name": "$PACKAGE_SHOWNAME",
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
    if is_package_installed; then
        echo "Package $PACKAGE_NAME is installed."
    else
        echo "Package $PACKAGE_NAME is not installed. Attempting to install..."
        if install_apache_package; then
            echo "Package $PACKAGE_NAME installed successfully."
        else
            echo "Failed to install package $PACKAGE_NAME."
            exit 1
        fi
    fi

    initial_state_active=$(is_apache_active; echo $?)
    initial_state_enabled=$(is_apache_enabled; echo $?)

    if is_apache_active; then
        echo "Apache service is running."
        generate_report true
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
            generate_report true
        else
            echo "Failed to start Apache service."
            generate_report false
        fi
    fi

    echo "Report generated at $REPORT_FILE"

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
