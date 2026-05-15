# Build cross-platform para Windows

## Objetivo

Este documento descreve os fluxos Windows x64 do NeoCAD a partir de Linux/WSL usando **CMake** como orquestrador para entregar:

1. uma versão **portable** próxima de um `.zip` extraível;
2. um **instalador simples** com NSIS, evitando admin sempre que possível.

## Estratégia atual

O repositório mantém duas saídas principais para Windows x64:

- **Portable `.zip`**: gerado a partir de `tauri build --no-bundle`, depois reorganizado por `cmake/StageWindowsPortable.cmake`.
- **NSIS current-user**: gerado por `src-tauri/tauri.windows.conf.json`, com `installMode: currentUser` e `webviewInstallMode: embedBootstrapper`.

Quando o time quiser um pacote mais autocontido, existe também um caminho opcional com **Fixed WebView2 Runtime**:

- **portable com Fixed Runtime**: inclui o runtime extraído dentro do `.zip` e usa `NeoCAD-portable.cmd` para definir `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER`;
- **NSIS current-user com Fixed Runtime**: gerado por `src-tauri/tauri.windows.fixed-runtime.conf.json`.

## Escopo realista

O fluxo atual foi desenhado para:

- cross-build **Windows x64**;
- geração de **`.zip` portable**;
- geração de **instalador NSIS**;
- opção adicional de **Fixed WebView2 Runtime** para ambientes offline ou controlados;
- uso de **CMake** para orquestrar comandos do `pnpm`, `cargo` e `tauri`.

## Limitações importantes

- **MSI/WiX não é suportado a partir de Linux/WSL**. Para MSI, o caminho seguro continua sendo runner Windows nativo.
- O fluxo cross-platform do Tauri 2 para Windows é possível, mas **menos testado** do que builds nativos em Windows.
- Assinatura de binários Windows em cross-build exige ferramenta externa; por isso os targets atuais usam `--no-sign`.
- O **Fixed WebView2 Runtime é viável**, mas aumenta bastante o tamanho do artefato e transfere para o projeto a responsabilidade de atualizar esse runtime.
- O pacote portable do NeoCAD **não registra associações de arquivo**. Associações `DWG`/`DXF` continuam sendo benefício do instalador NSIS.
- Se o pacote portable usar **Fixed Runtime**, a orientação é abrir o app pelo launcher `NeoCAD-portable.cmd`, não diretamente pelo `NeoCAD.exe`.

## Pré-requisitos no Linux/WSL

Segundo a documentação oficial do Tauri 2, o fluxo de cross-build para Windows requer pelo menos:

- `nsis`
- `llvm`
- `lld`
- `clang`
- target Rust `x86_64-pc-windows-msvc`
- `cargo-xwin`
- binário `llvm-rc` disponível no host para compilar recursos Windows durante o build do Tauri

Exemplo de preparação em Ubuntu/Debian:

```bash
sudo apt install nsis llvm lld clang cabextract
rustup target add x86_64-pc-windows-msvc
cargo install --locked cargo-xwin
```

> `cabextract` é opcional, mas útil para extrair o arquivo `.cab` do Fixed WebView2 Runtime a partir de Linux/WSL.

### Verificação rápida do host

Antes de rodar os targets Windows, vale confirmar:

```bash
which cargo-xwin
which llvm-rc
```

Se a sua distribuição instalar apenas uma variante versionada, como `llvm-rc-14`, você pode apontá-la manualmente:

```bash
export RC_x86_64_pc_windows_msvc=/usr/bin/llvm-rc-14
```

Os targets CMake do NeoCAD também tentam localizar variantes comuns em `/usr/lib/llvm-*/bin`, mas o binário ainda precisa existir no host.

## Preparar o Fixed WebView2 Runtime

### Viabilidade

O Tauri 2 suporta `bundle.windows.webviewInstallMode.type = "fixedRuntime"` para instaladores Windows. Isso torna o **Fixed WebView2 Runtime tecnicamente viável** para o NeoCAD.

Para o artefato portable, o Tauri não gera um “portable bundle” dedicado. Neste repositório, o fluxo portable é montado em duas etapas:

1. `tauri build --no-bundle` gera o `neocad.exe`;
2. `cmake/StageWindowsPortable.cmake` monta a pasta portable, copia um launcher `.cmd` e, se houver runtime extraído, inclui esse runtime no `.zip`.

### Onde extrair

Por padrão, os targets fixed-runtime esperam o runtime extraído em:

- `.webview2/fixed-runtime-x64`

Essa pasta **não deve ser commitada**. Ela é tratada como artefato local de build.

> O runtime fixo foi movido para fora de `build/` porque o `pnpm build` do frontend limpa a pasta de saída do SvelteKit/Vite durante o `tauri build`, o que removia `build/webview2` no meio do processo.

### Como extrair no Linux/WSL

Exemplo com `cabextract`:

```bash
mkdir -p .webview2/fixed-runtime-x64
cabextract -d .webview2/fixed-runtime-x64 Microsoft.WebView2.FixedVersionRuntime.<versao>.x64.cab
```

### Como extrair no Windows

Exemplo em PowerShell:

