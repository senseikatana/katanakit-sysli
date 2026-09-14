#!/usr/bin/env node
// ksys — shim del wrapper npm `katanakit-sysli`.
//
// Que hace (en orden):
//   1. Verifica que corre en Linux (ksys habla con systemd por D-Bus).
//   2. Elige el paquete de plataforma segun process.arch.
//   3. Ejecuta el binario Rust con los mismos argumentos (stdio heredado,
//      o sea la TUI recibe tu terminal tal cual).
//
// En desarrollo (este repo) usa target/release o target/debug si no hay
// paquete de plataforma instalado. En produccion (npm i -g) el binario
// viene de @senseikatana/ksys-linux-x64 | -arm64 via optionalDependencies.
"use strict";

const { existsSync } = require("node:fs");
const { spawnSync } = require("node:child_process");
const path = require("node:path");

const ARCH_TO_PKG = {
  x64: "@senseikatana/ksys-linux-x64",
  arm64: "@senseikatana/ksys-linux-arm64",
};

function fail(msg) {
  console.error(`ksys: ${msg}`);
  process.exit(1);
}

// 1. Solo Linux: systemd no existe en macOS/Windows.
if (process.platform !== "linux") {
  fail(
    `requiere Linux + systemd (estás en ${process.platform}). ` +
      `Si quieres compilar desde fuente en Linux: cargo install --path .`
  );
}

// 2. Arquitectura soportada.
const pkgName = ARCH_TO_PKG[process.arch];
if (!pkgName) {
  fail(
    `arquitectura no soportada: ${process.arch}. ` +
      `Soportadas: ${Object.keys(ARCH_TO_PKG).join(", ")}. ` +
      `Alternativa: compila con cargo build --release.`
  );
}

// 3a. Producción: binario del paquete de plataforma.
let binary = null;
try {
  binary = require.resolve(`${pkgName}/bin/ksys`);
} catch {
  binary = null;
}

// 3b. Desarrollo: fallback al build local de cargo (para probar sin publicar).
if (!binary || !existsSync(binary)) {
  const candidates = [
    path.join(__dirname, "..", "..", "target", "release", "ksys"),
    path.join(__dirname, "..", "..", "target", "debug", "ksys"),
  ];
  binary = candidates.find((p) => existsSync(p)) || null;
}

if (!binary) {
  fail(
    `no se encontró el binario.\n` +
      `  - Instalado por npm: reinstala con npm i -g katanakit-sysli\n` +
      `  - En este repo: corre primero cargo build --release`
  );
}

// 4. Aviso rápido si PID 1 no es systemd (containers sin systemd, WSL1...).
//    No bloquea: el TUI tiene fallback a systemctl, pero avisamos.
try {
  const fs = require("node:fs");
  const comm = fs.readFileSync("/proc/1/comm", "utf8").trim();
  if (comm !== "systemd" && !process.env.KSYS_ALLOW_NO_SYSTEMD) {
    console.error(
      `ksys: aviso — PID 1 es "${comm}", no systemd. ` +
        `Algunas vistas usarán fallback limitado.`
    );
  }
} catch {
  // Sin /proc (raro): seguimos, el binario decide.
}

// 5. Exec con los mismos args y terminal heredada.
const res = spawnSync(binary, process.argv.slice(2), { stdio: "inherit" });
process.exit(res.status ?? 1);
