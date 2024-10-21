# aibook-cli

![aibook-cli Banner](https://your-image-url.com/banner.png)

**aibook-cli** é uma ferramenta de Linha de Comando (CLI) impulsionada por IA que revoluciona a forma como você consome livros. Transforme livros extensos em versões de bolso concisas e perspicazes com resumos capítulo por capítulo, utilizando tecnologia de IA de ponta e recursos robustos de extração de texto e imagem.

## 🚀 Recursos

- **🧠 Resumo com IA**: Gere resumos detalhados e inteligentes para cada capítulo.
- **🌐 Suporte Multilíngue**: Processe livros em Inglês e Português Brasileiro.
- **🔄 Integração Flexível de IA**: Escolha entre os provedores de IA `stackspot` e `openrouter` para resumos.
- **🖼️ Extração de Imagens**: Preserve o conteúdo visual dos arquivos EPUB para uma experiência de resumo mais rica.
- **📚 Formatos Duplos de Saída**: Obtenha um arquivo EPUB resumido e um resumo abrangente em markdown.
- **📊 Acompanhamento de Progresso**: Desfrute de uma experiência do usuário suave com indicadores visuais de progresso durante o processamento.

## 🛠️ Instalação

### Pré-requisitos

Antes de começar, certifique-se de ter as seguintes ferramentas instaladas:

- **Rust**: A espinha dorsal do nosso CLI. Instale em [rustup.rs](https://rustup.rs/)
- **Cargo**: Vem junto com o Rust - seu gerenciador de pacotes e ferramenta de build
- **Git**: Para clonar o repositório. Obtenha em [git-scm.com](https://git-scm.com/)

### Início Rápido

1. **Clone o Repositório**   ```
   git clone https://github.com/yourusername/aibook-cli.git
   cd aibook-cli   ```

2. **Compile o Projeto**   ```
   cargo build --release   ```

3. **Execute o aibook-cli**   ```
   ./target/release/aibook-cli   ```

## ⚙️ Configuração

1. Crie um arquivo `.env` na raiz do projeto.
2. Adicione suas chaves de API e configurações padrão:   ```
   API_KEY=sua_chave_api_aqui
   DEFAULT_LANGUAGE=ptbr
   AI_PROVIDER=openrouter   ```

## 🖥️ Uso

### Comandos

- `process`: Extrai texto de um arquivo EPUB
- `summarize`: Gera um livro de bolso resumido

### Opções

- `-f, --file <FILE>`: Especifica o arquivo EPUB de entrada
- `-l, --lang <LANGUAGE>`: Escolhe o idioma (en/ptbr)
- `-a, --ai-provider <PROVIDER>`: Seleciona o provedor de IA (stackspot/openrouter)

### Exemplos
