#!/usr/bin/env node
/**
 * tauri-cli refuses to build when an npm plugin package and its Rust crate sit on
 * different major.minor versions ("plugin version mismatch"). Ranges like "^2.0.0"
 * make that happen silently on a fresh install, so pin npm packages exactly and
 * keep the Rust side on the same minor with "~".
 *
 * This check compares apps/windows/package.json against the resolved Cargo.lock
 * and fails with the exact pair that drifted.
 */
import { readFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const pkgPath = join(root, "apps/windows/package.json");
const lockPath = join(root, "Cargo.lock");

const pkg = JSON.parse(readFileSync(pkgPath, "utf8"));
const lock = readFileSync(lockPath, "utf8");

/** crate name -> resolved version, from the workspace lock file. */
function lockVersions() {
  const versions = new Map();
  for (const block of lock.split("[[package]]")) {
    const name = block.match(/^name = "(.*)"$/m)?.[1];
    const version = block.match(/^version = "(.*)"$/m)?.[1];
    if (name && version) versions.set(name, version);
  }
  return versions;
}

function minor(version) {
  const [major, min] = version.split(".");
  return `${major}.${min}`;
}

const deps = { ...pkg.dependencies, ...pkg.devDependencies };
const locked = lockVersions();
const errors = [];
const warnings = [];

const pluginCrates = [...locked.keys()].filter((name) => name.startsWith("tauri-plugin-"));

for (const crate of pluginCrates) {
  // tauri-plugin-autostart -> @tauri-apps/plugin-autostart
  const npmName = `@tauri-apps/${crate.replace(/^tauri-/, "")}`;
  const spec = deps[npmName];
  if (!spec) {
    warnings.push(`${crate} is in Cargo.lock but ${npmName} is not a dependency`);
    continue;
  }
  if (!/^\d+\.\d+\.\d+$/.test(spec)) {
    errors.push(
      `${npmName} uses "${spec}"; pin an exact version (e.g. "${locked.get(crate)}") so npm cannot drift away from ${crate}`,
    );
  }
  const declared = spec.match(/\d+\.\d+/)?.[0];
  if (declared && locked.get(crate) && declared !== minor(locked.get(crate))) {
    errors.push(
      `${npmName} ${spec} vs ${crate} ${locked.get(crate)}: major.minor must match (tauri-cli rejects the build otherwise)`,
    );
  }
}

// @tauri-apps/api ships with the `tauri` crate; keep the minors aligned so the
// JS runtime and the Rust host understand the same protocol.
const apiSpec = deps["@tauri-apps/api"];
const tauriVersion = locked.get("tauri");
if (apiSpec && tauriVersion) {
  const apiDeclared = apiSpec.match(/\d+\.\d+/)?.[0];
  if (apiDeclared && apiDeclared !== minor(tauriVersion)) {
    warnings.push(
      `@tauri-apps/api ${apiSpec} vs tauri crate ${tauriVersion}: minors differ, consider aligning`,
    );
  }
}

if (warnings.length) {
  for (const warning of warnings) console.warn(`warn: ${warning}`);
}

if (errors.length) {
  for (const error of errors) console.error(`error: ${error}`);
  console.error(
    "\nFix: pin @tauri-apps/plugin-* exactly in apps/windows/package.json and use ~<same minor> in apps/windows/src-tauri/Cargo.toml, then run `npm install` and `cargo update -p <crate>`.",
  );
  process.exit(1);
}

console.log(
  `Tauri plugin versions aligned for ${pluginCrates.length} crate(s): ` +
    pluginCrates.map((c) => `${c}@${locked.get(c)}`).join(", "),
);
