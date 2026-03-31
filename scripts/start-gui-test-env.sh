#!/bin/bash
# FactorioModTranslator リモートGUIテスト環境 起動スクリプト

DISPLAY_NUM=99
VNC_PORT=5900
NOVNC_PORT=6080
SCREEN_RESOLUTION=1280x720x24

echo "=== FactorioModTranslator リモートGUIテスト環境 ==="
echo ""

# 既存プロセスを停止
echo "既存プロセスを停止中..."
pkill -f "Xvfb :${DISPLAY_NUM}" 2>/dev/null
pkill -f "x11vnc.*:${VNC_PORT}" 2>/dev/null
pkill -f "websockify.*${NOVNC_PORT}" 2>/dev/null
pkill -f "openbox" 2>/dev/null
sleep 1

# Xvfbを開始
echo "Xvfbを起動中 (DISPLAY=:${DISPLAY_NUM}, 解像度: ${SCREEN_RESOLUTION})..."
Xvfb :${DISPLAY_NUM} -screen 0 ${SCREEN_RESOLUTION} -ac +extension GLX +render -noreset &
XVFB_PID=$!
sleep 2

if ! ps -p $XVFB_PID > /dev/null 2>&1; then
    echo "エラー: Xvfbの起動に失敗しました"
    exit 1
fi
echo "Xvfb起動成功 (PID: ${XVFB_PID})"

# DISPLAY環境変数を設定
export DISPLAY=:${DISPLAY_NUM}

# ウィンドウマネージャーを開始
echo "Openboxを起動中..."
openbox &
sleep 1
echo "Openbox起動成功"

# x11vncを開始（パスワードなし）
echo "x11vncを起動中 (ポート: ${VNC_PORT})..."
x11vnc -display :${DISPLAY_NUM} -forever -nopw -rfbport ${VNC_PORT} -bg -o /tmp/x11vnc.log
sleep 1
echo "x11vnc起動成功"

# noVNC (websockify) を開始
echo "noVNCを起動中 (ポート: ${NOVNC_PORT})..."
NOVNC_PATH=$(find /usr/share -name "vnc.html" -path "*/novnc/*" 2>/dev/null | head -1 | xargs dirname 2>/dev/null)
if [ -z "$NOVNC_PATH" ]; then
    NOVNC_PATH="/usr/share/novnc"
fi
websockify --web=$NOVNC_PATH ${NOVNC_PORT} localhost:${VNC_PORT} &
NOVNC_PID=$!
sleep 1
echo "noVNC起動成功"

# IPアドレスを取得
IP_ADDR=$(hostname -I 2>/dev/null | awk '{print $1}')
if [ -z "$IP_ADDR" ]; then
    IP_ADDR="localhost"
fi

echo ""
echo "========================================="
echo " リモートGUIテスト環境 起動完了"
echo "========================================="
echo ""
echo "VNC接続:  ${IP_ADDR}:${VNC_PORT}"
echo "noVNC (ブラウザ): http://${IP_ADDR}:${NOVNC_PORT}/vnc.html"
echo ""
echo "Tauriアプリを起動するには:"
echo "  export DISPLAY=:${DISPLAY_NUM}"
echo "  export PATH=\"\$PATH:/home/ubuntu/.cargo/bin\""
echo "  cd /home/ubuntu/workspace/FactorioModTranslator"
echo "  npm run tauri dev"
echo ""
echo "停止するには: ./scripts/stop-gui-test-env.sh"
echo "========================================="

# PIDを保存
echo "${XVFB_PID}" > /tmp/xvfb.pid
echo "${NOVNC_PID}" > /tmp/novnc.pid
