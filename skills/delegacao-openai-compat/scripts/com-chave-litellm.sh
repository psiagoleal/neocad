#!/usr/bin/env bash
# Caminho relativo: skills/delegacao-openai-compat/scripts/com-chave-litellm.sh
#
# Resolve a chave de API do gateway LiteLLM a partir de fontes configuráveis e
# executa o comando dado com AGENTRY_LITELLM_API_KEY exportada APENAS para o
# processo filho. A chave nunca é impressa, nunca vira argumento de linha de
# comando e nunca é gravada em disco por este script (skill `secrets-guard`).
#
#   bash com-chave-litellm.sh --check
#   bash com-chave-litellm.sh agentry --provider litellm "revise o diff ..."
#
# Fontes, em ordem de precedência (a primeira que resolver vence; nunca há merge):
#
#   1. AGENTRY_LITELLM_API_KEY  — já no ambiente
#   2. AGENTRY_LITELLM_KEY_CMD  — comando que imprime a chave em stdout (cofre)
#   3. AGENTRY_LITELLM_KEY_FILE — arquivo; com AGENTRY_LITELLM_KEY_JQ, extrai a
#                                 expressão jq dele (JSON com chaves de vários
#                                 serviços); sem ela, usa a primeira linha
#   4. ~/.agentry/credentials.json — não é lido aqui: o próprio agentry resolve
#
# Saída de erro: sempre em stderr, sempre sem o valor da chave.

set -euo pipefail

readonly VAR_CHAVE='AGENTRY_LITELLM_API_KEY'

# Rastreamento de shell exibiria a chave em cada expansão. Recusa fail-closed.
case "$-" in
  *x*)
    printf 'com-chave-litellm: recusado — xtrace (set -x) está ativo e vazaria a chave.\n' >&2
    exit 4
    ;;
esac

erro() {
  printf 'com-chave-litellm: %s\n' "$1" >&2
  exit "${2:-3}"
}

uso() {
  sed -n '3,25p' "$0" | sed 's/^# \{0,1\}//'
  exit 0
}

# Valida sem nunca ecoar o valor: não-vazia, uma linha só, sem placeholder do
# framework (ADR-0004 §3 — jamais disparar rede com `{{CHAVE}}` por engano).
validar_chave() {
  local chave="$1" fonte="$2"
  [ -n "$chave" ] || erro "a fonte '$fonte' devolveu uma chave vazia."
  case "$chave" in
    *'{{'*) erro "a fonte '$fonte' devolveu um placeholder não substituído, não uma chave." ;;
    *[[:space:]]*) erro "a fonte '$fonte' devolveu uma chave com espaço/quebra de linha — verifique a extração." ;;
  esac
}

avisar_permissao_aberta() {
  local arquivo="$1" modo
  modo="$(stat -c '%a' "$arquivo" 2>/dev/null || echo '')"
  case "$modo" in
    ''|*00) : ;;
    *) printf 'com-chave-litellm: aviso — %s está com permissão %s; recomendado 600.\n' "$arquivo" "$modo" >&2 ;;
  esac
}

# Recusa disparar rede com `providers.litellm` ainda no template: baseUrl/model
# com `<...>` ou `{{...}}` produziriam um erro de conexão obscuro, e a ADR-0004
# §3 manda validar o placeholder ANTES de qualquer chamada.
verificar_placeholder_config() {
  local cfg='.agentry/agentry.settings.json'
  [ -r "$cfg" ] || return 0
  command -v jq >/dev/null 2>&1 || return 0
  if jq -e '[(.providers.litellm.baseUrl // ""), (.providers.litellm.model // "")]
            | map(test("^<|[{][{]")) | any' "$cfg" >/dev/null 2>&1; then
    erro "providers.litellm ainda está com placeholder em $cfg — substitua baseUrl e model pelos valores reais do gateway antes de delegar."
  fi
}

