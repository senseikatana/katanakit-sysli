# `katanakit-sysli` (wrapper npm)

Este paquete **no contiene código**: solo instala el binario `ksys` correcto
para tu máquina. Es el patrón esbuild/turbo (core en Rust, npm como instalador).

```bash
npm i -g katanakit-sysli
ksys --help
```

## Cómo elige el binario

`bin/ksys.js` hace esto, en orden:

1. Exige Linux (ksys habla con systemd por D-Bus).
2. `x64` → `@senseikatana/ksys-linux-x64`, `arm64` → `@senseikatana/ksys-linux-arm64`
   (llegan solos vía `optionalDependencies`, npm instala solo el de tu arch).
3. En este repo, si no hay paquete instalado, usa `target/release/ksys`
   o `target/debug/ksys` (para probar sin publicar).
4. Ejecuta el binario con tus mismos argumentos y terminal heredada.

## Versiones

Siempre acoplada a `Cargo.toml`. `node scripts/sync-version.cjs --check`
falla el CI si difieren.
