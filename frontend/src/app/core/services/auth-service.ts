import { Injectable, signal, WritableSignal, inject, computed } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable, BehaviorSubject, throwError, tap, catchError, map, switchMap, filter, take } from 'rxjs';
import { Store } from '@ngrx/store';
import { Router } from '@angular/router';
import { setLoading } from '../../state/loading/loading.actions';
import { jwtDecode } from "jwt-decode";
import Snackbar from 'awesome-snackbar';
import { environment } from 'environments/environment';
import { Auth } from '@pages/auth/auth';

interface AuthResponse {
  data: {
    accessToken: string;
    refreshToken: string;
  },
  status: 'success' | 'failed';
  message?: string;
}

export interface UserPayload {
  id?: string;
  _id?: string;
  email: string;
  name?: string;
  isAdmin?: boolean;
  exp?: number;
  [key: string]: any;
}

@Injectable({
  providedIn: 'root'
})
export class AuthService {
  private router = inject(Router);
  private store = inject(Store);
  private http = inject(HttpClient);
  
  private apiUrl = environment.API_URL + 'auth/'; 

  accessToken: WritableSignal<string | null> = signal(localStorage.getItem('accessToken'));

  currentUser = computed<UserPayload | null>(() => {
    const token = this.accessToken();
    if (!token) return null;
    try {
      return jwtDecode<UserPayload>(token);
    } catch (error) {
      console.error('Token decode error:', error);
      return null;
    }
  });

  isAuthenticated = computed(() => !!this.accessToken());
  
  isAdmin = computed(() => {
    const user = this.currentUser();
    return user?.isAdmin === true;
  });

  private isRefreshing = false;
  private refreshTokenSubject = new BehaviorSubject<string | null>(null);

  constructor() {}

  authenticate(email: string, password: string): Observable<AuthResponse> {
    this.store.dispatch(setLoading({ state: true }));
    return this.http.post<AuthResponse>(`${this.apiUrl}login`, { email, password }).pipe(
      tap((response) => {
        this.router.navigate(['/']);
        this.updateTokens(response.data.accessToken, response.data.refreshToken);
      }),
      catchError((error) => {
        new Snackbar(`Authentication failed. Please check your credentials.`, {
          iconSrc: '/error.png',
          position: 'bottom-right',
        });
        // const snackbar = new Snackbar('Authentication failed. Please check your credentials.', {
        //   position: 'top-right',
        // });
        return throwError(() => error);
      }),
      tap(() => {
        this.store.dispatch(setLoading({ state: false }));
      })
    );
  }

  refreshTokenSafely(): Observable<string> {
    if (this.isRefreshing) {
      return this.refreshTokenSubject.pipe(
        filter(token => token !== null),
        take(1),
        map(token => token!)
      );
    } else {
      this.isRefreshing = true;
      this.refreshTokenSubject.next(null);

      const refreshToken = localStorage.getItem('refreshToken');

      return this.http.post<AuthResponse>(`${this.apiUrl}refresh`, { refreshToken }).pipe(
        tap((response) => {
          if (response.status === 'failed') {
            this.logout();
            throw new Error('Refresh token invalid');
          }
          this.isRefreshing = false;
          this.updateTokens(response.data.accessToken, response.data.refreshToken);
          this.refreshTokenSubject.next(response.data.accessToken);
        }),
        catchError((err) => {
          this.isRefreshing = false;
          this.logout();
          return throwError(() => err);
        }),
        map(response => response.data.accessToken)
      );
    }
  }

  updateTokens(access: string, refresh: string) {
    localStorage.setItem('accessToken', access);
    localStorage.setItem('refreshToken', refresh);
    this.accessToken.set(access);
  }

  logout() {
    localStorage.removeItem('accessToken');
    localStorage.removeItem('refreshToken');
    this.accessToken.set(null);
    
    this.router.navigate(['/auth']);
  }
}