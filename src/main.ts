import { mount } from 'svelte';
import App from './App.svelte';
import './app.css';
import './themes.css';
import { applyAppearance, styleStorageKey, themePreference, themeStorageKey, themeStyle } from './lib/theme';

const savedTone = localStorage.getItem(themeStorageKey);
applyAppearance(themeStyle(localStorage.getItem(styleStorageKey), savedTone), themePreference(savedTone), matchMedia('(prefers-color-scheme: dark)').matches);

mount(App, { target: document.getElementById('app')! });
