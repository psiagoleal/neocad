---
name: secrets-guard
description: >-
  Aplica os quatro princípios de não-exposição de segredos antes de executar
  qualquer comando de shell, ler arquivos ou produzir saída. Aciona quando a
  tarefa envolver credenciais, variáveis de ambiente, arquivos .env, cofres,
  chaves de API, tokens, certificados, ou inspeção de configuração de
  ferramentas (aws/gcloud/az/kubectl/docker/git config).
---

# secrets-guard — Guardião de segredos

## A lei

> **Segredo não passa pela janela do agente. Nunca.**
> Na dúvida sobre a saída de um comando, não rode. Pergunte.

Sem exceção por urgência, por ser "só para conferir", por ser ambiente de
desenvolvimento, por o valor "já estar exposto mesmo" ou por o usuário ter pedido.
Quando a inspeção direta for realmente inevitável, ela é feita **pelo humano, fora do
agente** — não por você com mais cuidado.

## Por que a lei é absoluta

Um segredo impresso no histórico não volta atrás. A partir daquele turno ele é
reenviado ao provedor do modelo a cada requisição, pode persistir em cache de prompt,
aparecer em *trace* de observabilidade e ser copiado sem querer para *issue*, descrição
de PR ou mensagem de *commit*. O comando leva um segundo; a exposição é permanente e
custa rotação de credencial.

É por isso que a regra é uma lei e não uma recomendação: o dano é assimétrico. O custo
de recusar um comando é um turno. O custo de rodá-lo errado uma vez é um incidente.

## Os quatro princípios

1. **Nunca execute** comando cujo objetivo seja, direta ou indiretamente, exibir
   conteúdo de `.env`, cofre desbloqueado, *keyring* do SO, variável de ambiente
   sensível, token JWT válido ou trecho de log que possa conter chave, certificado ou
   *fingerprint*.
2. **Avalie antes de invocar.** A pergunta é sempre "esta saída pode conter material
   sensível?". Dúvida é resposta suficiente: abstenha-se e relate ao operador.
3. **Prefira verificação indireta** para validar credencial: autenticar contra
   `/healthz` ou `/me`; conferir SHA-256 truncado; mostrar só os quatro últimos
   caracteres mascarados (`****abcd`); executar a chamada-fim e relatar apenas
   sucesso ou falha.
4. **Delegue ao humano** a inspeção inevitável. Ele roda localmente, fora da janela, e
   devolve só o resultado funcional saneado.

## Comandos proibidos (lista não exaustiva)

Não invoque. Se pedirem, recuse e ofereça a verificação indireta equivalente:

```
cat .env            *.env       .env.*          # leitura de arquivos de segredo
env | grep -i (token|key|secret|pass)
printenv            export -p
git config --get-all   git config --list
kubectl get secret ... -o yaml|-o json
aws configure list     aws sts get-...           # quando ecoa credenciais
gcloud config list     gcloud auth print-access-token
az account show        az account get-access-token
docker compose config  docker inspect ...        # quando expõe env de containers
Get-Content secrets.json   cat ~/.aws/credentials   cat ~/.ssh/id_*
```

A lista é ilustrativa, não um filtro. Comando fora dela que possa imprimir segredo cai
na mesma regra — o critério é a saída possível, não o nome do binário.

## O que você vai pensar para burlar a regra

Toda vez que a lei custar alguma coisa, aparece uma frase que a dispensa. Elas são
previsíveis, e reconhecê-las é a defesa:

| O que você vai pensar | Por que não vale |
|---|---|
| "É só para eu conferir se está setada" | `test -n` confere sem imprimir. Sempre houve alternativa. |
| "É ambiente de desenvolvimento" | Credencial de dev abre serviço de dev. O histórico é o mesmo. |
| "O usuário pediu explicitamente" | Ele pediu o resultado, não a exposição. Entregue o resultado. |
| "Já está no `.env` do repo mesmo" | Estar em disco é um risco; estar no histórico do modelo é outro. |
| "Vou só mascarar na resposta" | O comando já rodou. A saída já entrou no contexto antes de você editar. |
| "É mais rápido que explicar" | O incidente não é mais rápido. |
| "Só desta vez, é um caso diferente" | É sempre um caso diferente. É sempre a mesma regra. |

**Pare se pensar qualquer uma delas.** A frase não é um argumento — é o sinal de que a
lei está prestes a ser quebrada.

## Verificações indiretas (faça assim)

```bash
# Em vez de imprimir a chave, confirme só que está presente e o tamanho:
test -n "${API_KEY:-}" && printf 'API_KEY presente (%d chars)\n' "${#API_KEY}"

# Validar credencial pela chamada-fim, sem exibi-la:
curl -fsS -o /dev/null -w '%{http_code}\n' -H "Authorization: Bearer $API_KEY" https://api.exemplo/healthz

# Conferir digest sem revelar o segredo:
printf '%s' "$API_KEY" | sha256sum | cut -c1-12
```

## Armazenamento de segredos (por perfil)

- **empresa:** HashiCorp Vault / OpenBao on-premise, ou AWS Secrets Manager / Azure Key
  Vault / Google Secret Manager, com auditoria de leitura e rotação automática.
- **externo-confidencial:** Doppler, Infisical ou 1Password Secrets Automation; ativar
  opt-out de retenção de dados no provedor de modelo.
- **pessoal:** cofre leve aceitável; `.env` local **apenas** se constar no `.gitignore`
  global da máquina e nos arquivos de exclusão do agente (`.claudeignore` etc.).

Injete credenciais **somente em tempo de execução**, via variáveis de ambiente
carregadas do cofre — nunca materializadas em `.env` versionável.

Segredo é a exceção declarada à regra repo-local (ADR 0011): ele mora **fora** da
árvore versionada, porque segredo guardado dentro do repositório acaba commitado.

## Rede de segurança

A lei é probabilística enquanto depender de você lembrar dela. Torne-a estrutural:
varredura pré-commit (`gitleaks`, `trufflehog`, `detect-secrets`) e *hook*
`PreToolUse` que bloqueie os padrões acima **antes** de o comando chegar a rodar.
Quando o agente errar mesmo assim, a correção é de autoridade, não de texto — ver
`atribuicao-de-falha`.

## Definição de pronto da skill

- [ ] Nenhum comando da lista proibida foi executado na sessão.
- [ ] Nenhum comando fora da lista imprimiu material sensível.
- [ ] Toda validação de credencial usou verificação indireta.
- [ ] Inspeção inevitável foi delegada ao humano, fora da janela do agente.
- [ ] Nenhuma racionalização da tabela acima foi aceita como exceção.
