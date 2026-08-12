<!-- Caminho relativo: skills/meeting-minutes/references/pipeline-recipes.md -->

# Receitas técnicas — pipeline on-premise de ATA/resumo

Receitas **neutras e opcionais** para o fluxo da skill `meeting-minutes`. Todas rodam
**localmente** (nada sai do perímetro). Ajuste caminhos ao projeto; grave saídas
confidenciais na **área restrita** e nunca as versione. Preferir ferramentas **open-source**.

Convenções abaixo: `<RESTRITO>` = pasta restrita do projeto; `<SCRATCH>` = temp de sessão.

---

## 1. Extrair texto de `.docx` sem dependências

`.docx` é um zip; o texto está em `word/document.xml`.

```python
import sys, zipfile, re
def docx_text(path):
    with zipfile.ZipFile(path) as z:
        xml = z.read("word/document.xml").decode("utf-8", "ignore")
    xml = re.sub(r"</w:p>", "\n", xml); xml = re.sub(r"<[^>]+>", "", xml)
    for a, b in [("&amp;","&"),("&lt;","<"),("&gt;",">"),("&quot;",'"'),("&apos;","'")]:
        xml = xml.replace(a, b)
    return re.sub(r"\n{3,}", "\n\n", xml).strip()
if __name__ == "__main__":
    print(docx_text(sys.argv[1]))
```

**Meça o tamanho antes de decidir o modelo** (imprime só contagem — sem conteúdo):
`words=$(python3 extract.py in.docx | wc -w)` → estimar tokens ≈ `words × 1.5` (PT).

---

## 2. ASR local (áudio/vídeo → transcrição)

Requisitos: `ffmpeg` + `faster-whisper` (CTranslate2). Instale em ambiente **isolado** para
não tocar as dependências do projeto (ex.: `uv run --with faster-whisper …`).

```python
# asr.py — transcreve arquivo de mídia; grava .txt; imprime só métricas (sem conteúdo).
import sys, time
from faster_whisper import WhisperModel
media, out = sys.argv[1], sys.argv[2]
# GPU (≥8 GB): large-v3/float16 é o melhor p/ PT. Sem GPU: medium/int8 (mais lento).
try:
    import ctranslate2
    dev, ctype, name = ("cuda", "float16", "large-v3") \
        if ctranslate2.get_cuda_device_count() > 0 else ("cpu", "int8", "medium")
except Exception:
    dev, ctype, name = "cpu", "int8", "medium"
m = WhisperModel(name, device=dev, compute_type=ctype)
t = time.time()
segs, info = m.transcribe(media, language="pt", beam_size=5, vad_filter=True)
txt = "".join(s.text for s in segs).strip()
open(out, "w", encoding="utf-8").write(txt + "\n")
print(f"[ASR] {dev}/{name} dur={info.duration:.0f}s proc={time.time()-t:.0f}s words={len(txt.split())}")
```

Notas:
- `faster-whisper large-v3` em GPU roda a ~0,15× tempo real (≈6× mais rápido que a duração).
- Extrair só o áudio ajuda: `ffmpeg -i in.mp4 -ac 1 -ar 16000 -c:a pcm_s16le out.wav`.
- **Termos técnicos:** passe `initial_prompt="<glossário: siglas, nomes de sistemas/pessoas>"`
  em `m.transcribe(...)` — condiciona o Whisper ao vocabulário do domínio e reduz os erros
  típicos da transcrição automática.
- **Diarização** (quem fala) não vem do Whisper. Se precisar de atribuição, ou use a
  transcrição automática (que traz rótulos de orador) **cruzada** com o ASR, ou adicione
  diarização (ex.: `pyannote`, requer token do modelo). Trabalhos longos: rodar em background.

---

## 3. Síntese qualitativa com LLM local (Ollama)

Mantém o conteúdo no perímetro. Modelo ~7B (ex.: `qwen2.5:7b`) cabe em 8 GB e vai bem em PT
para a **parte qualitativa**; para preenchimento estruturado preciso, revise (modelos pequenos
alucinam campos). Passe **transcrição + gabarito** e restrinja a saída à seção desejada.

```bash
curl -s http://localhost:11434/api/generate -d '{
  "model": "qwen2.5:7b",
  "prompt": "<INSTRUÇÕES + TRANSCRIÇÃO + GABARITO>",
  "stream": false,
  "options": {"temperature": 0.15, "num_ctx": 24576, "num_predict": 2200}
}' | python3 -c "import sys,json;open('<RESTRITO>/obs.md','w').write(json.load(sys.stdin)['response'])"
```

Dicas de prompt (reduzem alucinação): "baseie-se ESTRITAMENTE nos transcritos"; "liste TODOS
os sistemas citados, sem omitir"; "não invente nomes; use os rótulos de orador"; "escreva
'não registrado' quando ausente". `num_ctx` deve comportar entrada + saída; dois transcritos
(automático + ASR) juntos podem passar de 16k tokens.

**Valide o modelo antes do dado real** com um transcrito **sintético** (que você escreve, não
confidencial): rode o pipeline, leia essa saída sintética e julgue a qualidade — assim você
afere o modelo local sem ler conteúdo confidencial.

---

## 4. Validação estrutural sem expor conteúdo

Confirme que a saída está bem-formada **sem** despejar o texto no contexto de um agente externo:

```bash
f="<RESTRITO>/ata.md"
echo "secoes=$(grep -c '^## ' "$f") obs=$(grep -c '^### ' "$f") \
marcadas=$(grep -o '☑' "$f" | wc -l) chars=$(wc -c < "$f")"
```

Peça **revisão humana** da qualidade do conteúdo — a validação por script cobre só a forma.

---

## 5. Higiene de conformidade

- Saídas confidenciais na **área restrita**; confira que o versionamento as ignora
  (`git status` não deve listá-las).
- Remova derivados confidenciais do `<SCRATCH>` ao terminar (ex.: `.wav` de amostra).
- Registre a **proveniência** no rodapé da ATA (o que foi on-prem × externo) e, se um passo
  exigiu modelo externo sobre dado confidencial, registre a **autorização** (ADR/nota).
