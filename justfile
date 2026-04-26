default:
    @just --list

build:
    cargo build --release

run:
    cargo run

check:
    cargo check --all-features

test:
    cargo test --all-features

fmt:
    cargo fmt

clippy:
    cargo clippy --all-features -- -D warnings

PREFIX := env_var_or_default("PREFIX", env_var("HOME") + "/.local")

install: build
    install -Dm755 target/release/systrkr "{{PREFIX}}/bin/systrkr"
    install -Dm644 data/com.system76.SysTrkr.desktop "{{PREFIX}}/share/applications/com.system76.SysTrkr.desktop"
    install -Dm644 data/icons/com.system76.SysTrkr.svg "{{PREFIX}}/share/icons/hicolor/scalable/apps/com.system76.SysTrkr.svg"
    @echo "Installed to {{PREFIX}}. Add the applet via cosmic-settings → Panel."

uninstall:
    rm -f "{{PREFIX}}/bin/systrkr"
    rm -f "{{PREFIX}}/share/applications/com.system76.SysTrkr.desktop"
    rm -f "{{PREFIX}}/share/icons/hicolor/scalable/apps/com.system76.SysTrkr.svg"
