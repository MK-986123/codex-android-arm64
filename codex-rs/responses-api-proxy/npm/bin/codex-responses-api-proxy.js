#!/usr/bin/env node
// Entry point for the Codex responses API proxy binary.

import { spawn } from "node:child_process";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const TERMUX_PREFIX = "/data/data/com.termux/files/usr";

function isTermuxEnvironment(env) {
  return Boolean(env.TERMUX_VERSION) || env.PREFIX === TERMUX_PREFIX;
}

function determineTargetTriple(platform, arch, env = process.env) {
  if (platform === "android" || (platform === "linux" && isTermuxEnvironment(env))) {
    return arch === "arm64" ? "aarch64-linux-android" : null;
  }

  switch (platform) {
    case "linux":
      if (arch === "x64") {
        return "x86_64-unknown-linux-musl";
      }
      if (arch === "arm64") {
        return "aarch64-unknown-linux-musl";
      }
      break;
    case "darwin":
      if (arch === "x64") {
        return "x86_64-apple-darwin";
      }
      if (arch === "arm64") {
        return "aarch64-apple-darwin";
      }
      break;
    case "win32":
      if (arch === "x64") {
        return "x86_64-pc-windows-msvc";
      }
      if (arch === "arm64") {
        return "aarch64-pc-windows-msvc";
      }
      break;
    default:
      break;
  }
  return null;
}

const targetTriple = determineTargetTriple(process.platform, process.arch, process.env);
if (!targetTriple) {
  throw new Error(
    `Unsupported platform: ${process.platform} (${process.arch})`,
  );
}

const vendorRoot = path.join(__dirname, "..", "vendor");
const archRoot = path.join(vendorRoot, targetTriple);
const binaryBaseName = "codex-responses-api-proxy";
const binaryPath = path.join(
  archRoot,
  binaryBaseName,
  process.platform === "win32" ? `${binaryBaseName}.exe` : binaryBaseName,
);

const child = spawn(binaryPath, process.argv.slice(2), {
  stdio: "inherit",
});

child.on("error", (err) => {
  console.error(err);
  process.exit(1);
});

const forwardSignal = (signal) => {
  if (!child.killed) {
    try {
      child.kill(signal);
    } catch {
      /* ignore */
    }
  }
};

["SIGINT", "SIGTERM", "SIGHUP"].forEach((sig) => {
  process.on(sig, () => forwardSignal(sig));
});

const childResult = await new Promise((resolve) => {
  child.on("exit", (code, signal) => {
    if (signal) {
      resolve({ type: "signal", signal });
    } else {
      resolve({ type: "code", exitCode: code ?? 1 });
    }
  });
});

if (childResult.type === "signal") {
  process.kill(process.pid, childResult.signal);
} else {
  process.exit(childResult.exitCode);
}
