name := 'clapgrep'
appid := 'de.leopoldluley.Clapgrep'
frontend := 'clapgrep-gnome'

rootdir := ''
prefix := '/usr'

base-dir := absolute_path(clean(rootdir / prefix))

bin-src := 'target' / 'release' / frontend
bin-dst := base-dir / 'bin' / name

desktop := appid + '.desktop'
desktop-src := 'assets' / desktop
desktop-dst := base-dir / 'share' / 'applications' / desktop

metainfo := appid + '.metainfo.xml'
metainfo-src := 'assets' / metainfo
metainfo-dst := base-dir / 'share' / 'metainfo' / metainfo

icons-src := 'assets' / 'icons' / 'hicolor'
icons-dst := base-dir / 'share' / 'icons' / 'hicolor'

icon-svg-src := icons-src / 'scalable' / 'apps' / appid + '.svg'
icon-svg-dst := icons-dst / 'scalable' / 'apps' / appid + '.svg'

po-src := 'assets' / 'locale'
po-dst := base-dir / 'share' / 'locale'

clean:
  cargo clean

build *args: build-translations
  cargo build --package {{frontend}} {{args}}

check *args:
  cargo clippy --all-features {{args}} -- -W clippy::pedantic

run *args: build-translations
  env RUST_BACKTRACE=full cargo run --package {{frontend}} {{args}}

install:
  mkdir -p {{po-dst}}
  install -Dm0755 {{bin-src}} {{bin-dst}}
  install -Dm0755 {{desktop-src}} {{desktop-dst}}
  install -Dm0755 {{metainfo-src}} {{metainfo-dst}}
  install -Dm0755 {{icon-svg-src}} {{icon-svg-dst}}
  cp -r {{po-src}} {{po-dst}}

make-makefile:
  echo "# This file was generated by 'just make-makefile'" > build-aux/Makefile
  echo ".PHONY: all" >> build-aux/Makefile
  echo "all:" >> build-aux/Makefile
  just -n build --release 2>&1 | sed 's/^/\t/' | sed 's/\$/$$/g' >> build-aux/Makefile
  just -n --set prefix /app install 2>&1 | sed 's/^/\t/' >> build-aux/Makefile

prepare-flatpak: make-makefile
  python3 build-aux/flatpak-cargo-generator.py ./Cargo.lock -o build-aux/cargo-sources.json

install-flatpak:
  flatpak-builder flatpak-build gnome/de.leopoldluley.Clapgrep.json --force-clean --install --user

gettext *args:
  xgettext \
    --from-code=UTF-8 \
    --add-comments \
    --keyword=_ \
    --keyword=C_:1c,2 \
    --language=C \
    --output=po/messages.pot \
    --files-from=po/POTFILES \
    {{args}}

add-translation language:
  msginit -l {{language}}.UTF8 -o po/{{language}}.po -i po/messages.pot

build-translations:
  cat po/LINGUAS | while read lang; do \
    mkdir -p assets/locale/$lang/LC_MESSAGES; \
    msgfmt -o assets/locale/$lang/LC_MESSAGES/{{appid}}.mo po/$lang.po; \
  done
