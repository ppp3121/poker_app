'use client'; // ブラウザ側で動くインタラクティブなコンポーネントであることを示す

import { useState } from 'react';

export default function Home() {
  // サーバーからの応答、ローディング状態、エラーを管理するための状態変数
  const [message, setMessage] = useState<string>('');
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  // ボタンがクリックされたときに実行される関数
  const handleHealthCheck = async () => {
    setIsLoading(true);
    setError(null);
    setMessage('');

    try {
      // RustサーバーのAPIエンドポイントを叩く
      const response = await fetch('http://localhost:8000/api/health');

      // レスポンスが成功でなければエラーを投げる
      if (!response.ok) {
        throw new Error('サーバーとの通信に失敗しました。');
      }

      // レスポンスのJSONを解析
      const data = await response.json();

      // 受け取ったデータを文字列に変換してstateにセット
      setMessage(JSON.stringify(data));

    } catch (err: any) {
      // エラーが発生した場合
      setError(err.message);
    } finally {
      // ローディング状態を解除
      setIsLoading(false);
    }
  };

  return (
    <main style={{ padding: '2rem' }}>
      <h1>Rust + Next.js 通信テスト</h1>
      <button onClick={handleHealthCheck} disabled={isLoading}>
        {isLoading ? '通信中...' : 'サーバーの状態を確認'}
      </button>

      {/* サーバーからの応答を表示するエリア */}
      <div style={{ marginTop: '1rem', fontFamily: 'monospace' }}>
        {message && <p>サーバーからの応答: {message}</p>}
        {error && <p style={{ color: 'red' }}>エラー: {error}</p>}
      </div>
    </main>
  );
}