#!/usr/bin/env bash
# Caminho relativo: scripts/release.sh
#
# Automação de release do NeoCAD. Centraliza tag git, build portátil Windows x64
# (Fixed WebView2 Runtime) com a versão no nome do artefato, e publicação de
# release no GitHub via `gh`.
#
# A versão é sempre derivada de package.json (fonte única), garantindo que tag,
# nome do .zip e título do release fiquem alinhados.
#
# Uso:
#   scripts/release.sh version   # imprime a versão atual (package.json)
#   scripts/release.sh package   # copia o .zip buildado para *_v<versão>.zip
#   scripts/release.sh build     # build portátil fixed-runtime + package
#   scripts/release.sh tag       # cria tag anotada v<versão> e faz push para origin
#   scripts/release.sh publish   # cria/atualiza release no GitHub com o asset versionado
#   scripts/release.sh all       # tag -> build -> publish
#
# Variáveis de ambiente:
#   NEOCAD_RELEASE_DRAFT=1  publica como rascunho (padrão).
#   NEOCAD_RELEASE_DRAFT=0  publica como release "latest".

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

PORTABLE_DIR="build/windows/portable"
SOURCE_ZIP="${PORTABLE_DIR}/NeoCAD-portable-x64.zip"

version() {
	node -p "require('./package.json').version"
}

versioned_zip() {
	echo "${PORTABLE_DIR}/NeoCAD-portable-x64_v$(version).zip"
}

# Extrai a seção "## [<versão>]" do CHANGELOG.md para usar como notas do release.
release_notes() {
	local ver="$1"
	awk -v ver="$ver" '
		$0 ~ ("^## \\[" ver "\\]") { grab = 1; next }
		grab && /^## \[/ { exit }
		grab { print }
	' CHANGELOG.md
}

cmd_version() {
	version
}

cmd_package() {
	local dst
	dst="$(versioned_zip)"
	if [[ ! -f "$SOURCE_ZIP" ]]; then
		echo "erro: artefato não encontrado: $SOURCE_ZIP (rode 'build' antes)" >&2
		exit 1
	fi
	cp -f "$SOURCE_ZIP" "$dst"
	echo "$dst"
}

cmd_build() {
	make cmake-windows-x64-portable-fixed-runtime
	cmd_package
}

cmd_tag() {
	local ver tag
	ver="$(version)"
	tag="v${ver}"
	if git rev-parse -q --verify "refs/tags/${tag}" >/dev/null; then
		echo "tag ${tag} já existe localmente; pulando criação." >&2
	else
		git tag -a "${tag}" -m "NeoCAD ${tag}"
		echo "tag ${tag} criada."
	fi
	git push origin "${tag}"
	echo "tag ${tag} enviada para origin."
}

cmd_publish() {
	local ver tag zip notes_file
	ver="$(version)"
	tag="v${ver}"
	zip="$(versioned_zip)"
	if [[ ! -f "$zip" ]]; then
		echo "erro: asset não encontrado: $zip (rode 'build' antes)" >&2
		exit 1
	fi

	if gh release view "${tag}" >/dev/null 2>&1; then
		gh release upload "${tag}" "${zip}" --clobber
		echo "asset enviado/atualizado no release ${tag}."
		return 0
	fi

	notes_file="$(mktemp)"
	if release_notes "$ver" | grep -q '[^[:space:]]'; then
		release_notes "$ver" >"$notes_file"
	else
		echo "Build portátil Windows x64 com Fixed WebView2 Runtime." >"$notes_file"
	fi

	local mode_flag="--draft"
	if [[ "${NEOCAD_RELEASE_DRAFT:-1}" == "0" ]]; then
		mode_flag="--latest"
	fi
	gh release create "${tag}" "${zip}" \
		--title "NeoCAD ${tag}" \
		--notes-file "${notes_file}" \
		"${mode_flag}"
	rm -f "$notes_file"
	echo "release ${tag} criado (${mode_flag})."
}

cmd_all() {
	cmd_tag
	cmd_build
	cmd_publish
}

main() {
	local cmd="${1:-all}"
	case "$cmd" in
		version) cmd_version ;;
		package) cmd_package ;;
		build) cmd_build ;;
		tag) cmd_tag ;;
		publish) cmd_publish ;;
		all) cmd_all ;;
		*)
			echo "uso: $0 {version|package|build|tag|publish|all}" >&2
			exit 2
			;;
	esac
}

main "$@"
