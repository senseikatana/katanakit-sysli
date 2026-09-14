#!/usr/bin/env node
// Mantiene acopladas las versiones: Cargo.toml == los 3 package.json.
//   node scripts/sync-version.cjs          -> escribe la versión de Cargo en npm
//   node scripts/sync-version.cjs --check  -> solo verifica (para CI)
"use strict";

const { readFileSync, writeFileSync } = require("node:fs");
const path = require("node:path");

const ROOT = path.join(__dirname, "..");
const MANIFESTS = [
  "npm/package.json",
  "npm-linux-x64/package.json",
  "npm-linux-arm64/package.json",
];

function cargoVersion() {
  const cargo = readFileSync(path.join(ROOT, "Cargo.toml"), "utf8");
  const m = cargo.match(/^version\s*=\s*"([^"]+)"/m);
  if (!m) throw new Error("no se encontró version en Cargo.toml");
  return m[1];
}

function readJson(p) {
  return JSON.parse(readFileSync(p, "utf8"));
}

const checkOnly = process.argv.includes("--check");
const version = cargoVersion();
let dirty = false;

for (const rel of MANIFESTS) {
  const file = path.join(ROOT, rel);
  const json = readJson(file);
  const before = JSON.stringify(json);
  // optionalDependencies del wrapper también apuntan a las plataformas.
  if (json.optionalDependencies) {
    for (const key of Object.keys(json.optionalDependencies)) {
      if (key.startsWith("@senseikatana/ksys-linux-")) {
        json.optionalDependencies[key] = version;
      }
    }
  }
  if (json.version !== version) {
    if (checkOnly) console.error(`DESYNC ${rel}: tiene ${json.version}, Cargo dice ${version}`);
    json.version = version;
  }
  if (JSON.stringify(json) !== before) {
    dirty = true;
    if (!checkOnly) {
      writeFileSync(file, JSON.stringify(json, null, 2) + "\n");
      console.log(`sync ${rel} -> ${version}`);
    }
  }
}

if (checkOnly && dirty) {
  console.error("Corre: node scripts/sync-version.mjs");
  process.exit(1);
}
if (!dirty) console.log(`OK todas en ${version}`);
