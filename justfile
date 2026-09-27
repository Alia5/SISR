set windows-shell := ["powershell.exe", "-NoProfile", "-Command"]

binary_name := "SISR"
main_pkg := "./cmd/sisr"
dist_dir := "./dist/"
target_goos := env_var_or_default("GOOS", if os_family() == "windows" { "windows" } else { "linux" })
exe_ext := if target_goos == "windows" { ".exe" } else { "" }
mkdir_p := if os_family() == "windows" { "New-Item -ItemType Directory -Force" } else { "mkdir -p" }
rm_rf := if os_family() == "windows" { "Remove-Item -Recurse -Force -ErrorAction 0" } else { "rm -rf" }

version := env_var_or_default("VERSION", `git describe --tags --always`)
buildType := env_var_or_default("BUILD_TYPE", "Debug")
commit := `git rev-parse --short HEAD`
build_time := if os_family() == "windows" {
	`Get-Date -Format 'yyyy-MM-ddTHH:mm:ssZ'`
} else {
	`date -u '+%Y-%m-%dT%H:%M:%SZ'`
}
ldflags_common := "-X github.com/Alia5/SISR/meta.Version=" + version + " -X github.com/Alia5/SISR/meta.Commit=" + commit + " -X github.com/Alia5/SISR/meta.Date=" + build_time
ldflags_release := "-s -w " + ldflags_common
build_path := join(dist_dir, binary_name + exe_ext)

default:
	just --list


[working-directory: 'dist']
run *args: build
	{{ if os_family() == "windows" { "$env:DEV='1'; & './" + binary_name + exe_ext + "'" } else { "DEV=1 './" + binary_name + exe_ext + "'" } }} {{ args }}

[working-directory: 'dist']
run-dev *args: build
	{{ if os_family() == "windows" { "$env:DEV='1'; & './" + binary_name + exe_ext + "'" } else { "DEV=1 './" + binary_name + exe_ext + "'" } }} --api.frontend-address=localhost:5173 --api.listen-address=localhost:5746 --api.cors-origins=https://steamloopback.host,http://steamloopback.host,http://localhost:5173 {{ args }}

win-resource:
	goversioninfo -64 -o cmd/sisr/resource.syso versioninfo.json

[arg("type", long="type", help="Build type (Debug/Release)")]
build-sdl type="Debug":
	{{ if os_family() == "windows" { "if (!(Test-Path deps/SDL/include/gameinput.h)) { python deps/SDL/build-scripts/download-gameinput-sdk.py -o deps/SDL }" } else { "true" } }}
	{{
		if os_family() == "windows" {
			"if (!(Test-Path deps/SDL/build)) { cmake -S deps/SDL -B deps/SDL/build -DSDL_TEST_LIBRARY=OFF -DSDL_TESTS=OFF -DSDL_INSTALL_TESTS=OFF }"
		} else {
			"[ -d deps/SDL/build ] || cmake -S deps/SDL -B deps/SDL/build -DCMAKE_BUILD_TYPE=" + type + " -DSDL_TEST_LIBRARY=OFF -DSDL_TESTS=OFF -DSDL_INSTALL_TESTS=OFF"
		}
	}}
	cmake --build deps/SDL/build --config {{ type }}
	{{ mkdir_p }} deps/SDL/build/lib
	{{ 
		if os_family() == "windows" {
			"Copy-Item -Path deps/SDL/build/" + type + "/* -Destination deps/SDL/build/lib -Recurse -Force"
		} else {
			"find deps/SDL/build -maxdepth 1 \\( -name '*.so*' -o -name '*.a' \\) -exec cp -a {} deps/SDL/build/lib/ \\;"
		} 
	}}
	{{ mkdir_p }} {{ dist_dir }}
	{{ 
		if target_goos == "windows" {
			"Copy-Item -Path deps/SDL/build/" + type + "/*.dll -Destination " + dist_dir + " -Force"
		} else {
			"find deps/SDL/build -maxdepth 1 -name '*.so*' -exec cp -a {} " + dist_dir + " \\;"
		} 
	}}

