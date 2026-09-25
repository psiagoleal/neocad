#!/usr/bin/env bash
# Caminho relativo: skills/pr-review-guard/scripts/checar-mensagem-commit.sh
#
# Hook commit-msg: recusa a mensagem de commit que traga qualquer entrada de agente além do
# marcador entre chaves ({agente: ...; modelo: ...}) — link ou ID de sessão, URL de conversa,
# rodapé "Generated with…", trailers Co-authored-by/Assisted-by/Generated-by/*-Session.
#
# Existe porque a regra em texto (seção de proveniência do AGENTS.md) é probabilística: a
# ferramenta pode acrescentar essas linhas por padrão, e o agente segue a ferramenta.
#
# Instalação no clone (local, não versionada):
#   ln -sf ../../skills/pr-review-guard/scripts/checar-mensagem-commit.sh .git/hooks/commit-msg
# Uso avulso: skills/pr-review-guard/scripts/checar-mensagem-commit.sh <arquivo-com-a-mensagem>
set -u
msg="${1:?uso: checar-mensagem-commit.sh <arquivo-da-mensagem>}"
# ignora comentários do editor (linhas iniciadas por #)
corpo="$(grep -v '^#' "$msg")"
padrao='(claude\.ai/code|chatgpt\.com/(c|share)/|/session[_-][A-Za-z0-9]+|[A-Za-z]+-Session:|^(Co-authored-by|Assisted-by|Generated-by):|Generated with \[?(Claude|Codex|Copilot|Gemini|Cursor))'
achados="$(printf '%s\n' "$corpo" | grep -niE "$padrao" || true)"
if [[ -n "$achados" ]]; then
  printf '\033[31mcommit-msg:\033[0m a mensagem tem entrada de agente além do marcador entre chaves:\n' >&2
  printf '%s\n' "$achados" | sed 's/^/  linha /' >&2
  printf '  Único registro permitido: {agente: <nome>; modelo: <modelo/versão>} ao final.\n' >&2
  exit 1
fi
exit 0
