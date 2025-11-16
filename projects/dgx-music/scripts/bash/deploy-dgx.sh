#!/usr/bin/env bash
# Deploy DGX Music to DGX Spark via systemd

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
INSTALL_DIR="/opt/dgx-music"
SERVICE_USER="dgx-music"

echo "🚀 Deploying DGX Music to DGX Spark..."

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "❌ This script must be run as root (use sudo)"
    exit 1
fi

# Create service user
if ! id "$SERVICE_USER" &>/dev/null; then
    echo "Creating service user: $SERVICE_USER"
    useradd --system --home "$INSTALL_DIR" --shell /bin/false "$SERVICE_USER"
fi

# Create installation directory
echo "Creating installation directory..."
mkdir -p "$INSTALL_DIR"/{data,logs,configs}

# Copy application files
echo "Copying application files..."
rsync -av --exclude='venv' --exclude='__pycache__' --exclude='.git' \
    "$PROJECT_ROOT"/ "$INSTALL_DIR"/

# Set ownership
chown -R "$SERVICE_USER":"$SERVICE_USER" "$INSTALL_DIR"

# Create Python virtual environment
echo "Setting up Python environment..."
sudo -u "$SERVICE_USER" python3 -m venv "$INSTALL_DIR/venv"
sudo -u "$SERVICE_USER" "$INSTALL_DIR/venv/bin/pip" install --upgrade pip
sudo -u "$SERVICE_USER" "$INSTALL_DIR/venv/bin/pip" install -r "$INSTALL_DIR/requirements.txt"

# Download AI models
echo "Downloading AI models..."
sudo -u "$SERVICE_USER" "$INSTALL_DIR/venv/bin/python3" -c "
from audiocraft.models import MusicGen
print('Downloading MusicGen Small...')
MusicGen.get_pretrained('small')
print('✅ Models downloaded')
"

# Initialize database
echo "Initializing database..."
sudo -u "$SERVICE_USER" "$INSTALL_DIR/venv/bin/python3" -c "
from services.storage.database import init_db
init_db()
"

# Create systemd service file
echo "Creating systemd service..."
cat > /etc/systemd/system/dgx-music.service <<EOF
[Unit]
Description=DGX Music Generation Service
After=network.target

[Service]
Type=simple
User=$SERVICE_USER
Group=$SERVICE_USER
WorkingDirectory=$INSTALL_DIR
ExecStart=$INSTALL_DIR/venv/bin/uvicorn services.generation.api:app --host 0.0.0.0 --port 8000
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

# Resource limits
LimitNOFILE=65536
MemoryMax=30G

[Install]
WantedBy=multi-user.target
EOF

# Reload systemd
systemctl daemon-reload

# Enable and start service
echo "Enabling service..."
systemctl enable dgx-music

echo "Starting service..."
systemctl start dgx-music

# Check status
sleep 3
if systemctl is-active --quiet dgx-music; then
    echo "✅ DGX Music deployed and running!"
    echo ""
    echo "Service status:"
    systemctl status dgx-music --no-pager
    echo ""
    echo "Useful commands:"
    echo "  journalctl -u dgx-music -f    # View logs"
    echo "  systemctl restart dgx-music   # Restart service"
    echo "  systemctl stop dgx-music      # Stop service"
    echo ""
    echo "API available at: http://$(hostname):8000"
    echo "Health check: curl http://localhost:8000/health"
else
    echo "❌ Service failed to start"
    echo "Check logs: journalctl -u dgx-music -n 50"
    exit 1
fi
