package viiper

import (
	"context"
	"net/http"

	"github.com/Alia5/SISR/cmd"
	"github.com/Alia5/VIIPER/viipertypes"
	"github.com/danielgtaylor/huma/v2"
)

type VIIPERStatusResponse struct {
	Body VIIPERStatus
}

type VIIPERStatus struct {
	Status  *viipertypes.PingResponse `json:"status,omitempty"`
	Address string                    `json:"address,omitempty"`
}

func Register(a huma.API, c *cmd.SISRContext) {
	huma.Register(a, huma.Operation{
		Method: http.MethodGet,
		Path:   "/api/v1/viiper/status",
	}, status(c))
	huma.Register(a, huma.Operation{
		Method: http.MethodPost,
		Path:   "/api/v1/viiper/ping",
	}, ping(c))
}

func status(c *cmd.SISRContext) func(ctx context.Context, req *struct{}) (*VIIPERStatusResponse, error) {
	return func(ctx context.Context, req *struct{}) (*VIIPERStatusResponse, error) {
		return &VIIPERStatusResponse{
			Body: VIIPERStatus{
				Status:  c.ViiperBridge.Info(),
				Address: c.ViiperBridge.ResolvedAddressAndPort(),
			},
		}, nil
	}
}

func ping(c *cmd.SISRContext) func(ctx context.Context, req *struct{}) (*VIIPERStatusResponse, error) {
	return func(ctx context.Context, req *struct{}) (*VIIPERStatusResponse, error) {
		resp, err := c.ViiperBridge.Ping(ctx)
		if err != nil {
			return nil, err
		}
		return &VIIPERStatusResponse{
			Body: VIIPERStatus{
				Status:  resp,
				Address: c.ViiperBridge.ResolvedAddressAndPort(),
			},
		}, nil
	}
}
