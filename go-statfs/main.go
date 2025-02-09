package main

import (
	"encoding/json"
	"fmt"
	_ "net" // for dynamic linking
	"os"
	"os/signal"
	"syscall"
)

func main() {
	rootPath := "/"
	var statfs syscall.Statfs_t
	err := syscall.Statfs(rootPath, &statfs)
	if err != nil {
		fmt.Println("Error:", err)
		return
	}

	// Convert struct to a JSON-friendly format
	data := map[string]interface{}{
		"bavail":  statfs.Bavail,
		"bfree":   statfs.Bfree,
		"blocks":  statfs.Blocks,
		"bsize":   statfs.Bsize,
		"ffree":   statfs.Ffree,
		"files":   statfs.Files,
		"flags":   statfs.Flags,
		"frsize":  statfs.Frsize,
		"fsid":    []int32{int32(statfs.Fsid.X__val[0]), int32(statfs.Fsid.X__val[1])},
		"namelen": statfs.Namelen,
		"spare":   statfs.Spare,
		"type":    statfs.Type,
	}

	// Convert to JSON
	jsonData, err := json.MarshalIndent(data, "", "  ")
	if err != nil {
		fmt.Println("JSON Encoding Error:", err)
		return
	}

	// Print JSON
	fmt.Println(string(jsonData))

	// Create a channel to receive OS signals
	sigs := make(chan os.Signal, 1)
	signal.Notify(sigs, syscall.SIGINT, syscall.SIGTERM)

	// Block forever waiting for a signal
	<-sigs

}
