package handlers

import (
	"os"
	"path/filepath"
)

func readFirst(candidates ...string) ([]byte, string, error) {
	var last error
	for _, p := range candidates {
		data, err := os.ReadFile(p)
		if err == nil {
			abs, _ := filepath.Abs(p)
			return data, abs, nil
		}
		last = err
	}
	return nil, "", last
}
