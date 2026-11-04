<!-- Caminho relativo: docs/verificacao-gpu.md -->

# Verificação da tela na GPU (spec 00, N5)

Roteiro manual do [MT-KV-15](tickets/kv-kernel-e-o-que-se-ve.md#mt-kv-15-verificação-na-nvidia):
o app empacotado abre o desenho sintético de 100 mil entidades, faz pan, zoom e uma
edição, e o resultado fica registrado abaixo.

O desenho usa só WebGL2 e canvas 2D (ADR 0008), APIs que não dependem de fabricante.
**AMD e Intel são presumidas compatíveis** até haver relato em contrário; quem as tiver,
rode o roteiro e abra uma issue com o bloco de resultado preenchido.

## Roteiro (Linux)

Todos os comandos rodam na raiz do repositório.

### 1. Ambiente

```bash
git pull
nvidia-smi --query-gpu=name,driver_version --format=csv,noheader
echo "sessão: $XDG_SESSION_TYPE"
dpkg -s libwebkit2gtk-4.1-0 | grep '^Version'
uname -r
```

### 2. Desenho sintético

```bash
node scripts/gerar-dxf-sintetico.mjs --entidades 100000 --camadas 20 --saida ~/neocad-100k.dxf
```

### 3. Build de teste (release) e execução

```bash
make dist-test-linux
./dist-test/NeoCAD-test-linux-x64/neocad
```

Se a janela abrir em branco, feche e repita com
`WEBKIT_DISABLE_DMABUF_RENDERER=1 ./dist-test/NeoCAD-test-linux-x64/neocad` e anote isso
no resultado (é um problema conhecido do WebKitGTK com o driver NVIDIA).

No app:

1. `Arquivo > Abrir`, escolha `~/neocad-100k.dxf`. Cronometre do clique em "Abrir" até
   o desenho aparecer (meta N1: até 5 s).
2. **Pan:** arraste com o botão do meio por uns 5 s em várias direções. Anote se fica
   fluido ou se trava.
3. **Zoom:** gire a roda do mouse para dentro e para fora por uns 5 s. Mesma anotação.
4. **Edição:** na linha de comando, digite `L` Enter, `0,0` Enter, `5000,5000` Enter e
   Enter de novo. A reta deve aparecer logo (meta N2: até 100 ms, ou seja, sem atraso
   perceptível). Depois, `Editar > Desfazer` deve tirá-la.
5. **Seleção:** clique num ponto vazio no canto superior esquerdo do desenho e depois
   no canto inferior direito (janela em volta de tudo). O desenho inteiro deve ficar
   destacado em azul sem atraso perceptível (meta S6: até 100 ms). `Esc` limpa.

### 4. Qual caminho de desenho foi usado

O build de depuração abre o inspetor do WebKit (botão direito > "Inspect Element"):

```bash
make tauri-debug-nobundle
./src-tauri/target/debug/neocad
```

Abra o mesmo desenho e, na aba "Console" do inspetor, cole:

```js
const c = document.querySelector('canvas[data-tela="kernel"]');
const gl = document.createElement('canvas').getContext('webgl2');
const info = gl?.getExtension('WEBGL_debug_renderer_info');
console.log({
	caminho: c?.dataset.caminho,
	renderizador: gl
		? gl.getParameter(info ? info.UNMASKED_RENDERER_WEBGL : gl.RENDERER)
		: 'sem WebGL2',
	multiDraw: !!gl?.getExtension('WEBGL_multi_draw')
});
```

`caminho` deve ser `webgl2`. Se der `canvas2d`, o WebKitGTK não ofereceu WebGL2: anote,
porque isso pede um ticket.

### 5. Resultado

Copie o bloco abaixo, preencha e mande de volta (ou cole direto na seção seguinte).

```text
data:
placa e driver (passo 1):
sessão (x11/wayland):
WebKitGTK:
kernel:
precisou de WEBKIT_DISABLE_DMABUF_RENDERER? sim/não
abertura (s):
pan: fluido / engasga / trava
zoom: fluido / engasga / trava
reta nova apareceu sem atraso? sim/não · desfazer tirou? sim/não
janela em volta de tudo sem atraso? sim/não
caminho / renderizador / multiDraw (passo 4):
observações:
```

## Resultados registrados

| data | sistema | placa e driver | webview | caminho | resultado |
| ---- | ------- | -------------- | ------- | ------- | --------- |

## Windows

Sem máquina Windows com NVIDIA disponível (o mantenedor tem só Linux, 2026-11-04). O
roteiro vale igual com `make dist-test-windows`, levando a pasta
`dist-test/NeoCAD-test-windows-x64` para o Windows; o build de teste usa o WebView2, e o
console fica em F12 no build de depuração. Fica pendente até haver relato.
