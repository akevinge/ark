package main

import (
	"ark-core/db"
	"ark-core/server"
)

func main() {
	if err := db.InitDbConn(); err != nil {
		panic(err)
	}

	server.Run()
}
