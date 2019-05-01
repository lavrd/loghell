package main

import (
	"context"
	"fmt"
	"io/ioutil"
	"log"
	"time"

	"nhooyr.io/websocket"
)

func main() {
	ctx, cancel := context.WithTimeout(context.Background(), time.Second)
	defer cancel()

	c, r, err := websocket.Dial(ctx, "ws://127.0.0.1:3032?rule=!level@debug", websocket.DialOptions{})
	if err != nil {
		log.Fatal(err)
	}
	defer c.Close(websocket.StatusInternalError, "the sky is falling")

	fmt.Println(r.StatusCode)

	// ctx, cancel = context.WithTimeout(context.Background(), time.Second)
	// defer cancel()

	for {
		_, reader, err := c.Reader(context.Background())
		if err != nil {
			log.Fatal(err)
		}

		buff, err := ioutil.ReadAll(reader)
		if err != nil {
			log.Fatal(err)
		}

		fmt.Println(string(buff))
	}

	c.Close(websocket.StatusNormalClosure, "")
}

// todo at first need to prepare log for every ws client and then send to clients
