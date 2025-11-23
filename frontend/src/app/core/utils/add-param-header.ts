import { inject, PLATFORM_ID } from '@angular/core';
import { isPlatformBrowser } from '@angular/common';
import { NavigationEnd, Router } from '@angular/router';
import { OperatorFunction, pipe, tap } from 'rxjs';

export function addParamHeader<T>(param: string, field: string): OperatorFunction<T, T> {
  return (source$) => {
    const router = inject(Router);

    return source$.pipe(
      tap((res: any) => {
        const fields = field.split('.');
        for (let f of fields) {
          res = res[f];
          if (res == null) break;
        }
        if (res != null) {
          history.replaceState(
            { ...history.state, [param]: res },
            ''
          );

          (router.events as any).next(new NavigationEnd(0, router.url, router.url));
        }
      })
    );
  };
}
