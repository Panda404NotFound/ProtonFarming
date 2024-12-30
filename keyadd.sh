#!/usr/bin/expect -f

# Получаем API_KEY из .env файла
set api_key [exec bash -c "source .env && echo \$API_KEY"]

# Запуск команды proton key:add с полученным ключом
log_user 0
spawn proton key:add "$api_key"
log_user 1

# Ожидание запроса ввода ответа
expect "Would you like to encrypt your stored keys with a password?"

# Отправка ответа "no"
send "no\r"

sleep 2

# Ожидание завершения команды
expect eof