resolver_chave() {
  local chave=''

  if [ -n "${AGENTRY_LITELLM_API_KEY:-}" ]; then
    FONTE='ambiente'
    validar_chave "$AGENTRY_LITELLM_API_KEY" "$FONTE"
    CHAVE="$AGENTRY_LITELLM_API_KEY"
    return 0
  fi

  if [ -n "${AGENTRY_LITELLM_KEY_CMD:-}" ]; then
    FONTE='comando externo (AGENTRY_LITELLM_KEY_CMD)'
    if ! chave="$(sh -c "$AGENTRY_LITELLM_KEY_CMD" 2>/dev/null)"; then
      erro "o comando de AGENTRY_LITELLM_KEY_CMD falhou (cofre travado? sessão expirada?)."
    fi
    chave="${chave%%$'\n'*}"
    validar_chave "$chave" "$FONTE"
    CHAVE="$chave"
    return 0
  fi

  if [ -n "${AGENTRY_LITELLM_KEY_FILE:-}" ]; then
    local arquivo="${AGENTRY_LITELLM_KEY_FILE/#\~/$HOME}"
    [ -r "$arquivo" ] || erro "AGENTRY_LITELLM_KEY_FILE aponta para '$arquivo', que não existe ou não é legível."
    avisar_permissao_aberta "$arquivo"
    if [ -n "${AGENTRY_LITELLM_KEY_JQ:-}" ]; then
      command -v jq >/dev/null 2>&1 || erro "AGENTRY_LITELLM_KEY_JQ exige o 'jq', ausente no PATH."
      FONTE="arquivo JSON + expressão jq"
      # `jq -e` sai com 1 quando o resultado é null/false, mas AINDA imprime
      # "null" em stdout — sem checar o código de saída, "null" viraria a chave.
      if ! chave="$(jq -er "$AGENTRY_LITELLM_KEY_JQ" "$arquivo" 2>/dev/null)"; then
        erro "a expressão jq de AGENTRY_LITELLM_KEY_JQ não encontrou uma chave em '$arquivo'."
      fi
    else
      FONTE='arquivo de uma linha'
      chave="$(head -n 1 "$arquivo" 2>/dev/null || true)"
    fi
    chave="${chave%%$'\n'*}"
    [ "$chave" != 'null' ] || erro "a fonte '$FONTE' devolveu o literal 'null'."
    validar_chave "$chave" "$FONTE"
    CHAVE="$chave"
    return 0
  fi

  # Última fonte: o próprio agentry lê ~/.agentry/credentials.json. Este script
  # não abre o arquivo — só confirma que há entrada para o provider, sem tocar
  # no valor, e delega a leitura a quem já faz isso sob a disciplina da ADR-0038.
  local credenciais="$HOME/.agentry/credentials.json"
  if [ -r "$credenciais" ] && command -v jq >/dev/null 2>&1 \
     && jq -e '(.providers.litellm.apiKey // "") | length > 0' "$credenciais" >/dev/null 2>&1; then
    FONTE='~/.agentry/credentials.json (resolvida pelo próprio agentry)'
    CHAVE=''
    return 0
  fi

  erro "nenhuma fonte de chave configurada.
  Escolha uma:
    agentry --set-credential litellm                       # grava em ~/.agentry/credentials.json (lê de stdin)
    export AGENTRY_LITELLM_KEY_CMD='op read op://cofre/litellm/key'
    export AGENTRY_LITELLM_KEY_FILE=~/chaves.json AGENTRY_LITELLM_KEY_JQ='.litellm.api_key'
  Gateway sem autenticação: exporte AGENTRY_LITELLM_SEM_CHAVE=1 para pular esta checagem." 3
}

main() {
  case "${1:-}" in
    ''|-h|--help|--ajuda) uso ;;
  esac

  verificar_placeholder_config

  if [ "${AGENTRY_LITELLM_SEM_CHAVE:-}" = '1' ]; then
    FONTE='gateway declarado sem autenticação'
    CHAVE=''
  else
    FONTE=''
    CHAVE=''
    resolver_chave
  fi

  if [ "$1" = '--check' ]; then
    if [ -n "$CHAVE" ] || [ "${AGENTRY_LITELLM_SEM_CHAVE:-}" != '1' ]; then
      printf 'com-chave-litellm: ok — chave disponível via %s.\n' "$FONTE"
    else
      printf 'com-chave-litellm: ok — %s; nenhum cabeçalho de autorização será anexado.\n' "$FONTE"
    fi
    exit 0
  fi

  if [ -n "$CHAVE" ]; then
    export "$VAR_CHAVE=$CHAVE"
  fi
  unset CHAVE
  exec "$@"
}

main "$@"
