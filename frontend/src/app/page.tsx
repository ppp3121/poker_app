import Link from 'next/link';

export default function Home() {
  return (
    <main style={{ padding: '2rem', textAlign: 'center' }}>
      <h1>ポーカーアプリへようこそ！</h1>
      <nav style={{ marginTop: '2rem', display: 'flex', gap: '1rem', justifyContent: 'center' }}>
        <Link href="/register" style={{ padding: '0.5rem 1rem', border: '1px solid white' }}>
          ユーザー登録
        </Link>
        <Link href="/login" style={{ padding: '0.5rem 1rem', border: '1px solid white' }}>
          ログイン
        </Link>
      </nav>
    </main>
  );
}