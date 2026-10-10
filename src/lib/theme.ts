export type Theme = 'light' | 'dark';
export type ThemePreference = 'system' | Theme;
export type ThemeStyle = 'modern' | 'archaic' | 'nimb';

export const themeStorageKey = 'a-guilda-vtt-theme';
export const styleStorageKey = 'a-guilda-vtt-style';

export function themeStyle(saved: string | null, legacyTheme: string | null): ThemeStyle {
  if (saved === 'modern' || saved === 'archaic' || saved === 'nimb') return saved;
  return legacyTheme === 'system' || legacyTheme === 'light' || legacyTheme === 'dark' ? 'archaic' : 'modern';
}

export function resolveTheme(saved: string | null, prefersDark: boolean): Theme {
  return saved === 'light' || saved === 'dark' ? saved : prefersDark ? 'dark' : 'light';
}

export function themePreference(saved:string|null):ThemePreference {
  return saved==='light'||saved==='dark'?saved:'system';
}

export function appearanceTheme(style: ThemeStyle, choice: ThemePreference, prefersDark: boolean): Theme {
  return style === 'modern' ? 'dark' : style === 'nimb' ? 'light' : resolveTheme(choice, prefersDark);
}

export function applyAppearance(style: ThemeStyle, choice: ThemePreference, prefersDark: boolean): void {
  const theme = appearanceTheme(style, choice, prefersDark);
  document.documentElement.dataset.style = style;
  document.documentElement.dataset.theme = theme;
  document.querySelector<HTMLMetaElement>('meta[name="theme-color"]')?.setAttribute('content',
    style === 'modern' ? '#181818' : style === 'nimb' ? '#cf2e35' : theme === 'dark' ? '#0b0c0d' : '#eee8dc');
  window.dispatchEvent(new Event('guilda-theme-change'));
}
