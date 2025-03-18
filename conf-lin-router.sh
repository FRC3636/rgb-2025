#!/usr/bin/env bash

IFACE=$1

# Bring the interface down
echo "Bringing $IFACE down..."
ip link set $IFACE down

# Set the IP address and routable subnet of the interface
echo "Setting IP address and subnet..."
ip addr add 10.36.36.1/24 dev $IFACE

# Enable IP forwarding
echo "Enabling IP forwarding..."
sysctl -w net.ipv4.ip_forward=1 > /dev/null

# Enable NAT from the robot subnet to our upstream connection
echo "Enabling NAT..."
iptables -t nat -s 10.36.36.0/24 -A POSTROUTING -j MASQUERADE

# Bring the interface up
echo "Bringing $IFACE up..."
ip link set $IFACE up

echo "Done configuring $IFACE"