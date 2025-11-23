import { HttpInterceptorFn, HttpErrorResponse } from '@angular/common/http';
import { inject } from '@angular/core';
import { catchError, switchMap, throwError } from 'rxjs';
import { AuthService } from '../services/auth-service';

export const tokenInterceptor: HttpInterceptorFn = (req, next) => {
  const authService = inject(AuthService);
  const token = authService.accessToken();

  if (token && !req.url.includes('/refresh')) {
    req = req.clone({
      setHeaders: { Authorization: `Bearer ${token}` }
    });
  }

  return next(req).pipe(
    catchError((error) => {
      
      if (error instanceof HttpErrorResponse && error.status === 401 && !req.url.includes('/refresh')) {
        
        return authService.refreshTokenSafely().pipe(
          switchMap((newToken) => {
            const clonedReq = req.clone({
              setHeaders: { Authorization: `Bearer ${newToken}` }
            });
            return next(clonedReq);
          }),
          catchError((refreshErr) => {
             return throwError(() => refreshErr);
          })
        );
      }

      return throwError(() => error);
    })
  );
};