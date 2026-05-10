<p align="center"><code>npm i -g @openai/codex</code><br />or <code>brew install --cask codex</code></p>
<p align="center"><strong>Codex CLI</strong> is a coding agent from OpenAI that runs locally on your computer.
<p align="center">
  <img src="https://github.com/openai/codex/blob/main/.github/codex-cli-splash.png" alt="Codex CLI splash" width="80%" />
</p>
</br>
If you want Codex in your code editor (VS Code, Cursor, Windsurf), <a href="https://developers.openai.com/codex/ide">install in your IDE.</a>
</br>If you want the desktop app experience, run <code>codex app</code> or visit <a href="https://chatgpt.com/codex?app-landing-page=true">the Codex App page</a>.
</br>If you are looking for the <em>cloud-based agent</em> from OpenAI, <strong>Codex Web</strong>, go to <a href="https://chatgpt.com/codex">chatgpt.com/codex</a>.</p>

---

## Quickstart

### Installing and running Codex CLI

Install globally with your preferred package manager:

```shell
# Install using npm
npm install -g @openai/codex
```

```shell
# Install using Homebrew
brew install --cask codex
```

Then simply run `codex` to get started.

<details>
<summary>You can also go to the <a href="https://github.com/openai/codex/releases/latest">latest GitHub Release</a> and download the appropriate binary for your platform.</summary>

Each GitHub Release contains many executables, but in practice, you likely want one of these:

- macOS
  - Apple Silicon/arm64: `codex-aarch64-apple-darwin.tar.gz`
  - x86_64 (older Mac hardware): `codex-x86_64-apple-darwin.tar.gz`
- Linux
  - x86_64: `codex-x86_64-unknown-linux-musl.tar.gz`
  - arm64: `codex-aarch64-unknown-linux-musl.tar.gz`

Each archive contains a single entry with the platform baked into the name (e.g., `codex-x86_64-unknown-linux-musl`), so you likely want to rename it to `codex` after extracting it.

</details>

### Using Codex with your ChatGPT plan

Run `codex` and select **Sign in with ChatGPT**. We recommend signing into your ChatGPT account to use Codex as part of your Plus, Pro, Business, Edu, or Enterprise plan. [Learn more about what's included in your ChatGPT plan](https://help.openai.com/en/articles/11369540-codex-in-chatgpt).

You can also use Codex with an API key, but this requires [additional setup](https://developers.openai.com/codex/auth#sign-in-with-an-api-key).

## Docs

- [**Codex Documentation**](https://developers.openai.com/codex)
- [**Contributing**](./docs/contributing.md)
- [**Installing & building**](./docs/install.md)
- [**Open source fund**](./docs/open-source-fund.md)

This repository is licensed under the [Apache-2.0 License](LICENSE).

---

## Android / Termux (ARM64)

Codex TUI runs natively on Android ARM64 via [Termux](https://termux.dev/).

### Prerequisites

Install Termux from [F-Droid](https://f-droid.org/packages/com.termux/) (recommended) or Google Play.

### One-line Bootstrap

Run once inside Termux after installation:

```sh
bash scripts/termux-bootstrap.sh
```

This installs all required packages, builds the Rust TUI binary, and places `codex` in `$PREFIX/bin`.

### Manual Steps

```sh
# 1. Install system dependencies
pkg update && pkg install -y rust clang openssl-dev pkg-config nodejs-lts pnpm ripgrep git

# 2. Set env vars (add to ~/.bashrc for persistence)
export OPENSSL_DIR=$PREFIX
export PKG_CONFIG_PATH=$PREFIX/lib/pkgconfig
export OPENSSL_NO_VENDOR=1

# 3. Build the TUI
cd codex-rs
cargo build --release -p codex-tui

# 4. Install binary
install -Dm755 target/release/codex $PREFIX/bin/codex

# 5. Install TypeScript CLI (optional)
cd ../codex-cli
pnpm install
pnpm run build
```

Or use the just targets (requires `pkg install just`):

```sh
just termux-bootstrap   # first-time setup
just termux-build       # rebuild
just termux-install     # copy binary to $PREFIX/bin
```

### Known Limitations

| Feature | Status |
|---------|--------|
| TUI / chat | &#x2705; Fully functional |
| Shell execution | &#x2705; Runs via Termux `$SHELL` |
| Clipboard | &#x2705; Termux clipboard API used automatically |
| Sandboxing (bwrap/landlock) | &#x26A0;&#xFE0F; Disabled &#x2014; not available on Android |
| Voice / audio | &#x26A0;&#xFE0F; Disabled &#x2014; no NDK audio in Termux |
| IDE integration (IPC) | &#x2705; Works via Unix sockets |

Audio and sandboxing features compile as no-ops on Android &#x2014; the binary functions
fully for all text-based agent tasks.
