import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule } from '@angular/router';
import { Layout } from './layout';



@NgModule({
  declarations: [],
  imports: [
    RouterModule.forChild([
      { path: '', component: Layout, children: [
        { path: '', loadComponent: () => import('@pages/home/home').then(m => m.Home) },
        { path: 'domains', loadChildren: () => import('@pages/domains/domains-module').then(m => m.DomainsModule) },
        { path: 'file-storages', loadChildren: () => import('@pages/file-storages/file-storages-module').then(m => m.FileStoragesModule) },
        { path: '**', redirectTo: 'domains' }
      ] }
    ])
  ]
})
export class LayoutModule { }
