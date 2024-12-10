export DEBIAN_FRONTEND=noninteractive
apt update
apt upgrade -y
apt-get install -y sudo curl apt-transport-https ca-certificates gnupg lsb-release netcat-openbsd