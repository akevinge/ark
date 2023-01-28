package db

import (
	"ark-core/config"
	"time"

	"github.com/kamva/mgm/v3"
	"go.mongodb.org/mongo-driver/mongo/options"
)

func InitDbConn() error {
	return mgm.SetDefaultConfig(&mgm.Config{CtxTimeout: 10 * time.Second}, config.Config.DbName, options.Client().ApplyURI(config.Config.DbURL))
}
