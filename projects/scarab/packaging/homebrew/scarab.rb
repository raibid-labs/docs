class Scarab < Formula
  desc "GPU-accelerated terminal emulator with plugin system"
  homepage "https://github.com/raibid-labs/scarab"
  version "0.1.0-alpha.7"
  license "MIT OR Apache-2.0"

  # URL will be updated when releases are available
  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/raibid-labs/scarab/releases/download/v#{version}/scarab-v#{version}-aarch64-apple-darwin.tar.gz"
    sha256 "PLACEHOLDER_SHA256_ARM64"
  elsif OS.mac? && Hardware::CPU.intel?
    url "https://github.com/raibid-labs/scarab/releases/download/v#{version}/scarab-v#{version}-x86_64-apple-darwin.tar.gz"
    sha256 "PLACEHOLDER_SHA256_X64"
  end

  depends_on "rust" => :build

  def install
    # Install pre-built binaries
    bin.install "scarab-daemon"
    bin.install "scarab-client" => "scarab"

    # Create wrapper script for convenient launching
    (bin/"scarab-terminal").write <<~EOS
      #!/bin/bash
      # Start daemon if not running
      if ! pgrep -x scarab-daemon > /dev/null; then
        #{bin}/scarab-daemon &
        sleep 1
      fi
      # Launch client
      exec #{bin}/scarab "$@"
    EOS

    chmod 0755, bin/"scarab-terminal"

    # Install shell completions
    generate_completions_from_executable(bin/"scarab", "completions", shells: [:bash, :zsh, :fish])

    # Install configuration examples
    (prefix/"examples").install Dir["examples/*"] if Dir.exist?("examples")

    # Install documentation
    doc.install "README.md", "LICENSE" if File.exist?("README.md")
  end

  def post_install
    # Create config directory
    config_dir = etc/"scarab"
    config_dir.mkpath

    # Install default config if it doesn't exist
    unless (config_dir/"config.toml").exist?
      (config_dir/"config.toml").write default_config
    end

    # Create runtime directory
    runtime_dir = var/"run/scarab"
    runtime_dir.mkpath
  end

  service do
    run [opt_bin/"scarab-daemon"]
    keep_alive true
    log_path var/"log/scarab-daemon.log"
    error_log_path var/"log/scarab-daemon-error.log"
    environment_variables PATH: std_service_path_env
  end

  def default_config
    <<~EOS
      # Scarab Terminal Configuration

      [terminal]
      shell = "/bin/zsh"
      font_size = 14.0
      font_family = "SF Mono"

      [theme]
      name = "default"

      [gpu]
      backend = "metal"
      vsync = true

      [plugins]
      enabled = true
      directory = "#{var}/scarab/plugins"
    EOS
  end

  def caveats
    <<~EOS
      Scarab has been installed!

      To start the daemon as a background service:
        brew services start scarab

      To launch the terminal:
        scarab-terminal

      Or run the client directly:
        scarab

      Configuration file is located at:
        #{etc}/scarab/config.toml

      For plugin development, see:
        https://github.com/raibid-labs/scarab/docs
    EOS
  end

  test do
    # Test daemon version
    assert_match "scarab-daemon", shell_output("#{bin}/scarab-daemon --version")

    # Test client version
    assert_match "scarab", shell_output("#{bin}/scarab --version")

    # Test daemon can start (briefly)
    pid = fork do
      exec bin/"scarab-daemon"
    end
    sleep 2
    Process.kill("TERM", pid)
    Process.wait(pid)
  end
end