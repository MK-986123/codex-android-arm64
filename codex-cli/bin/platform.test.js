import test from "node:test";
import assert from "node:assert/strict";

import { detectTargetTriple, getPlatformPackage } from "./platform.js";

test("detects Android Termux arm64 target triple", () => {
  assert.equal(detectTargetTriple("android", "arm64"), "aarch64-linux-android");
  assert.equal(getPlatformPackage("aarch64-linux-android"), "@openai/codex-android-arm64");
});

test("keeps Linux target selection unchanged", () => {
  assert.equal(detectTargetTriple("linux", "x64"), "x86_64-unknown-linux-musl");
  assert.equal(detectTargetTriple("linux", "arm64"), "aarch64-unknown-linux-musl");
});

test("rejects unsupported Android architectures", () => {
  assert.equal(detectTargetTriple("android", "x64"), null);
});
