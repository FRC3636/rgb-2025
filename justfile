build-pi:
    @echo "Building for rpi 4"
    cargo b --target aarch64-unknown-linux-gnu --release
upload user ip:
    @just build-pi
    @echo "Killing existing rgb-2025 on {{ip}}"
    ssh {{user}}@{{ip}} "sudo pkill rgb-2025 || true"
    @echo "Uploading to {{user}}@{{ip}}"
    scp target/aarch64-unknown-linux-gnu/release/rgb-2025 {{user}}@\[{{ip}}\]:~/rgb-2025-unwrapped
    scp rgb-2025-wrapper.sh {{user}}@\[{{ip}}\]:~/rgb-2025
    ssh {{user}}@{{ip}} "chmod +x ~/rgb-2025"
deploy user ip:
    just upload {{user}} {{ip}}
    @echo "Running rgb-2025 remotely on {{ip}}"
    ssh {{user}}@{{ip}} "~/rgb-2025"