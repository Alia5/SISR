package cli

import (
	"github.com/Alia5/SISR/cli/cmd/checkrates"
	"github.com/Alia5/SISR/cli/cmd/sisr"
	"github.com/Alia5/SISR/config"
)

type CLI struct {
	Config     config.Global         `embed:""`
	SISR       sisr.SISR             `cmd:"" help:"Run SISR (Default)" default:"withargs" name:"run"`
	CheckRates checkrates.CheckRates `cmd:"" name:"checkrates" help:"Measure Steam Input polling rate" `
}
