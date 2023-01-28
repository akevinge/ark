package config

import (
	"log"

	"github.com/caarlos0/env/v6"
	"github.com/joho/godotenv"
)

var Config = loadConfig()

type config struct {
	DbName string `env:"DB_NAME,required"`
	DbURL  string `env:"MONGO_CONNECTION_STRING,required"`
}

func loadConfig() config {
	err := godotenv.Load()
	if err != nil {
		log.Printf("Unable to load .env file: %v", err.Error())
	}

	cfg := config{}

	err = env.Parse(&cfg)
	if err != nil {
		log.Fatalf("Unable to load .env file: %v", err.Error())
	}

	return cfg
}
