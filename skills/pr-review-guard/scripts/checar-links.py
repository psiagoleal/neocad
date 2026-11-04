#!/usr/bin/env python3
# Caminho relativo: skills/pr-review-guard/scripts/checar-links.py
"""Confere links relativos de Markdown: o arquivo existe e a âncora existe.

Uso:  checar-links.py <arquivo-ou-diretorio>...  [--http] [--excluir PADRÃO]...

Saída 0 nada a corrigir · 1 há link quebrado · 2 uso. Sem dependência externa.

Link http fica FORA por padrão: exige rede e deixa o gate instável. Com `--http` ele não
busca — apenas **relata** que não confere, para o link não passar por conferido em silêncio.

`--excluir` aceita padrão de caminho (fnmatch) e serve ao caso do repositório que guarda
*templates*: link de template resolve no projeto instalado, não na origem. Ex.:
`--excluir '*/templates/*' --excluir 'profiles/*'`.

⚠️ A regra de âncora é a do **GitHub** (minúsculas, acento preservado, sufixo `-1` para
cabeçalho repetido). Outros renderizadores divergem — o mdBook, por exemplo, gera âncora
própria. Num repositório publicado por outro renderizador, este verificador vira alarme falso.
"""
import fnmatch
import re
import sys
import pathlib
from urllib.parse import unquote

LINK = re.compile(r"(?<!\!)\[[^\]]*\]\(\s*<?([^)\s>]+)>?(?:\s+\"[^\"]*\")?\s*\)")
CABECALHO = re.compile(r"^(#{1,6})\s+(.*?)\s*#*\s*$")
EXTERNO = re.compile(r"^(?:[a-z][a-z0-9+.-]*:|//)", re.I)


def ancoras(md: pathlib.Path) -> set[str]:
    """Âncoras que o GitHub gera para os cabeçalhos — e os `id=` explícitos."""
    vistas, saida = {}, set()
    dentro_de_cerca = False
    for linha in md.read_text(encoding="utf-8", errors="replace").splitlines():
        if linha.lstrip().startswith("```"):
            dentro_de_cerca = not dentro_de_cerca
            continue
        if dentro_de_cerca:
            continue
        for ident in re.findall(r"""<a\s+(?:id|name)=["']([^"']+)["']""", linha):
            saida.add(ident)
        m = CABECALHO.match(linha)
        if not m:
            continue
        texto = re.sub(r"`([^`]*)`", r"\1", m.group(2))          # tira código
        texto = re.sub(r"\[([^\]]*)\]\([^)]*\)", r"\1", texto)    # tira link, guarda o rótulo
        texto = re.sub(r"[*_~]", "", texto)                       # tira ênfase
        base = re.sub(r"\s", "-", re.sub(r"[^\w\s-]", "", texto.strip().lower(), flags=re.U))
        n = vistas.get(base, 0)
        vistas[base] = n + 1
        saida.add(base if n == 0 else f"{base}-{n}")               # duplicado ganha sufixo
    return saida


def _apagar(m: "re.Match[str]") -> str:
    """Troca o trecho por espaços, preservando as quebras — o número da linha continua batendo."""
    return re.sub(r"[^\n]", " ", m.group(0))


def mascarar(texto: str) -> str:
    """Zera comentário HTML, bloco cercado e código EM LINHA.

    O código em linha importa mais do que parece: documentação que ENSINA a sintaxe de link
    escreve `[MT-n](detalhe)` dentro de crase, e o GitHub não o renderiza como link. Sem esta
    máscara o verificador acusa a própria documentação da convenção — aconteceu no primeiro uso.
    """
    texto = re.sub(r"<!--.*?-->", _apagar, texto, flags=re.S)
    texto = re.sub(r"^[ \t]*(`{3,}|~{3,}).*?^[ \t]*\1[^\n]*$", _apagar, texto, flags=re.S | re.M)
    return re.sub(r"(`+)(?:(?!\1).)*?\1", _apagar, texto)


def conferir(md: pathlib.Path, com_http: bool) -> list[str]:
    problemas = []
    cache: dict[pathlib.Path, set[str]] = {}
    for n, linha in enumerate(mascarar(md.read_text(encoding="utf-8", errors="replace")).splitlines(), 1):
        for alvo in LINK.findall(linha):
            if EXTERNO.match(alvo):
                if com_http and alvo.lower().startswith(("http://", "https://")):
                    problemas.append(f"{md}:{n}: http não é conferido por este script: {alvo}")
                continue
            caminho, _, ancora = alvo.partition("#")
            # O Markdown escapa espaço como %20 no caminho: sem decodificar, todo arquivo
            # com espaço no nome vira "inexistente".
            caminho, ancora = unquote(caminho), unquote(ancora)
            destino = (md.parent / caminho).resolve() if caminho else md.resolve()
            if not destino.exists():
                problemas.append(f"{md}:{n}: arquivo inexistente: {alvo}")
                continue
            if not ancora or destino.suffix.lower() != ".md":
                continue
            if destino not in cache:
                cache[destino] = ancoras(destino)
            if ancora not in cache[destino]:
                problemas.append(f"{md}:{n}: âncora inexistente: {alvo}")
    return problemas


def main(argv: list[str]) -> int:
    com_http = "--http" in argv
    excluir = [argv[i + 1] for i, a in enumerate(argv) if a == "--excluir" and i + 1 < len(argv)]
    pular = set(excluir)
    alvos = [a for i, a in enumerate(argv[1:], 1)
             if not a.startswith("--") and argv[i - 1] != "--excluir"]
    if not alvos:
        print(__doc__, file=sys.stderr)
        return 2
    arquivos: list[pathlib.Path] = []
    for a in alvos:
        p = pathlib.Path(a)
        if p.is_dir():
            arquivos += sorted(p.rglob("*.md"))
        elif p.is_file():
            arquivos.append(p)
        else:
            print(f"não encontrei: {a}", file=sys.stderr)
            return 2
    if pular:
        arquivos = [f for f in arquivos
                    if not any(fnmatch.fnmatch(str(f), p) for p in pular)]
    problemas = [q for f in arquivos for q in conferir(f, com_http)]
    for q in problemas:
        print(q)
    print(f"{len(arquivos)} arquivo(s) conferido(s), {len(problemas)} problema(s)")
    return 1 if problemas else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
