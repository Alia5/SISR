package viiperdevice

import (
	"bufio"
	"encoding"
	"io"

	"github.com/Alia5/VIIPER/device/dualsense"
)

func readDualSenseFeedback(r *bufio.Reader) (encoding.BinaryUnmarshaler, error) {
	var b [7]byte
	if _, err := io.ReadFull(r, b[:]); err != nil {
		return nil, err
	}

	msg := new(dualsense.OutputState)
	if err := msg.UnmarshalBinary(b[:]); err != nil {
		return nil, err
	}

	return msg, nil
}
