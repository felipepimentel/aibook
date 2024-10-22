# 📖 AIBook

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Repo](https://img.shields.io/badge/GitHub-Repository-blue?logo=github)](https://github.com/felipepimentel/aibook.git)

Welcome to **AIBook**! 🎉 This is a command-line application built in Rust that allows you to generate comprehensive summaries of EPUB e-books using advanced AI language models. With this tool, you can extract the essence of your favorite books, complete with images, references, and additional resources. 🚀

## ✨ Features

- **Detailed Summaries**: Generates in-depth summaries highlighting key points and insights from each chapter.
- **Image Extraction**: Extracts images from the e-book and includes them in the summary.
- **References & Resources**: Incorporates citations, references, and additional materials to enrich your understanding.
- **Customizable Output**: Adjust the level of detail, output language, and format to suit your preferences.
- **Easy to Use**: Simple command-line interface for quick and efficient summarization.

## 📋 Table of Contents

- [Prerequisites](#-prerequisites)
- [Installation](#-installation)
- [Configuration](#-configuration)
- [Usage](#-usage)
- [Customization](#-customization)
- [Contributing](#-contributing)
- [License](#-license)
- [Acknowledgments](#-acknowledgments)
- [Contact](#-contact)

## 🛠 Prerequisites

- **Rust**: Ensure you have Rust installed (version 1.56 or higher). Install it [here](https://www.rust-lang.org/tools/install).
- **OpenRouter API Key**: Required to access the AI language models. Get yours [here](https://openrouter.ai/).

## 📦 Installation

1. **Clone the repository**:

   ```bash
   git clone https://github.com/felipepimentel/aibook.git
   cd aibook
   ```

2. **Build the application**:

   ```bash
   cargo build --release
   ```

## ⚙️ Configuration

### API Key

To use **AIBook**, you need an API key from OpenRouter:

1. Sign up at [OpenRouter](https://openrouter.ai/) and obtain your API key.
2. Copy the `.env.sample` file to `.env`:

   ```bash
   cp .env.sample .env
   ```

3. Open the `.env` file and add your API key:

   ```env
   OPENROUTER_API_KEY=your-api-key-here
   ```

### Optional Settings

You can define additional settings in the `.env` file:

```env
# Model to be used (default: openai/gpt-3.5-turbo)
MODEL_NAME=openai/gpt-3.5-turbo

# Output language of the summary (default: en)
OUTPUT_LANGUAGE=en
```

### `.env.sample` File

An example `.env.sample` file is provided in the repository. It contains placeholders for the necessary environment variables. Copy it to create your own `.env` file.

## 🚀 Usage

Run the application by providing the path to the EPUB file you want to summarize:

```bash
cargo run --release -- --input /path/to/your/ebook.epub
```

### Available Options

- `--input`: Path(s) to the EPUB file(s).
- `--output_dir`: Directory where summaries and images will be saved (default: `output/`).
- `--api_key`: OpenRouter API key (can be set in the `.env` file).
- `--model`: Language model to be used.
- `--language`: Output language of the summary (default: `en`).
- `--detail_level`: Level of detail of the summary (`short`, `medium`, `long`; default: `medium`).
- `--output_format`: Output format (`markdown`, `html`; default: `markdown`).
- `--verbose`: Verbosity level of logs (use `-v` for more details).

### Full Example

```bash
cargo run --release -- \
  --input /path/to/your/ebook.epub \
  --output_dir /path/to/output/ \
  --language en \
  --detail_level long \
  --output_format markdown \
  --verbose
```

## 🎛 Customization

Feel free to adjust the application's behavior:

- **Custom Prompts**: Modify the prompts in `src/summarizer.rs` to change how the AI model generates summaries.
- **Source Code**: If you're familiar with Rust, you can adapt the code to your specific needs.

## 🤝 Contributing

Contributions are welcome! If you find a bug or have an idea to improve the project:

1. Open an issue describing the problem or suggestion.
2. Fork the repository.
3. Create a new branch for your contribution.
4. Submit a pull request detailing the changes.

## 📄 License

This project is licensed under the [MIT License](LICENSE).

## 🙏 Acknowledgments

We thank all contributors and users who make this project possible.

## 📫 Contact

For questions or suggestions:

- **Email**: [fpimentel88@gmail.com](mailto:fpimentel88@gmail.com)
- **GitHub**: [felipepimentel](https://github.com/felipepimentel)

---

We hope that **AIBook** is a useful tool for you to extract the most from your favorite e-books. Happy reading! 📚✨

---

## 📁 `.env.sample` File

Create a `.env.sample` file in the root directory of the repository with the following content:

```env
# OpenRouter API Key
OPENROUTER_API_KEY=your-api-key-here

# Model to be used (default: openai/gpt-3.5-turbo)
# MODEL_NAME=openai/gpt-3.5-turbo

# Output language of the summary (default: en)
# OUTPUT_LANGUAGE=en
```

This sample file provides a template for the necessary environment variables. Users can copy this file to `.env` and fill in their own API key and optional settings.

---

**Note:** Ensure that the `.env.sample` file has the `.env.sample` extension and that it is included in your repository. The `.env` file, containing your actual API keys and sensitive information, should not be committed to version control. It's good practice to add `.env` to your `.gitignore` file to prevent accidental commits.

---

By following these steps, you'll have a fully functional **AIBook** application ready to generate summaries of your EPUB e-books. If you have any questions or run into issues, feel free to reach out!

Happy summarizing! 🎉
