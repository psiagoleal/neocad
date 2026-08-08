#!/usr/bin/env bash
# Caminho relativo: scripts/build-test.sh
#
# Gera builds locais de TESTE do NeoCAD para Linux e Windows, sempre os dois, e
# os organiza em `dist-test/`.
#
# Não substitui `scripts/release.sh`: o release oficial de Windows usa a toolchain
# MSVC e passa pelo empacotamento portátil/NSIS. Aqui o Windows sai por
# cross-build MinGW, que não exige `cargo-xwin`, `llvm-rc` nem privilégios de
# administrador — barato para validar, impróprio para distribuir.
#
# Uso:
#   scripts/build-test.sh          # ambos (padrão)
#   scripts/build-test.sh linux
#   scripts/build-test.sh windows
#   scripts/build-test.sh deps     # apenas diagnostica pré-requisitos

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

DIST_DIR="dist-test"
WINDOWS_TARGET="x86_64-pc-windows-gnu"

LINUX_DEPS=(glib-2.0 gtk+-3.0 webkit2gtk-4.1 libsoup-3.0 javascriptcoregtk-4.1)
LINUX_APT_PACKAGES=(
	libwebkit2gtk-4.1-dev
	libjavascriptcoregtk-4.1-dev
	libsoup-3.0-dev
	libayatana-appindicator3-dev
	librsvg2-dev
	libxdo-dev
	build-essential
	curl
	wget
	file
	libssl-dev
	patchelf
)

version() {
	node -p "require('./package.json').version"
}

log() {
	printf '\n\033[1m==> %s\033[0m\n' "$1"
}

warn() {
	printf '\033[33maviso:\033[0m %s\n' "$1" >&2
}

missing_linux_deps() {
	local missing=()
	local dep
	for dep in "${LINUX_DEPS[@]}"; do
		pkg-config --exists "$dep" 2>/dev/null || missing+=("$dep")
	done
	printf '%s\n' "${missing[@]:-}"
}

