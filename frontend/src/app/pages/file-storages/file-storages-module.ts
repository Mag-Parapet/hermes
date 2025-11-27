import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule } from '@angular/router';



@NgModule({
  declarations: [],
  imports: [
    RouterModule.forChild([
      { path: '', loadComponent: () => import('./file-storages').then(m => m.FileStorages) },
      { path: 'add', loadComponent: () => import('./pages/add/add').then(m => m.Add) },
      { path: ':id', loadChildren: () => import('./pages/details/details-module').then(m => m.DetailsModule) },
      { path: ':id/edit', loadComponent: () => import('./pages/edit/edit').then(m => m.Edit) },
    ])
  ]
})
export class FileStoragesModule { }
