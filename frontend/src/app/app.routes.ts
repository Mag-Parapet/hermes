import { Routes } from '@angular/router';
import { authGuard } from '@core/guards/auth-guard';
import { logoutGuard } from '@core/guards/logout-guard';

export const routes: Routes = [
    { 
        path: 'auth', 
        loadComponent: () => import('@pages/auth/auth').then(m => m.Auth),
        canActivate: [logoutGuard] 
    },
    { 
        path: '', 
        loadChildren: () => import('@core/components/layout/layout-module').then(m => m.LayoutModule),
        canActivate: [authGuard]
    }
];
