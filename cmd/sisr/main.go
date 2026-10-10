package main

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"unicode/utf8"

	"github.com/alecthomas/kong"
	kongtoml "github.com/alecthomas/kong-toml"
	kongyaml "github.com/alecthomas/kong-yaml"

	"github.com/Alia5/SISR/cli"
	"github.com/Alia5/SISR/helper"
	"github.com/Alia5/SISR/logging"
	"github.com/Alia5/SISR/meta"
)

func main() {
	userCfg := findUserConfig(os.Args[1:])
	jsonPaths, yamlPaths, tomlPaths := configCandidatePaths(userCfg)

	var cli cli.CLI
	parser := kong.Must(&cli,
		kong.Name("SISR"),
		kong.Description(meta.Description()),
		kong.UsageOnError(),
		kong.Configuration(kong.JSON, jsonPaths...),
		kong.Configuration(kongyaml.Loader, yamlPaths...),
		kong.Configuration(kongtoml.Loader, tomlPaths...),
	)
	expandOneCharacterAliases(parser.Model, os.Args[1:])
	ctx, err := parser.Parse(os.Args[1:])
	parser.FatalIfErrorf(err)

	applyPlatformStartup(cli.Config)

	if cli.Config.Log.File == "" { // nolint
		dataDir, err := helper.GetDataDir()
		if err != nil {
			fmt.Fprintln(os.Stderr, "failed to determine SISR data directory:", err)
		} else {
			cli.Config.Log.File = filepath.Join(dataDir, "SISR.log") // nolint
		}
	}
	if cli.Config.Log.File != "" { // nolint
		_ = os.MkdirAll(filepath.Dir(cli.Config.Log.File), 0o755) // nolint
	}

	_, closeFiles, err := logging.SetupLogger(cli.Config.Log.Level, cli.Config.Log.File) // nolint
	if err != nil {
		fmt.Fprintln(os.Stderr, "failed to setup logger:", err)
		os.Exit(2)
	}
	defer func() {
		for _, c := range closeFiles {
			_ = c.Close()
		}
	}()

	ctx.Bind(cli.Config)

	err = ctx.Run()
	ctx.FatalIfErrorf(err)
}

func expandOneCharacterAliases(model *kong.Application, args []string) {
	longNames := map[string]string{}
	_ = kong.Visit(model, func(node kong.Visitable, next kong.Next) error {
		flag, ok := node.(*kong.Flag)
		if ok {
			for _, alias := range flag.Aliases {
				if utf8.RuneCountInString(alias) == 1 {
					longNames["--"+alias] = "--" + flag.Name
				}
			}
		}
		return next(nil)
	})
	for i, arg := range args {
		name, _, _ := strings.Cut(arg, "=")
		longName, ok := longNames[name]
		if ok {
			args[i] = longName + strings.TrimPrefix(arg, name)
		}
	}
}

func findUserConfig(args []string) string {
	for i, a := range args {
		if strings.HasPrefix(a, "--config=") {
			return a[len("--config="):]
		}
		if a == "--config" && i+1 < len(args) {
			return args[i+1]
		}
	}
	return os.Getenv("SISR_CONFIG")
}

func configCandidatePaths(userPath string) (jsonPaths, yamlPaths, tomlPaths []string) {
	add := func(slice *[]string, p string) { *slice = append(*slice, p) }

	if userPath != "" {
		switch ext := filepath.Ext(userPath); ext {
		case ".json":
			add(&jsonPaths, userPath)
		case ".yaml", ".yml":
			add(&yamlPaths, userPath)
		case ".toml":
			add(&tomlPaths, userPath)
		default:
			add(&jsonPaths, userPath)
		}
	}

	wd, _ := os.Getwd()
	for _, base := range []string{"github.com/Alia5/SISR", "SISR", "config"} {
		add(&jsonPaths, filepath.Join(wd, base+".json"))
		add(&yamlPaths, filepath.Join(wd, base+".yaml"))
		add(&yamlPaths, filepath.Join(wd, base+".yml"))
		add(&tomlPaths, filepath.Join(wd, base+".toml"))
	}

	return
}
