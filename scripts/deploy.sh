#!/bin/bash

# Usage ./scripts/deploy.sh <debug | release> <IP>
target=target/$1/spi_laser_emu
chmod 777 $target
scp $target pi@$2:freeform/