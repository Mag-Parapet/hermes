import { isPlatformBrowser } from '@angular/common';
import { effect, inject, Injectable, PLATFORM_ID, signal, WritableSignal } from '@angular/core';

@Injectable({
  providedIn: 'root',
})
export class ThemeService {
  private readonly STORAGE_KEY = 'dark-mode';
  
  private _isDarkMode: WritableSignal<boolean> = signal(false);

  public readonly isDarkMode = this._isDarkMode.asReadonly();
  
  constructor() {
    this._isDarkMode.set(this.getInitialThemeValue());

    effect(() => {
      const isDark = this._isDarkMode();
      const html = document.documentElement;

      if (isDark) {
        html.classList.add('dark');
        localStorage.setItem(this.STORAGE_KEY, 'true');
      } else {
        html.classList.remove('dark');
        localStorage.setItem(this.STORAGE_KEY, 'false');
      }
    });
  }

  toggleDarkMode(enable: boolean): void {
    this._isDarkMode.set(enable);
  }

  private getInitialThemeValue(): boolean {
    const saved = localStorage.getItem(this.STORAGE_KEY);
    let useDark = false;

    if (saved === null) {
      useDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    } else {
      useDark = saved === 'true';
    }
    
    return useDark;
  }
}
