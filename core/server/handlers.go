package server

import (
	"ark-core/model"
	"net/http"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/kamva/mgm/v3"
)

type createLogBody struct {
	Location    string `json:"location"`
	DeviceCount uint32 `json:"device_count"`
	CreatedAt   uint64 `json:"created_at"`
}

func createLogHandler() gin.HandlerFunc {
	return func(c *gin.Context) {
		var req createLogBody
		c.BindJSON(&req)

		newLog := model.NewLog(req.Location, req.DeviceCount, time.Unix(int64(req.CreatedAt), 0))

		mgm.Coll(newLog).Create(newLog)

		c.Status(http.StatusCreated)
	}
}
