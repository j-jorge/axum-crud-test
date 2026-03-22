#!/bin/bash

# echo | openssl genrsa -aes256 -out certificates/localhost.pass.key 4096
# openssl rsa -in certificates/localhost.pass.key -out certificates/localhost.key
# rm certificates/localhost.pass.key

# openssl req -nodes -new -key certificates/localhost.key -out certificates/localhost.csr
# openssl x509 -req -sha256 -days 365 -in certificates/localhost.csr -signkey certificates/localhost.key -out certificates/localhost.crt

openssl req -noenc -days 365 -new -x509 \
        -subj "/CN=localhost" \
        -keyout certificates/localhost.key \
        -out certificates/localhost.crt
