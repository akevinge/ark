package main

import (
	"context"
	"time"

	"github.com/aws/aws-lambda-go/lambda"
	"github.com/aws/aws-sdk-go/aws"
	"github.com/aws/aws-sdk-go/aws/session"
	"github.com/aws/aws-sdk-go/service/dynamodb"
	"github.com/aws/aws-sdk-go/service/dynamodb/dynamodbattribute"
)

type Request struct {
	Location    string `json:"location"`
	DeviceCount uint64 `json:"device_count"`
}

var db *dynamodb.DynamoDB

var TableName = aws.String("Logs")

func Handler(ctx context.Context, req Request) error {

	item, err := dynamodbattribute.MarshalMap(
		struct {
			Location    *string
			CreatedAt   int64
			DeviceCount uint64
		}{
			Location:    aws.String(req.Location),
			CreatedAt:   time.Now().Unix(),
			DeviceCount: *aws.Uint64(req.DeviceCount),
		},
	)

	if err != nil {
		return err
	}

	_, err = db.PutItem(
		&dynamodb.PutItemInput{
			TableName: TableName,
			Item:      item,
		},
	)

	if err != nil {
		return err
	}

	return nil
}

func main() {
	sess := session.Must(session.NewSessionWithOptions(session.Options{SharedConfigState: session.SharedConfigEnable}))

	db = dynamodb.New(sess)

	lambda.Start(Handler)
}
