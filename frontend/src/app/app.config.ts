import { ApplicationConfig, provideBrowserGlobalErrorListeners } from '@angular/core';
import { InMemoryScrollingFeature, InMemoryScrollingOptions, provideRouter, withInMemoryScrolling } from '@angular/router';

import { routes } from './app.routes';
import { provideState, provideStore } from '@ngrx/store';
import { sidenavFeature } from './state/sidenav/sidenav.reduce';
import { loadingFeature } from './state/loading/loading.reduce';
import { provideHttpClient, withInterceptors } from '@angular/common/http';
import { tokenInterceptor } from '@core/interceptors/token-interceptor';

const scrollConfig: InMemoryScrollingOptions = {
  scrollPositionRestoration: 'top',
  anchorScrolling: 'enabled',
};

const inMemoryScrollingFeature: InMemoryScrollingFeature =
  withInMemoryScrolling(scrollConfig);

export const appConfig: ApplicationConfig = {
  providers: [
    provideBrowserGlobalErrorListeners(),
    provideRouter(routes, inMemoryScrollingFeature),
    provideState(loadingFeature),
    provideState(sidenavFeature),
    provideStore(),
    provideHttpClient(withInterceptors([tokenInterceptor])),
  ]
};
