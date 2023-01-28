package model

import (
	"time"

	"github.com/kamva/mgm/v3"
	"go.mongodb.org/mongo-driver/bson/primitive"
)

type Log struct {
	mgm.IDField `bson:",inline"`
	Location    string `bson:"location"`
	DeviceCount uint32 `bson:"device_count"`
}

func NewLog(location string, deviceCount uint32, createdAt time.Time) *Log {
	return &Log{
		IDField:     mgm.IDField{ID: primitive.NewObjectIDFromTimestamp(createdAt)},
		Location:    location,
		DeviceCount: deviceCount,
	}
}
