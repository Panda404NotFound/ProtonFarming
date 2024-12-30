#!/bin/bash

# Устанавливаем необходимые пакеты
sudo apt update
sudo apt upgrade -y
sudo apt-get install -y pkg-config libssl-dev curl git xclip expect cargo

sleep 1

echo "YOU BEST BRO!"

# Запуск скрипта keyadd.sh
chmod +x keyadd.sh
./keyadd.sh