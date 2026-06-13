package walrusdb

import "encoding/binary"

func Test(name ...string) string {
	world := "World"
	if len(name) > 0 {
		world = name[0]
	}
	return "Hello" + world
}

// Record represents a single key-value entry
type Record struct {
	CRC uint32	// 4 bytes: Checksum for data integrity
	Timestamp uint64
	KeySize uint32	// length of the key
	ValueSize uint32	// length of the value
	Key	[]byte	// variable length key
	Value []byte // variable length value
}

func EncodeRecord(r Record) []byte {
	buf := make([]byte, binary.MaxVarintLen64)
	binary.AppendUvarint(buf, uint64(r.CRC))
	binary.AppendUvarint(buf, uint64(r.Timestamp))
	binary.AppendUvarint(buf, uint64(r.KeySize))
	binary.AppendUvarint(buf, uint64(r.ValueSize))
	buf = append(buf, r.Key...)
	buf = append(buf, r.Value...)

	return buf
}

