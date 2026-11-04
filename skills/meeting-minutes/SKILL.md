---
name: meeting-minutes
description: >-
  Produz ATAs e resumos de reunião/entrevista a partir de gravações ou
  transcrições, com abordagem sensível-à-confidencialidade e on-premise-first:
  transcreve localmente quando há áudio (Whisper), divide o trabalho de forma híbrida
  entre modelo local (síntese qualitativa/sentimento) e modelo mais capaz
  (campos estruturados) sob autorização quando há saída do perímetro, aplica
  antialucinação e registra proveniência. Aciona ao pedir ATA, minuta, resumo
  ou notas de reunião, transcrição de entrevista, ou ao processar gravações
  (.mp4/.wav/.docx) e transcrições automáticas.
---

# meeting-minutes — ATAs e resumos de reunião

Reuniões e entrevistas concentram **decisões, compromissos e sentimento** — informação de
alto valor que se perde se não virar um registro durável e compartilhável. Ao mesmo tempo,
**áudio e transcrições costumam ser o artefato mais sensível** de um projeto: contêm nomes,
opiniões atribuíveis a pessoas e dados de terceiros. Registrar bem a reunião **e** respeitar
o perímetro do dado são o **mesmo** trabalho — e é por isso que esta skill trata os dois
juntos. Um agente compreende melhor uma restrição quando entende o *porquê*: aqui, o porquê é
que resumir um áudio confidencial num serviço externo pode **exfiltrar** o material mais
sensível do projeto sem que ninguém tenha decidido isso.

## Princípio central (restrição cognitiva)

**Sensível-primeiro, on-premise-first.** Antes de processar qualquer gravação/transcrição:

1. **Classifique a sensibilidade** do material (há dado de terceiros, nomes, opiniões
   atribuíveis, dado operacional?).
2. **Consulte a governança de dados** do projeto (ADRs/política de acesso). Se a tarefa
   conflita com uma decisão `Accepted`, **pare e reporte** — não contorne. (ver
   [`secrets-guard`](../secrets-guard/SKILL.md) e [`adr-writer`](../adr-writer/SKILL.md))
3. Se **confidencial**: **processe onde o dado vive** (modelos locais) e **minimize** o que
   sai do perímetro.

## Fluxo

0. **Consentimento e retenção.** Confirme que houve consentimento para gravar/anotar e a
   política de retenção do material bruto. Registre isso na ATA.
1. **Sensibilidade & governança** (princípio acima).
2. **Transcrição.** Se há áudio/vídeo, avalie a transcrição automática existente antes de
   confiar nela — transcrições automáticas de ferramentas de reunião normalmente **não são
   boas o suficiente**: erram termos técnicos e **atribuição de orador**. Quando a qualidade
   importa, gere **ASR local com Whisper** (ex.: `faster-whisper`) para aprimorar/substituir
   a transcrição automática; se o material é confidencial, ASR local é **obrigatório** (nada
   de enviar o áudio a serviço externo). Veja `references/pipeline-recipes.md`.
3. **Divisão de trabalho (híbrido).** Separe por afinidade de tarefa:
   - **Síntese qualitativa** (resumo, decisões, sentimento, "quem disse o quê") → funciona
     bem em **modelo local**; se confidencial, mantenha-a on-premise (não entra no contexto
     de um agente externo).
   - **Campos estruturados/precisos** (datas, listas, marcações de formulário) → um modelo
     mais capaz ajuda. Se isso implicar **saída do perímetro**, faça-o apenas com
     **autorização explícita e registrada** de um decisor de dados e no **escopo mínimo**.
4. **Dois artefatos por reunião.** Se o usuário forneceu um template/modelo de ATA próprio,
   **use o dele**; apenas na ausência de um, use o padrão `templates/ata-template.md`:
   - **ATA/estrutura:** participantes, pauta, discussão, **decisões**, **ações
     (o quê / responsável / prazo)**, pendências, próxima reunião.
   - **Observações & sentimento:** **sistemas/ferramentas de dados citados**, **impressões
     por participante**, clima geral e **pontos acionáveis** para o andamento do trabalho.
5. **Antialucinação.** Registre só o que foi dito; escreva **"não registrado"** onde a
   informação não aparecer; **não invente** data, participantes, números ou decisões; não
   atribua fala a quem não falou. Preencha campos determinísticos (ex.: índice, data do
   arquivo) a partir da própria fonte, não da imaginação.
6. **Validação sem exposição.** Confira o resultado por **estrutura** (seções presentes,
   contagens, campos preenchidos) via script — sem despejar o conteúdo bruto confidencial no
   contexto de um agente externo. Peça revisão humana da qualidade.
7. **Proveniência & destino.** Documente **o que foi produzido on-premise × externo**; salve
   o artefato na **área restrita** do projeto; **nada confidencial entra em commit**. No
   handoff/versionado, referencie participantes/empresas **por índice ou papel**, não por nome.

## Caminho simplificado (material não-confidencial)

Se o material **não** é sensível (reunião interna sem dado de terceiros, perfil pessoal),
dispense o rigor on-premise/híbrido: use o melhor modelo disponível diretamente. Mantenha
apenas **antialucinação**, os **dois artefatos** e o **template** (o do usuário, se fornecido).

## Armadilhas comuns

- Colar transcrição bruta confidencial no chat de um agente externo "só para resumir".
- Confiar na **atribuição de orador** da transcrição automática — cruze com o áudio/ASR.
- Inventar data/participantes/decisões ausentes em vez de marcar "não registrado".
- Versionar a ATA confidencial — mantenha-a na área restrita; no versionado, use índice/papel.
- Tratar o resumo como fiel sem revisão humana quando gerado por modelo pequeno/local.

## Relação com outras skills

- [`secrets-guard`](../secrets-guard/SKILL.md) — antes de tocar material sensível.
- [`adr-writer`](../adr-writer/SKILL.md) — consultar a governança de dados; registrar
  exceções (ex.: quando uma parte precisa de modelo externo).
- [`handoff-updater`](../handoff-updater/SKILL.md) — após concluir, registrar o avanço
  (referenciando por índice).

## Definição de pronto da skill

- [ ] Sensibilidade classificada e governança de dados consultada (conflito → reportado).
- [ ] Consentimento/retenção verificados e registrados.
- [ ] Transcrição adequada à finalidade (ASR **local** quando confidencial / qualidade importa).
- [ ] ATA **+** seção de observações/sentimento produzidas no template (o do usuário ou,
  na ausência, o padrão da skill).
- [ ] Antialucinação aplicada ("não registrado" onde ausente; sem atribuição indevida).
- [ ] Proveniência (on-prem × externo) registrada; saída em área restrita; nada confidencial
  versionado.
