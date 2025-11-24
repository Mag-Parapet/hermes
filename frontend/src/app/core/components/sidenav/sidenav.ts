import { AsyncPipe } from '@angular/common';
import { Component, inject, PLATFORM_ID, signal } from '@angular/core';
import { RouterLink, RouterLinkActive } from '@angular/router';
import { AuthService } from '@core/services/auth-service';
import { ThemeService } from '@core/services/theme-service';
import { Store } from '@ngrx/store';
import { selectSidenavState } from 'app/state/sidenav/sidenav.reduce';

@Component({
  selector: 'app-sidenav',
  imports: [RouterLink, RouterLinkActive, AsyncPipe],
  templateUrl: './sidenav.html',
  styleUrl: './sidenav.css',
})
export class Sidenav {
  store = inject(Store)
  authService = inject(AuthService)
  platformId = inject(PLATFORM_ID)
  themeService = inject(ThemeService)
  
  isDarkMode = this.themeService.isDarkMode
  
  collapsed = signal<boolean>(true);
  isMobile = signal<boolean>(false);
  
  collapsed$ = this.store.select(selectSidenavState)
  
  constructor() {
    this.isMobile.set(window.matchMedia('(max-width: 768px)').matches);
  }

  menu = [
    { title: 'Home', icon: 'dashboard', route: '/' },
    { title: 'Domains', icon: 'host', route: '/domains' },
    { title: 'File Storages', icon: 'box', route: '/file-storages' },
  ]

  onToggleTheme() {
    const isDark = this.themeService.isDarkMode();
    this.themeService.toggleDarkMode(!isDark);
  }

  onLogout() {
    this.authService.logout();
  }
}
