#!/bin/bash
# FactorioModTranslator リモートGUIテスト環境 停止スクリプト

echo "=== リモートGUIテスト環境を停止中 ==="

# プロセスを停止
pkill -f "Xvfb :99" 2>/dev/null && echo "Xvfb stopped"
pkill -f "x11vnc" 2>/dev/null && echo "x11vnc stopped"
pkill -f "websockify" 2>/dev/null && echo "noVNC stopped"
pkill -f "openbox" 2>/dev/null && echo "openbox stopped"

# PIDファイルを削除
rm -f /tmp/xvfb.pid /tmp/novnc.pid 2>/dev/null

echo "完了"
