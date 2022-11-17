#!/bin/sh

GOOS=linux GOARCH=amd64 go build -o logger main.go

zip ark-logger.zip logger

rm -f logger