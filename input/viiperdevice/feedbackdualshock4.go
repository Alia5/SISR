package viiperdevice

import (
	"bufio"
	"encoding"
	"io"

	"github.com/Alia5/VIIPER/device/dualshock4"
)

func readDualShock4Feedback(r *bufio.Reader) (encoding.BinaryUnmarshaler, error) {
	var b [8]byte
	if _, err := io.ReadFull(r, b[:]); err != nil {
		return nil, err
	}

	msg := new(dualshock4.OutputState)
	if err := msg.UnmarshalBinary(b[:]); err != nil {
		return nil, err
	}

	return msg, nil
}
