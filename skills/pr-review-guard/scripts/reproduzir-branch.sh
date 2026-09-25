#!/usr/bin/env bash
# Caminho relativo: skills/pr-review-guard/scripts/reproduzir-branch.sh
#
# Roda um comando contra o código de OUTRO ramo, sem tocar no checkout de quem revisa.
#
#   reproduzir-branch.sh <ref> [-C <repo>] [--] <comando...>
#
#   reproduzir-branch.sh origin/feature/x -- .venv/bin/python -m pytest tests -q
#   reproduzir-branch.sh origin/main -C ~/dev/api -- cargo test
#
# Por que existe: revisão por leitura tem teto, e comportamento de biblioteca fica abaixo
# dele. Enquanto reproduzir custa caro, o revisor deduz — e deduzir é onde nascem os
# achados falsos. Isto derruba o custo para um comando.
#
# `git archive` é somente leitura: não faz checkout, não mexe no índice, não exige árvore
# limpa e não cria branch. O ramo em revisão é extraído para um diretório temporário,
# apagado ao final.
#
# O venv/toolchain do repositório é reaproveitado por caminho absoluto — o temporário tem o
# CÓDIGO do ramo, não as dependências.
#
# Códigos: o do comando · 2 uso ou ref inexistente.
set -euo pipefail

repo="."
ref=""
cmd=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    -C) repo="${2:?-C exige um diretório}"; shift 2 ;;
    --) shift; cmd=("$@"); break ;;
    -h|--help) sed -n '3,20p' "$0" | sed 's/^# \?//'; exit 0 ;;
    -*) printf 'opção desconhecida: %s\n' "$1" >&2; exit 2 ;;
    *)  if [[ -z "$ref" ]]; then ref="$1"; shift; else cmd=("$@"); break; fi ;;
  esac
done

[[ -n "$ref" && ${#cmd[@]} -gt 0 ]] || { sed -n '3,20p' "$0" | sed 's/^# \?//' >&2; exit 2; }
repo="$(cd "$repo" && pwd)"
git -C "$repo" rev-parse --git-dir >/dev/null 2>&1 || { printf 'não é repositório git: %s\n' "$repo" >&2; exit 2; }
git -C "$repo" rev-parse --verify --quiet "$ref^{commit}" >/dev/null || {
  printf 'ref inexistente: %s (faltou `git fetch`?)\n' "$ref" >&2; exit 2; }

sha="$(git -C "$repo" rev-parse --short "$ref^{commit}")"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

git -C "$repo" archive "$ref" | tar -x -C "$tmp"
printf '\033[36m›\033[0m %s (%s) extraído em %s\n' "$ref" "$sha" "$tmp"
printf '\033[36m›\033[0m %s\n\n' "${cmd[*]}"

# O comando roda com o temporário como raiz; caminhos absolutos do repositório real
# (venv, toolchain, dados) continuam válidos.
cd "$tmp"
set +e
"${cmd[@]}"
codigo=$?
set -e

printf '\n\033[36m›\033[0m saída %d — código de %s (%s), checkout de %s intacto\n' \
  "$codigo" "$ref" "$sha" "$repo"
exit "$codigo"
