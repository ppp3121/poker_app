import { create } from 'zustand';

// ストアで管理する状態の型を定義
interface UserState {
  username: string | null;
  isLoggedIn: boolean;
  login: (username: string) => void;
  logout: () => void;
}

// ストアを作成
export const useUserStore = create<UserState>((set) => ({
  // 初期状態
  username: null,
  isLoggedIn: false,

  // 状態を更新するためのアクション
  login: (username) => set({ username, isLoggedIn: true }),
  logout: () => set({ username: null, isLoggedIn: false }),
}));