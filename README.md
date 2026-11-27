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

---

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

---

## Instalação (versão pronta — Recomendado)

Você **não precisa instalar Rust** para usar o Blue Pages. Basta baixar a versão pronta para o seu sistema na aba **[Releases](../../releases/latest)** deste repositório (o motor `Pdfium` já vem incluso nos pacotes!).

### Linux (CachyOS, Arch, Ubuntu, Fedora, Mint, etc.)
1. Acesse **[Releases](../../releases/latest)** e baixe o arquivo **`blue-pages-linux-x86_64.tar.gz`**.
2. Extraia e instale no menu de aplicativos pelo terminal:
   ```bash
   tar -xzf blue-pages-linux-x86_64.tar.gz
   cd blue-pages
   ./install.sh
   ```
3. Pronto! O **Blue Pages** aparecerá no seu menu de aplicativos com o ícone da ararinha-azul (você também pode rodar diretamente com `./blue-pages`).

### Windows
1. Acesse **[Releases](../../releases/latest)** e baixe o arquivo **`blue-pages-windows-x86_64.zip`**.
2. Extraia a pasta em qualquer lugar do seu computador.
3. Dê dois cliques em **`blue-pages.exe`** (mantenha o arquivo `pdfium.dll` na mesma pasta).

### macOS (Apple Silicon)
1. Acesse **[Releases](../../releases/latest)** e baixe **`blue-pages-macos-arm64.tar.gz`**.
2. Extraia e execute:
   ```bash
   tar -xzf blue-pages-macos-arm64.tar.gz
   cd blue-pages
   ./blue-pages
   ```

---

## Compilar a partir do código-fonte

Se preferir compilar localmente (ou contribuir com o projeto):

### 1. Instale o Rust e as dependências do sistema
Instale o Rust em [rustup.rs](https://rustup.rs/) e as bibliotecas gráficas da sua distro (no Linux):

- **CachyOS / Arch Linux**:
  ```bash
  sudo pacman -S base-devel gtk3 poppler
  ```
- **Ubuntu / Debian**:
  ```bash
  sudo apt install build-essential pkg-config libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev poppler-utils
  ```
- **Fedora**:
  ```bash
  sudo dnf install gcc pkg-config gtk3-devel libxcb-devel libxkbcommon-devel openssl-devel poppler-utils
  ```

> **Dica de performance para PDF**: para máxima velocidade ao compilar do fonte, baixe o `libpdfium.so` / `pdfium.dll` / `libpdfium.dylib` em [pdfium-binaries](https://github.com/bblanchon/pdfium-binaries/releases) e coloque na raiz do projeto. Sem ele, o Blue Pages usa automaticamente o `poppler` (`pdftoppm`) como fallback. Arquivos `.epub` funcionam em Rust puro sem nenhuma dependência externa.

### 2. Clone e execute
```bash
git clone https://github.com/SEU_USUARIO/blue-pages.git
cd blue-pages
cargo run --release
```

### 3. Instalar no menu do Linux após compilar
```bash
chmod +x linux/install-linux.sh
./linux/install-linux.sh
```

---

## Atalhos de teclado

| Tecla | Ação |
| :--- | :--- |
| `←` / `→` | Página ou capítulo anterior / próximo |
| `F` | Alternar tela cheia |
| `Esc` | Sair da tela cheia ou fechar ajustes |
| `+` / `-` | Aumentar / diminuir zoom |

---

## Como publicar um novo Release (para mantenedores)

Este repositório já possui GitHub Actions configurado (`.github/workflows/release.yml`). Para gerar automaticamente os instaladores de Linux, Windows e macOS na aba **Releases**, basta criar e enviar uma tag de versão:

```bash
git tag v0.2.0
git push origin v0.2.0
```

Em alguns minutos o GitHub compila os três sistemas, inclui o `Pdfium` em cada pacote e publica o Release automaticamente.

---

## Licença

Distribuído sob a licença **MIT**. Veja o arquivo [LICENSE](LICENSE) para mais detalhes.
