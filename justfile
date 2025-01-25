build-pi:
    @echo "Building for rpi 4"
    cargo b --target aarch64-unknown-linux-gnu --release
upload ip:
    @just build-pi
    @echo "Killing existing rgb-2025 on {{ip}}"
    ssh {{ip}} "sudo pkill rgb-2025 || true"
    @echo "Uploading to {{ip}}"
    scp target/aarch64-unknown-linux-gnu/release/rgb-2025 {{ip}}:~/rgb-2025
deploy ip:
    just upload {{ip}}
    @echo "Running rgb-2025 remotely on {{ip}}"
    ssh {{ip}} "~/rgb-2025"