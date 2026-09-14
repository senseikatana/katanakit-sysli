# `@senseikatana/ksys-linux-arm64`

Solo el binario `ksys` compilado para Linux arm64 (gnu).

- **No lo instales directo**: instala `katanakit-sysli`, que elige este
  paquete solo vía `optionalDependencies`.
- `bin/ksys` **no se commitea** (ver `.gitignore`): lo pone el CI con
  `cargo build --release --target aarch64-unknown-linux-gnu`.
