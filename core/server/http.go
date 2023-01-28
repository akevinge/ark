package server

import (
	"github.com/gin-gonic/gin"
)

func Run() {
	router := gin.Default()

	router.POST("/log", createLogHandler())

	router.Run()
}
