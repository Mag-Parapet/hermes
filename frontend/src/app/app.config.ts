import { ApplicationConfig, provideBrowserGlobalErrorListeners } from '@angular/core';
import { InMemoryScrollingFeature, InMemoryScrollingOptions, provideRouter, withInMemoryScrolling, withPreloading } from '@angular/router';

import { routes } from './app.routes';
import { provideState, provideStore } from '@ngrx/store';
import { sidenavFeature } from './state/sidenav/sidenav.reduce';
import { loadingFeature } from './state/loading/loading.reduce';
import { provideHttpClient, withInterceptors } from '@angular/common/http';
import { tokenInterceptor } from '@core/interceptors/token-interceptor';
import { quicklinkProviders, QuicklinkStrategy } from 'ngx-quicklink';

const scrollConfig: InMemoryScrollingOptions = {
  scrollPositionRestoration: 'top',
  anchorScrolling: 'enabled',
};

const inMemoryScrollingFeature: InMemoryScrollingFeature =
  withInMemoryScrolling(scrollConfig);

export const appConfig: ApplicationConfig = {
  providers: [
    provideBrowserGlobalErrorListeners(),
    provideRouter(routes, inMemoryScrollingFeature, withPreloading(QuicklinkStrategy)),
    provideState(loadingFeature),
    provideState(sidenavFeature),
    provideStore(),
    provideHttpClient(withInterceptors([tokenInterceptor])),
    quicklinkProviders
  ]
};