```powershell
New-Item -ItemType Directory -Force .webview2\fixed-runtime-x64
Expand .\Microsoft.WebView2.FixedVersionRuntime.<versao>.x64.cab -F:* .\.webview2\fixed-runtime-x64
```

### Validação mínima

Depois da extração, a pasta deve conter `msedgewebview2.exe` na raiz.

### Observação sobre limpeza

Como o runtime agora fica em `.webview2/`, o target `make clean` não remove mais esse cache local.

## Presets disponíveis

### Configurar CMake

```bash
cmake --preset linux-default
```

### Rodar smoke checks

```bash
cmake --build --preset smoke
```

### Gerar bundle Linux

```bash
cmake --build --preset linux-bundle
```

### Gerar portable `.zip`

```bash
cmake --build --preset windows-x64-portable
```

### Gerar portable `.zip` exigindo Fixed Runtime

```bash
cmake --build --preset windows-x64-portable-fixed-runtime
```

### Gerar instalador NSIS current-user

```bash
cmake --build --preset windows-x64-nsis
```

### Gerar instalador NSIS current-user com Fixed Runtime

```bash
cmake --build --preset windows-x64-nsis-fixed-runtime
```

## Comportamento dos artefatos Windows

### Portable (`windows-x64-portable`)

- sempre produz uma pasta staged e um `.zip` em `build/windows/portable/`;
- se `.webview2/fixed-runtime-x64` existir e contiver `msedgewebview2.exe`, o runtime é copiado para `webview2-fixed-runtime/` dentro do pacote;
- o launcher `NeoCAD-portable.cmd` tenta aplicar `icacls` em modo best-effort e seta `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER` para o runtime empacotado;
- se o runtime não existir, o `.zip` ainda é gerado, mas o app depende de um WebView2 Runtime já presente no sistema.

### Portable fixed runtime (`windows-x64-portable-fixed-runtime`)

- usa o mesmo fluxo do target portable padrão;
- falha cedo se o runtime extraído estiver ausente ou inválido;
- é o target mais próximo de um `.zip` realmente autocontido dentro das limitações atuais do Tauri 2.

### NSIS (`windows-x64-nsis`)

- usa `src-tauri/tauri.windows.conf.json`;
- mantém `installMode: currentUser`, evitando admin por padrão;
- usa `webviewInstallMode: embedBootstrapper`, equilibrando tamanho pequeno com instalação automática do runtime quando necessário;
- preserva associações `DWG` e `DXF`.

### NSIS fixed runtime (`windows-x64-nsis-fixed-runtime`)

- usa `src-tauri/tauri.windows.fixed-runtime.conf.json`;
- continua em modo `currentUser`;
- embute o runtime fixo extraído localmente;
- é indicado para ambientes offline ou rigidamente controlados.

## Troubleshooting rápido

### Erro `NotAttempted("llvm-rc")`

Se o build falhar com algo como:

```text
called `Result::unwrap()` on an `Err` value: NotAttempted("llvm-rc")
```

isso normalmente significa que o host Linux/WSL ainda **não tem `llvm-rc` disponível** para o `tauri-winres`.

A correção mais comum é instalar LLVM no host:

```bash
sudo apt install llvm lld clang
```

Depois disso, confirme:

```bash
which llvm-rc
```

Se o binário existir apenas em forma versionada, exporte a variável de ambiente compatível com o target:

```bash
export RC_x86_64_pc_windows_msvc=/usr/bin/llvm-rc-14
```

### Mensagem `package.metadata does not exist`

Essa linha pode aparecer no log do build, mas **não é a causa principal** do erro mostrado acima. Quando o backtrace termina em `NotAttempted("llvm-rc")`, o problema real é a ausência do compilador de recursos Windows no host.

## Limitações específicas do Fixed Runtime

Segundo a documentação da Microsoft:

- para apps Win32 unpackaged em Windows 10, Fixed Runtime 120+ exige permissões adicionais (`ALL APPLICATION PACKAGES` e `ALL RESTRICTED APPLICATION PACKAGES`) na pasta do runtime;
- o launcher portable tenta aplicar essas ACLs com `icacls` em modo best-effort, mas isso ainda pressupõe que o usuário extraia o `.zip` em um caminho local gravável e abra o app pelo launcher;
- o Fixed Runtime **não funciona a partir de localização de rede/UNC**;
- o runtime **não se atualiza sozinho**: cada refresh de segurança exige baixar e reempacotar uma nova versão.

## Arquivos relevantes

- `CMakeLists.txt`
- `CMakePresets.json`
- `Makefile`
- `cmake/StageWindowsPortable.cmake`
- `cmake/NeoCADPortableLauncher.cmd`
- `cmake/VerifyWindowsFixedRuntime.cmake`
- `src-tauri/tauri.windows.conf.json`
- `src-tauri/tauri.windows.fixed-runtime.conf.json`

## Recomendação de release

A combinação mais equilibrada hoje para o NeoCAD é:

- **portable `.zip`** para o cenário mais próximo de “extrair e usar”; se o destino exigir maior autonomia, usar a variante com Fixed Runtime;
- **NSIS current-user** como instalador padrão, porque reduz atrito e evita admin na maioria dos cenários;
- **runner Windows nativo** para releases oficiais, assinatura e validação final do instalador.