[arg("type", long="type", help="Build type (Debug/Release)")]
build-polyhook2 type="Debug":
	{{
		if os_family() == "windows" {
			"if (!(Test-Path deps/PolyHook2/build)) { cmake -S deps/PolyHook2 -B deps/PolyHook2/build }"
		} else {
			"[ -d deps/PolyHook2/build ] || cmake -S deps/PolyHook2 -B deps/PolyHook2/build -DCMAKE_BUILD_TYPE=" + type
		}
	}}
	cmake --build deps/PolyHook2/build --config {{ type }}
	{{ mkdir_p }} deps/PolyHook2/build/lib
	{{ 
		if os_family() == "windows" {
			"Copy-Item -Path deps/PolyHook2/build/" + type + "/* -Destination deps/PolyHook2/build/lib -Recurse -Force"
		} else {
			"find deps/PolyHook2/build -maxdepth 1 \\( -name '*.so*' -o -name '*.a' \\) -exec cp -a {} deps/PolyHook2/build/lib/ \\;"
		} 
	}}

[working-directory: 'UI']
install-frontend-deps:
	{{
		if os_family() == "windows" {
			"if (!(Test-Path node_modules)) { npm i }"
		} else {
			"[ -d node_modules ] || npm i"
		}
	}}

[working-directory: 'UI']
generate-openapi-frontend-types:
	{{
		if os_family() == "windows" {
			"if (!(Test-Path src/lib/api/openapi.d.ts)) { npm run gen:openapi }"
		} else {
			"[ -f src/lib/api/openapi.d.ts ] || npm run gen:openapi"
		}
	}}

[working-directory: 'UI']
build-frontend: install-frontend-deps generate-openapi-frontend-types
	{{
		if os_family() == "windows" {
			"if (!(Test-Path .env) -and (Test-Path .env.example)) { Copy-Item .env.example .env } ; if (!(Test-Path build)) { npm run build }"
		} else {
			"[ -f .env ] || [ ! -f .env.example ] || cp .env.example .env ; [ -d build ] || npm run build"
		}
	}}

[working-directory: 'cefpayloads']
install-cefpayloads-deps:
	{{
		if os_family() == "windows" {
			"if (!(Test-Path node_modules)) { npm i }"
		} else {
			"[ -d node_modules ] || npm i"
		}
	}}

[working-directory: 'cefpayloads']
generate-openapi-cefpayloads-types:
	{{
		if os_family() == "windows" {
			"if (!(Test-Path src/lib/api/openapi.d.ts)) { npm run gen:openapi }"
		} else {
			"[ -f src/lib/api/openapi.d.ts ] || npm run gen:openapi"
		}
	}}

[working-directory: 'cefpayloads']
build-cefpayloads: install-cefpayloads-deps generate-openapi-cefpayloads-types
	{{
		if os_family() == "windows" {
			"if (!(Test-Path build)) { npm run build }"
		} else {
			"[ -d build ] || npm run build"
		}
	}}

[arg("type", long="type", help="Build type (Debug/Release)")]
build-deps type="Debug": (build-sdl type) (build-frontend) (build-cefpayloads)

build-sisr type="Debug": (build-deps type)
    {{ 
        if target_goos == "windows" {
            "just win-resource"
        } else { 
            "echo Skipping win-resource for non-windows target" 
        } 
    }}
    {{ mkdir_p }} {{ dist_dir }}
    go build -ldflags "{{ if type == "Release" { ldflags_release } else { ldflags_common } }}" -o {{ build_path }} {{ main_pkg }}

[arg("type", pattern="Debug|Release")]
build type="Debug": (build-sisr type)

clean-sdl:
	{{
		if os_family() == "windows" {
			"if (Test-Path deps/SDL/build) { Remove-Item -Recurse -Force deps/SDL/build }"
		} else {
			"[ ! -d deps/SDL/build ] || rm -rf deps/SDL/build"
		}
	}}

clean-polyhook2:
	{{
		if os_family() == "windows" {
			"if (Test-Path deps/PolyHook2/build) { Remove-Item -Recurse -Force deps/PolyHook2/build }"
		} else {
			"[ ! -d deps/PolyHook2/build ] || rm -rf deps/PolyHook2/build"
		}
	}}

clean-frontend:
	{{
		if os_family() == "windows" {
			"if (Test-Path UI/build) { Remove-Item -Recurse -Force UI/build }"
		} else {
			"[ ! -d UI/build ] || rm -rf UI/build"
		}
	}}

clean-cefpayloads:
	{{
		if os_family() == "windows" {
			"if (Test-Path cefpayloads/dist) { Remove-Item -Recurse -Force cefpayloads/dist }"
		} else {
			"[ ! -d cefpayloads/dist ] || rm -rf cefpayloads/dist"
		}
	}}

clean-deps: clean-sdl clean-polyhook2 clean-frontend clean-cefpayloads

clean: clean-deps
    -{{ rm_rf }} {{ dist_dir }}
    go clean

test:
    go test -v ./...

fmt:
    go fmt ./...

lint:
    golangci-lint run ./...
