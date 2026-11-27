#!/usr/bin/env bash
# Non-destructively deploys `target/release/ella-world-server`, `public/` (including signed Android APK
# & release metadata), and `config/` to `robert@mail:/opt/ella-world/`, ensures `ELLA_SUPERADMIN_PASSWORD`
# is configured in `/etc/systemd/system/ella-world.service.d/override.conf` without overwriting existing
# secrets, and restarts `ella-world.service`.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${REPO_ROOT}"

SSH_HOST="${ELLA_PROD_SSH_HOST:-robert@mail}"

if [[ ! -x "${REPO_ROOT}/target/release/ella-world-server" ]]; then
  echo "ERROR: target/release/ella-world-server not found. Run ./scripts/remote_build_android.sh first." >&2
  exit 1
fi
if [[ ! -f "${REPO_ROOT}/public/download/ella-world.apk" ]]; then
  echo "ERROR: public/download/ella-world.apk not found. Run ./scripts/remote_build_android.sh first." >&2
  exit 1
fi

STAMP="$(date +%Y%m%d-%H%M%S)"

echo "🚀 [1/4] Uploading release binary & static assets (including signed ella-world.apk) to ${SSH_HOST}..."
ssh -o BatchMode=yes "${SSH_HOST}" "mkdir -p ~/deploy-stage/ella-world/public"
rsync -az "${REPO_ROOT}/target/release/ella-world-server" "${SSH_HOST}:~/deploy-stage/ella-world/ella-world-server.new"
rsync -az "${REPO_ROOT}/public/" "${SSH_HOST}:~/deploy-stage/ella-world/public/"
rsync -az "${REPO_ROOT}/config/" "${SSH_HOST}:~/deploy-stage/ella-world/config/"

echo "🛡️  [2/4] Backing up existing production binary non-destructively & installing update..."
ssh -o BatchMode=yes "${SSH_HOST}" "STAMP='${STAMP}' bash -s" <<'REMOTE_DEPLOY'
set -euo pipefail

if [[ -f /opt/ella-world/ella-world-server ]]; then
  sudo -n cp -p /opt/ella-world/ella-world-server "/opt/ella-world/ella-world-server.bak.${STAMP}"
fi

sudo -n mkdir -p /opt/ella-world/public/download /opt/ella-world/config /var/lib/ella-world/playgrounds
sudo -n chown -R robert:robert /opt/ella-world/public /opt/ella-world/config /var/lib/ella-world/playgrounds

rsync -az ~/deploy-stage/ella-world/public/ /opt/ella-world/public/
rsync -az ~/deploy-stage/ella-world/config/ /opt/ella-world/config/

chmod +x ~/deploy-stage/ella-world/ella-world-server.new
sudo -n mv ~/deploy-stage/ella-world/ella-world-server.new /opt/ella-world/ella-world-server
sudo -n chown root:root /opt/ella-world/ella-world-server
sudo -n chmod 0755 /opt/ella-world/ella-world-server

OVERRIDE_FILE="/etc/systemd/system/ella-world.service.d/override.conf"
sudo -n mkdir -p /etc/systemd/system/ella-world.service.d
if ! sudo -n grep -q "^Environment=ELLA_SUPERADMIN_PASSWORD=" "${OVERRIDE_FILE}" 2>/dev/null; then
  GEN_SUPER_PW="super-$(head -c 16 /dev/urandom | od -An -tx1 | tr -d ' \n' | cut -c1-24)"
  if ! sudo -n test -f "${OVERRIDE_FILE}"; then
    printf "[Service]\nEnvironment=ELLA_SUPERADMIN_PASSWORD=%s\n" "${GEN_SUPER_PW}" | sudo -n tee "${OVERRIDE_FILE}" > /dev/null
  else
    sudo -n cp -p "${OVERRIDE_FILE}" "/var/lib/ella-world/override.conf.bak.${STAMP}"
    printf "\nEnvironment=ELLA_SUPERADMIN_PASSWORD=%s\n" "${GEN_SUPER_PW}" | sudo -n tee -a "${OVERRIDE_FILE}" > /dev/null
  fi
  sudo -n chmod 0600 "${OVERRIDE_FILE}"
fi

sudo -n systemctl daemon-reload
sudo -n systemctl restart ella-world.service
sleep 1.5
sudo -n systemctl is-active ella-world.service
REMOTE_DEPLOY

echo "✅ Production deployment to ${SSH_HOST} (/opt/ella-world/) active!"
