'use client';

import { useUserStore } from '@/store/userStore';
import { useEffect, useState } from 'react';

// このコンポーネントはUIを持たず、アプリの初期化ロジックのみを担当します
export default function AuthInitializer() {
  const { login, logout } = useUserStore();
  const [initialized, setInitialized] = useState(false);

  useEffect(() => {
    // 一度だけ実行されるようにする
    if (initialized) {
      return;
    }
    setInitialized(true);

    const checkAuthStatus = async () => {
      try {
        // Cookieを自動で送信してユーザー情報を取得
        const response = await fetch('http://localhost:8000/api/me', {
          // GETリクエストではbodyは不要
          method: 'GET',
          headers: {
            // Content-TypeもGETでは通常不要ですが、念のため
            'Content-Type': 'application/json',
          },
          credentials: 'include',
        });

        if (response.ok) {
          const data = await response.json();
          // バックエンドのClaims構造体のsubフィールドからユーザー名を取得
          if (data.sub) {
            login(data.sub);
          }
        } else {
          // 401 Unauthorizedなど、セッションが無効な場合はログアウト状態にする
          logout();
        }
      } catch (error) {
        console.error('認証状態の確認に失敗しました:', error);
        logout();
      }
    };

    checkAuthStatus();
  }, [initialized, login, logout]);

  // このコンポーネントは何もレンダリングしない
  return null;
}