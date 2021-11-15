package main

import (
	"fmt"
	"math/rand"
	"strconv"
	"time"

	"github.com/blevesearch/bleve/v2"
	"github.com/blevesearch/bleve/v2/analysis/analyzer/keyword"
	"github.com/blevesearch/bleve/v2/index/upsidedown"
	"github.com/blevesearch/bleve/v2/index/upsidedown/store/gtreap"
)

type Data struct {
	ID   uint64    `json:"id"`
	Name string    `json:"name"`
	Time time.Time `json:"time"`
}

type IdxData struct {
	Data      interface{} `json:"data"`
	ArrivedAt time.Time   `json:"arrivedAt"`
}

func main() {
	mapping := bleve.NewIndexMapping()
	mapping.DefaultAnalyzer = keyword.Name
	idx, err := bleve.NewUsing("", mapping, upsidedown.Name, gtreap.Name, nil)
	if err != nil {
		panic(err)
	}
	defer func() {
		if err := idx.Close(); err != nil {
			panic(err)
		}
	}()

	storage := make(map[string]interface{})

	go func() {
		for range time.Tick(time.Millisecond * 50) {
			id := rand.Uint64()
			idStr := strconv.FormatUint(id, 10)
			data := &Data{
				ID:   id,
				Name: strconv.FormatInt(rand.Int63n(10), 10),
				Time: time.Now(),
			}
			idxData := &IdxData{
				Data:      data,
				ArrivedAt: time.Now(),
			}
			if err := idx.Index(idStr, idxData); err != nil {
				panic(err)
			}
			storage[idStr] = data
		}
	}()

	offset := 0

	for range time.Tick(time.Millisecond * 500) {
		q := bleve.NewQueryStringQuery("+data.name:1")
		req := bleve.NewSearchRequest(q)
		req.Size = 1000
		req.From = offset
		req.SortBy([]string{"arrivedAt"})
		res, err := idx.Search(req)
		if err != nil {
			panic(err)
		}
		if res.Hits.Len() > 0 {
			offset += res.Hits.Len()
			fmt.Printf("\n---- NEW RESULTS -----\n")
			for _, hit := range res.Hits {
				data := storage[hit.ID].(*Data)
				fmt.Println(data.Name, data.Time)
			}
		}
	}
}
