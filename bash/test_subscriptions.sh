#!/usr/bin/env bash
curl --request POST \
--data 'name=le%20guin&email=ursula_le_guin%40gmail.com' \
127.0.0.1:8000/subscriptions --verbose
