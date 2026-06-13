package main

import (
    "fmt"
    "github.com/themayukhbasu/walrusdb"
)

func main() {
	fmt.Println("Foo Bar")
    temp := walrusdb.Test("Baz")
    fmt.Printf("Got value: %s\n", temp)

    key := "hello"
    key_buf := []byte(key)

    value := "world"
    value_buf := []byte(value)

    record := walrusdb.Record{
        CRC: 10,
        Timestamp: 1000,
        KeySize: uint32(len(key_buf)),
        ValueSize: uint32(len(value_buf)),
        Key: key_buf,
        Value: value_buf, 
    }
    encoded := walrusdb.EncodeRecord(record)
    fmt.Println(encoded)
}