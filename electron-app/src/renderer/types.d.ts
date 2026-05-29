/* eslint-disable @typescript-eslint/no-explicit-any */

interface TotpConfig {
  secret: string;
  algorithm?: string;
  digits?: number;
  period?: number;
}

interface TotpResult {
  code: string;
  elapsed: number;
  period: number;
  remaining: number;
  expiring: boolean;
}

interface TotpUtil {
  computeTotp(conf: TotpConfig): Promise<TotpResult>;
}

interface PasswordConfig {
  length?: number;
  uppercase?: boolean;
  lowercase?: boolean;
  digits?: boolean;
  symbols?: boolean;
  excludeAmbiguous?: boolean;
}

interface Category {
  key: string;
  label: string;
  icon: string;
  color: string;
  createdAt?: number;
  updatedAt?: number;
}

interface VaultApi {
  check(filePath: string): Promise<{ ok: boolean; exists?: boolean; error?: string }>;
  open(filePath: string): Promise<{ ok: boolean; error?: string }>;
  create(filePath: string, password: string): Promise<{ ok: boolean; error?: string }>;
  unlock(password: string): Promise<{ ok: boolean; error?: string }>;
  lock(): Promise<{ ok: boolean }>;
  getEntries(): Promise<{ ok: boolean; entries?: any[]; error?: string }>;
  addEntry(entry: any): Promise<{ ok: boolean; error?: string }>;
  updateEntry(entry: any): Promise<{ ok: boolean; error?: string }>;
  deleteEntry(id: string): Promise<{ ok: boolean; error?: string }>;
  changePassword(oldPwd: string, newPwd: string): Promise<{ ok: boolean; error?: string }>;
  exportJson(filePath: string): Promise<{ ok: boolean; error?: string }>;
  getCategories(): Promise<{ ok: boolean; categories?: Category[]; error?: string }>;
  addCategory(cat: Category): Promise<{ ok: boolean; error?: string }>;
  updateCategory(cat: Category): Promise<{ ok: boolean; error?: string }>;
  deleteCategory(key: string): Promise<{ ok: boolean; error?: string }>;
}

interface DialogApi {
  openFile(): Promise<string | null>;
  saveFile(): Promise<string | null>;
  exportJson(): Promise<string | null>;
}

interface ClipboardApi {
  write(text: string): Promise<{ ok: boolean }>;
}

interface GeneratorApi {
  generate(config: PasswordConfig): Promise<{ ok: boolean; password?: string }>;
  evaluate(password: string): Promise<{ ok: boolean; strength?: string }>;
}

interface ThemeApi {
  getSystem(): Promise<{ dark: boolean }>;
  onSystemChanged(cb: (dark: boolean) => void): void;
}

interface SettingsApi {
  update(settings: Record<string, any>): Promise<{ ok: boolean }>;
}

interface VaultXAPI {
  vault: VaultApi;
  dialog: DialogApi;
  clipboard: ClipboardApi;
  generator: GeneratorApi;
  theme: ThemeApi;
  settings: SettingsApi;
  onVaultLocked(cb: () => void): void;
  pingActivity(): void;
}

interface Window {
  vaultxAPI: VaultXAPI;
  TotpUtil: TotpUtil;
}
