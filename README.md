<p align="center">
  <img src="assets/blue_pages_icon.png" alt="Blue Pages Icon" width="96" />
</p>

<h1 align="center">Blue Pages</h1>

<p align="center">
  Leitor desktop nativo de <strong>.epub</strong> e <strong>.pdf</strong> escrito em <strong>Rust</strong>, rápido, fluido e com tema escuro inspirado no Karikari.app.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-2021-000000?logo=rust&logoColor=white" alt="Rust 2021" />
  <img src="https://img.shields.io/badge/GUI-egui%20%2F%20eframe-38BDF8" alt="egui / eframe" />
  <img src="https://img.shields.io/badge/Formatos-EPUB%20%7C%20PDF-0284C7" alt="EPUB e PDF" />
  <img src="https://img.shields.io/badge/Licen%C3%A7a-MIT-22C55E" alt="Licença MIT" />
</p>

## Recursos

- **Tela inicial com leituras recentes**:
  - Destaque principal com carrossel das últimas obras lidas, capas em cache e retomada instantânea.
  - Seção **Continuar lendo** com barra de progresso, porcentagem lida e busca em tempo real por título, autor ou formato.
  - Suporte a **arrastar e soltar** (Drag & Drop) arquivos `.epub` ou `.pdf` diretamente na janela.
- **Modo de leitura fluido**:
  - **Por páginas** (com setas laterais flutuantes) e **Rolagem vertical contínua**.
  - **Renderização assíncrona de PDF** em thread dedicada com fila LIFO e pré-carregamento — ideal para PDFs grandes (300+ páginas) sem travar a interface.
  - **Sumário lateral** clicável com destaque da posição atual e progresso total do livro.
  - **Personalização completa de EPUB**: 6 fontes embutidas (*Caladea*, *Carlito*, *Liberation Serif*, *DejaVu Serif*, *Liberation Mono* e *Sans*), ajuste de tamanho da fonte e 5 estilos de cores (*Escuro*, *AMOLED*, *Sépia*, *Claro* e *Noite Suave*).
  - **Indicador de páginas e porcentagem** na barra inferior, no sumário e nos cartões iniciais.

## Atalhos de teclado

| Tecla | Ação |
| :--- | :--- |
| `←` / `→` | Página ou capítulo anterior / próximo |
| `F` | Alternar tela cheia |
| `Esc` | Sair da tela cheia ou fechar ajustes |
| `+` / `-` | Aumentar / diminuir zoom |

## Como compilar e executar

### 1. Pré-requisitos
Instale o toolchain do Rust (`rustup` / `cargo`) e um motor de renderização para PDF:

- **Pdfium (recomendado — máximo desempenho)**: baixe o binário pré-compilado para seu sistema em [pdfium-binaries](https://github.com/bblanchon/pdfium-binaries/releases) e coloque `libpdfium.so` (Linux), `pdfium.dll` (Windows) ou `libpdfium.dylib` (macOS) na pasta do executável.
- **Poppler (`pdftoppm`, fallback automático)**:
  - **CachyOS / Arch Linux**: `sudo pacman -S poppler`
  - **Ubuntu / Debian**: `sudo apt install poppler-utils`
  - **macOS**: `brew install poppler`

> Arquivos `.epub` funcionam 100% em Rust puro, sem dependências externas.

### 2. Executar
```bash
cargo run --release
```

### 3. Instalar no menu de aplicativos do Linux (CachyOS / Arch / Ubuntu)
```bash
chmod +x linux/install-linux.sh
./linux/install-linux.sh
```
Isso compila a versão `--release`, instala o binário em `~/.local/bin/blue-pages`, o ícone da ararinha-azul em `~/.local/share/icons/` e o atalho no menu do sistema.

## Licença

Distribuído sob a licença **MIT**. Veja o arquivo [LICENSE](LICENSE) para mais detalhes.
