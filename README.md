# Run on a local machine

1. Check you've installed [Rust](https://www.rust-lang.org/):

    ```bash
    rustc -V # rustc 1.57.0 (f1edd0429 2021-11-29)
    ```

1. Go to the project root.

1. Install `mzoon` to `cargo_install_root`:

    ```bash
    cargo install mzoon --git https://github.com/MoonZoon/MoonZoon --rev 15cb619faca5f78a47e08f4af4bfa595f0eb64b1 --root cargo_install_root --locked
    ```

    - _Note:_ There will be faster and simpler ways with pre-compiled binaries.

1. Move `cargo_install_root/bin/mzoon` to the project root.

    ```bash
    mv cargo_install_root/bin/mzoon mzoon
    # or
    move cargo_install_root/bin/mzoon mzoon
    ```

1. Build and run:

    ```bash
    mzoon start -o
    ```

1. Deploy to Heroku

```bash
git push heroku main
```
