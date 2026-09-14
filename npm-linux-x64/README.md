# `@senseikatana/ksys-linux-x64`

Solo el binario `ksys` compilado para Linux x86_64 (gnu).

- **No lo instales directo**: instala `katanakit-sysli`, que elige este
  paquete solo vía `optionalDependencies`.
- `bin/ksys` **no se commitea** (ver `.gitignore`): lo pone el CI con
  `cargo build --release --target x86_64-unknown-linux-gnu`.
