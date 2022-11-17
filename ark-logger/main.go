package main

import (
	"context"
	"encoding/json"
	"net/http"

	"github.com/aws/aws-lambda-go/events"
	"github.com/aws/aws-lambda-go/lambda"
	"github.com/aws/aws-sdk-go/aws"
	"github.com/aws/aws-sdk-go/aws/session"
	"github.com/guregu/dynamo"
)

type LoggerRequest struct {
	// Unix epoch time
	CreatedAt   *int64  `json:"created_at"`
	Location    *string `json:"location"`
	DeviceCount *int    `json:"device_count"`
}

type LoggerInfo struct {
	// Unix epoch time
	CreatedAt   int64
	Location    string
	DeviceCount int `dynamo:"DeviceCount"`
}

// Response for the code
type Response struct {
	Response string `json:"response"`
}

var db dynamo.Table

func Handler(ctx context.Context, request events.LambdaFunctionURLRequest) (events.LambdaFunctionURLResponse, error) {
	//log.Println(request.Body)
	if request.RequestContext.HTTP.Method == http.MethodGet {
		return events.LambdaFunctionURLResponse{Body: "Function is online", StatusCode: 200}, nil
	} else if request.RequestContext.HTTP.Method == http.MethodPost {

		var req LoggerRequest
		err := json.Unmarshal([]byte(request.Body), &req)

		if err != nil || req.DeviceCount == nil || req.Location == nil || req.CreatedAt == nil {
			return events.LambdaFunctionURLResponse{Body: "bad input", StatusCode: http.StatusBadRequest}, err
		}
		// Create dynamo struct and pass in information
		loggerEntry := LoggerInfo{
			CreatedAt:   *req.CreatedAt,
			Location:    *req.Location,
			DeviceCount: *req.DeviceCount,
		}

		err = PutData(db, loggerEntry)

		if err != nil {
			return events.LambdaFunctionURLResponse{Body: "Not Put data in url", StatusCode: http.StatusBadRequest}, nil
		}
		return events.LambdaFunctionURLResponse{Body: "Success!", StatusCode: http.StatusOK}, nil
	}

	return events.LambdaFunctionURLResponse{Body: request.Body, StatusCode: http.StatusMethodNotAllowed}, nil
}

func PutData(table dynamo.Table, data LoggerInfo) error {
	err := table.Put(data).Run()
	return err
}

func main() {
	sess := session.Must(session.NewSessionWithOptions(session.Options{SharedConfigState: session.SharedConfigEnable}))
	tableName := "Logs"
	config := &aws.Config{Region: aws.String("us-east-1")}
	db = dynamo.New(sess, config).Table(tableName)

	lambda.Start(Handler)
}
