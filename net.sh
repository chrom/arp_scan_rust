#!/usr/bin/zsh

echo "# Network devices"
ip link list

echo -e "\n# Route table"
ip route list

echo -e "\n# iptables rules"
sudo iptables --list-rules