check_linux_deps() {
	local missing
	mapfile -t missing < <(missing_linux_deps)

	if [[ ${#missing[@]} -gt 0 && -n "${missing[0]}" ]]; then
		cat >&2 <<-EOF

			erro: faltam bibliotecas de sistema para compilar o shell Tauri em Linux:
			  ${missing[*]}

			Instale com:
			  sudo apt-get update && sudo apt-get install -y ${LINUX_APT_PACKAGES[*]}
		EOF
		return 1
	fi
}

check_windows_deps() {
	local missing=()
	command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1 || missing+=("x86_64-w64-mingw32-gcc (pacote gcc-mingw-w64-x86-64)")
	rustup target list --installed 2>/dev/null | grep -qx "$WINDOWS_TARGET" ||
		missing+=("alvo rustup $WINDOWS_TARGET (rustup target add $WINDOWS_TARGET)")

	if [[ ${#missing[@]} -gt 0 ]]; then
		printf '\nerro: faltam pré-requisitos para o cross-build Windows:\n' >&2
		printf '  - %s\n' "${missing[@]}" >&2
		return 1
	fi
}

cmd_deps() {
	local missing status=0
	mapfile -t missing < <(missing_linux_deps)

	log "Pré-requisitos Linux"
	if [[ ${#missing[@]} -gt 0 && -n "${missing[0]}" ]]; then
		printf '  faltam: %s\n' "${missing[*]}"
		printf '  instale: sudo apt-get install -y %s\n' "${LINUX_APT_PACKAGES[*]}"
		status=1
	else
		printf '  tudo presente\n'
	fi

	log "Pré-requisitos Windows (cross-build MinGW)"
	if check_windows_deps 2>/dev/null; then
		printf '  tudo presente\n'
	else
		check_windows_deps || status=1
	fi

	return $status
}

stage_linux() {
	local out="${DIST_DIR}/NeoCAD-test-linux-x64"
	local binary="src-tauri/target/release/neocad"

	[[ -f "$binary" ]] || {
		echo "erro: binário não encontrado: $binary" >&2
		return 1
	}

	rm -rf "$out"
	mkdir -p "$out"
	cp "$binary" "$out/"
	chmod +x "$out/neocad"

	# Empacota também o .deb quando o bundler o tiver produzido. Bloco explícito
	# em vez de `[[ … ]] && cp`, que sob `set -e` propagaria status 1 quando o
	# .deb não existe.
	local deb
	deb="$(find src-tauri/target/release/bundle/deb -name '*.deb' -print -quit 2>/dev/null || true)"
	if [[ -n "$deb" ]]; then
		cp "$deb" "$out/"
	fi

	write_readme "$out" "Linux x64"

	printf '%s\n' "$out"
}

stage_windows() {
	local out="${DIST_DIR}/NeoCAD-test-windows-x64"
	local release="src-tauri/target/${WINDOWS_TARGET}/release"

	[[ -f "${release}/neocad.exe" ]] || {
		echo "erro: binário não encontrado: ${release}/neocad.exe" >&2
		return 1
	}

	rm -rf "$out"
	mkdir -p "$out"
	cp "${release}/neocad.exe" "$out/"

	# O executável importa WebView2Loader.dll dinamicamente; ele precisa viajar junto.
	if [[ -f "${release}/WebView2Loader.dll" ]]; then
		cp "${release}/WebView2Loader.dll" "$out/"
	else
		warn "WebView2Loader.dll não encontrado ao lado do .exe; o app não abrirá no Windows."
	fi

	write_readme "$out" "Windows x64"

	printf '%s\n' "$out"
}

# Cada pacote descreve a si mesmo: como executar, o que dá para testar, o que
# ainda não existe e sob que licença o binário é distribuído.
write_readme() {
	local dir="$1" platform="$2"

	{
		echo "NeoCAD $(version) — build de TESTE para ${platform}"
		echo "=================================================="
		echo
		echo "Como executar"
		echo "-------------"
		if [[ "$platform" == Linux* ]]; then
			cat <<-'EOF'
				  ./neocad

				O .deb ao lado, se presente, instala o app no sistema:
				  sudo apt install ./neocad_*.deb
			EOF
		else
			cat <<-'EOF'
				1. Copie esta pasta inteira para o Windows. Os dois arquivos ficam juntos.
				2. Execute neocad.exe.

				Requer o Microsoft Edge WebView2 Runtime, que já vem no Windows 11.
				O binário não é assinado: o SmartScreen pedirá "Executar assim mesmo".
			EOF
		fi
		cat <<-'EOF'

			O que dá para testar
			--------------------
			- abrir desenhos DWG e DXF por diálogo nativo (Arquivo > Abrir)
			- arrastar e soltar arquivo CAD sobre a área do viewer
			- ajustar a vista ao desenho e alternar o fundo do canvas
			- lista de desenhos recentes, persistida entre execuções
			- barra de comandos do viewer dentro do canvas
			- catálogo de comandos em Ajuda > Comandos CAD
			- dock de mensagens e progresso de abertura

			O que ainda NÃO existe
			----------------------
			- desfazer/refazer (fase K1 do kernel, em andamento)
			- salvar ou exportar arquivo
			- painéis de camadas e propriedades
			- edição consistente de entidades
		EOF
		if [[ "$platform" == Windows* ]]; then
			cat <<-'EOF'

				Aviso sobre este binário
				------------------------
				Compilado com toolchain MinGW (x86_64-pc-windows-gnu), e não com o MSVC
				que o release oficial usa. Serve para teste, não para distribuição.
			EOF
		fi
		cat <<-'EOF'

			Licença
			-------
			GPL-3.0-or-later. Este binário embute bibliotecas GPL-3.0 (LibreDWG para DWG
			e dxf-json para DXF). Ao redistribuir, a GPL-3.0 exige a oferta do
			código-fonte correspondente. Ver LICENSE e THIRD-PARTY-LICENSES.md.
		EOF
	} >"${dir}/LEIA-ME.txt"
}

archive() {
	local dir="$1"
	[[ -d "$dir" ]] || return 0
	command -v zip >/dev/null 2>&1 || {
		warn "zip ausente; pacote entregue apenas como diretório."
		return 0
	}
	(cd "$DIST_DIR" && zip -qr "$(basename "$dir").zip" "$(basename "$dir")")
}

cmd_linux() {
	check_linux_deps
	log "Build Linux x64 (nativo)"
	pnpm tauri build --bundles deb
	local out
	out="$(stage_linux)"
	archive "$out"
	log "Linux pronto: ${out}"
}

cmd_windows() {
	check_windows_deps
	log "Build Windows x64 (cross-build MinGW — teste, não distribuição)"
	pnpm tauri build --target "$WINDOWS_TARGET" --no-bundle
	local out
	out="$(stage_windows)"
	archive "$out"
	log "Windows pronto: ${out}"
}

cmd_all() {
	mkdir -p "$DIST_DIR"
	local failed=()

	cmd_linux || failed+=("linux")
	cmd_windows || failed+=("windows")

	log "Resumo — NeoCAD $(version)"
	ls -1sh "$DIST_DIR" 2>/dev/null || true

	if [[ ${#failed[@]} -gt 0 ]]; then
		printf '\nerro: build(s) com falha: %s\n' "${failed[*]}" >&2
		return 1
	fi
}

main() {
	mkdir -p "$DIST_DIR"
	case "${1:-all}" in
		linux) cmd_linux ;;
		windows) cmd_windows ;;
		deps) cmd_deps ;;
		all) cmd_all ;;
		*)
			echo "uso: $0 {all|linux|windows|deps}" >&2
			exit 2
			;;
	esac
}

main "$@"
