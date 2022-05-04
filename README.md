# Run on a local machine

## Create `mzoon` binary in `cargo_install_root` (only needed initially and for MoonZoon updates)

```bash
cd ${PROJECT_HOME}
cargo install mzoon \
    --git https://github.com/MoonZoon/MoonZoon \
    --rev 5769c15d6376ce591120c994764809c1a65ed7bd \
    --root cargo_install_root \
    --locked
mv cargo_install_root/bin/mzoon mzoon
rm -rf cargo_install_root
```

## Run the live-loading server

```bash
./mzoon start -o
```

## Deploy to Heroku

```bash
# Create the Heroku app
heroku create $PROJ --buildpack https://github.com/MoonZoon/heroku-buildpack-moonzoon.git

# Deploy to Heroku
git push heroku main

# Or, if you're still in a position where force-push might be appropriate
git push --force heroku main
```
